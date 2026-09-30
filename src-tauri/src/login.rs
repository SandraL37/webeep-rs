use crate::AppState;
use base64::Engine;
use tauri::{Emitter, Manager, Url, WebviewUrl, WebviewWindowBuilder};
use tokio::sync::MutexGuard;

#[derive(Debug, Clone, Copy, serde::Serialize, specta::Type)]
pub enum LoginError {
    LoginWindowClosed,
    IncompatibleWithCurrentWeBeep,
    KeyringError,
    TauriError,
}

impl From<LoginError> for std::string::String {
    fn from(e: LoginError) -> Self {
        format!("{:?}", e)
    }
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
    app_handle: tauri::AppHandle,
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

    fn set_state(&mut self, state: LoginState) -> Result<(), LoginError> {
        match &state {
            LoginState::Logged { token } => Self::write_token(Some(token))?,
            LoginState::NotLogged | LoginState::Logging | LoginState::Error { .. } => {
                Self::write_token(None)?
            }
        }
        self.state = state;

        self.app_handle
            .emit("login-state-changed", self.state.clone())?;
        Ok(())
    }

    fn capture_token(&mut self, target_url: &Url) -> Result<Option<String>, LoginError> {
        // Logged in
        if target_url.as_str() == Self::WEBEEP_MY_URL {
            let Some(window) = self.app_handle.get_webview_window(Self::WINDOW_LABEL) else {
                return Err(LoginError::LoginWindowClosed);
            };
            let redirect_url = Self::url(Self::WEBEEP_MOODLE_REDIRECT);
            window.navigate(redirect_url)?;
        // Capture the token
        } else if target_url.as_str().starts_with(Self::MOODLE_PROTOCOL) {
            if let Some(window) = self.app_handle.get_webview_window(Self::WINDOW_LABEL) {
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

            self.set_state(LoginState::Logged {
                token: String::from(token),
            })?;

            return Ok(Some(String::from(token)));
        }

        Ok(None)
    }

    pub fn get_state(&self) -> LoginState {
        self.state.clone()
    }

    const WINDOW_LABEL: &str = "polimi-login";
    pub async fn login(&mut self) -> Result<(), LoginError> {
        // Set focus on existing window, if any (Silent failure is ok)
        self.app_handle
            .get_webview_window(Self::WINDOW_LABEL)
            .and_then(|window| window.set_focus().ok());

        self.set_state(LoginState::Logging)?;

        // Clones for the closures
        let app_handle_nav = self.app_handle.clone();
        let app_handle_evt = self.app_handle.clone();

        // Build the embedded secondary window
        let window = WebviewWindowBuilder::new(
            &self.app_handle,
            Self::WINDOW_LABEL,
            WebviewUrl::External(Self::url(Self::WEBEEP_AUTH_URL)),
        )
        .title("Polimi Login")
        .inner_size(1000.0, 600.0)
        .focused(true)
        .on_navigation(move |target_url| {
            let target_url = target_url.clone();
            let app_handle = app_handle_nav.clone();

            // Spawn async task to acquire lock and run capture_token
            tauri::async_runtime::spawn(async move {
                let mut manager = Self::instance(&app_handle).await;
                let result = manager.capture_token(&target_url);

                if let Err(e) = result {
                    manager.set_state(LoginState::Error { error: e }).ok();
                }
            });

            true
        })
        .build()?;

        window.clear_all_browsing_data()?;

        window.on_window_event(move |event| {
            if let tauri::WindowEvent::Destroyed = event {
                let app_handle = app_handle_evt.clone();

                // Spawn async task to acquire lock and update state
                tauri::async_runtime::spawn(async move {
                    let mut manager = Self::instance(&app_handle).await;
                    if matches!(manager.get_state(), LoginState::Logging) {
                        let _ = manager.set_state(LoginState::NotLogged);
                    }
                });
            }
        });

        Ok(())
    }

    pub async fn logout(&mut self) -> Result<(), LoginError> {
        Self::write_token(None)?;
        self.set_state(LoginState::NotLogged)?;

        Ok(())
    }

    pub fn new(app_handle: tauri::AppHandle) -> Self {
        if let Some(token) = Self::read_token() {
            Self {
                state: LoginState::Logged { token },
                app_handle,
            }
        } else {
            Self {
                state: LoginState::NotLogged,
                app_handle,
            }
        }
    }
}

impl LoginManager {
    pub async fn instance<'a>(app_handle: &'a tauri::AppHandle) -> MutexGuard<'a, LoginManager> {
        app_handle
            .state::<AppState>()
            .inner()
            .login_manager
            .lock()
            .await
    }
}
