use crate::AppState;
use base64::Engine;
use tauri::{Emitter, Manager, Url, WebviewUrl, WebviewWindowBuilder};

#[derive(Debug, Clone, Copy, serde::Serialize, specta::Type)]
pub enum LoginError {
    LoginWindowClosed,
    IncompatibleWithCurrentWeBeep,
    KeyringError,
    TauriError,
}

impl From<keyring::Error> for LoginError {
    fn from(_: keyring::Error) -> Self {
        LoginError::KeyringError
    }
}

impl From<tauri::Error> for LoginError {
    fn from(_: tauri::Error) -> Self {
        LoginError::TauriError
    }
}

impl std::fmt::Display for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[LoginError]: {:?}", self)
    }
}

impl std::error::Error for LoginError {}

#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub enum LoginState {
    Logged { token: String },
    NotLogged,
    Logging,
    Error { error: LoginError },
}

pub struct LoginManager {
    state: LoginState,
}

impl LoginManager {
    const WEBEEP_AUTH_URL: &str = "http://webeep.polimi.it/auth/shibboleth/index.php";
    const WEBEEP_MY_URL: &str = "https://webeep.polimi.it/my/";
    const WEBEEP_MOODLE_REDIRECT: &str = "https://webeep.polimi.it/admin/tool/mobile/launch.php?service=moodle_mobile_app&passport=12345";
    const MOODLE_PROTOCOL: &str = "moodlemobile://";

    // A shortcut for inizializing the Url.
    // The parse is unchecked so this call must not fail.
    fn url(url: &str) -> Url {
        Url::parse(url).expect("Failed to parse url")
    }

    const SERVICE_NAME: &str = "webeep";
    const ACCOUNT_NAME: &str = "auth_token";

    fn write_token(token: Option<&str>) -> Result<(), LoginError> {
        let entry = keyring::Entry::new(Self::SERVICE_NAME, Self::ACCOUNT_NAME)?;

        match token {
            Some(token) => entry.set_password(token)?,
            None => {
                entry.delete_credential().ok();
            }
        };

        Ok(())
    }

    fn read_token() -> Option<String> {
        let Ok(entry) = keyring::Entry::new(Self::SERVICE_NAME, Self::ACCOUNT_NAME) else {
            return None;
        };

        entry.get_password().ok()
    }

    fn set_state(&mut self, app: &tauri::AppHandle, state: LoginState) -> Result<(), LoginError> {
        match &state {
            LoginState::Logged { token } => Self::write_token(Some(token))?,
            LoginState::NotLogged | LoginState::Logging | LoginState::Error { .. } => {
                Self::write_token(None)?
            }
        }
        self.state = state;

        app.emit("login-state-changed", self.state.clone())?;
        Ok(())
    }

    fn capture_token(
        &mut self,
        target_url: &Url,
        app: &tauri::AppHandle,
    ) -> Result<Option<String>, LoginError> {
        // Logged in
        if target_url.as_str() == Self::WEBEEP_MY_URL {
            let Some(window) = app.get_webview_window(Self::WINDOW_LABEL) else {
                return Err(LoginError::LoginWindowClosed);
            };
            let redirect_url = Self::url(Self::WEBEEP_MOODLE_REDIRECT);
            window.navigate(redirect_url)?;
        // Capture the token
        } else if target_url.as_str().starts_with(Self::MOODLE_PROTOCOL) {
            if let Some(window) = app.get_webview_window(Self::WINDOW_LABEL) {
                window.close()?;
            }

            let Some(b64tokens) = target_url.as_str().split("token=").nth(1) else {
                return Err(LoginError::IncompatibleWithCurrentWeBeep);
            };

            let Ok(byte_tokens) = base64::engine::general_purpose::STANDARD.decode(b64tokens)
            else {
                return Err(LoginError::IncompatibleWithCurrentWeBeep);
            };

            // Can't fail
            let tokens = String::from_utf8(byte_tokens).unwrap();

            let Some(token) = tokens.split(":::").nth(1) else {
                return Err(LoginError::IncompatibleWithCurrentWeBeep);
            };

            self.set_state(
                &app,
                LoginState::Logged {
                    token: String::from(token),
                },
            )?;

            return Ok(Some(String::from(token)));
        }

        Ok(None)
    }

    pub fn get_state(&self) -> LoginState {
        self.state.clone()
    }

    const WINDOW_LABEL: &str = "polimi-login";
    pub async fn login(app: tauri::AppHandle) -> Result<(), LoginError> {
        // Set focus on existing window, if any (Slient failure is ok)
        app.get_webview_window(Self::WINDOW_LABEL)
            .and_then(|window| window.set_focus().ok());

        let app_cloned = app.clone();

        {
            let state = app_cloned.state::<AppState>();
            state
                .login_manager
                .lock()
                .await
                .set_state(&app_cloned, LoginState::Logging)?;
        }

        let app_cloned = app.clone();

        // Build the embedded secondary window
        let window = WebviewWindowBuilder::new(
            &app,
            Self::WINDOW_LABEL,
            WebviewUrl::External(Self::url(Self::WEBEEP_AUTH_URL)),
        )
        .title("Polimi Login")
        .inner_size(1000.0, 600.0)
        .focused(true)
        .on_navigation(move |target_url| {
            let app = app_cloned.clone();
            let target_url = target_url.clone();

            tauri::async_runtime::spawn(async move {
                let state = app.state::<AppState>();
                let mut login_manager = state.login_manager.lock().await;

                let result = login_manager.capture_token(&target_url, &app);

                if let Err(e) = result {
                    login_manager
                        .set_state(&app, LoginState::Error { error: e })
                        .ok();
                }
            });

            true
        })
        .build()?;

        window.clear_all_browsing_data()?;

        let app_on_close = app.clone();
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::Destroyed = event {
                let app = app_on_close.clone();
                tauri::async_runtime::spawn(async move {
                    let state = app.state::<AppState>();
                    let mut login_manager = state.login_manager.lock().await;

                    // Only set to NotLogged if the user closed the window while still logging in
                    if matches!(login_manager.get_state(), LoginState::Logging) {
                        let _ = login_manager.set_state(&app, LoginState::NotLogged);
                    }
                });
            }
        });

        Ok(())
    }

    pub async fn logout(app: tauri::AppHandle) -> Result<(), LoginError> {
        Self::write_token(None)?;
        {
            let state = app.state::<AppState>();

            state
                .login_manager
                .lock()
                .await
                .set_state(&app, LoginState::NotLogged)?;
        }

        Ok(())
    }

    pub fn new() -> Self {
        if let Some(token) = Self::read_token() {
            Self {
                state: LoginState::Logged { token },
            }
        } else {
            Self {
                state: LoginState::NotLogged,
            }
        }
    }
}
