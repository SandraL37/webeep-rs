use thiserror::Error;

#[derive(Debug, Clone, Error, serde::Serialize, specta::Type)]
pub enum LoginError {
    #[error("Login window closed")]
    LoginWindowClosed,

    #[error("Incompatible with current WeBeep version")]
    IncompatibleWithCurrentWeBeep,

    #[error("Keyring error: {0}")]
    KeyringError(String),

    #[error("Tauri error: {0}")]
    TauriError(String),
}

impl From<keyring::Error> for LoginError {
    fn from(e: keyring::Error) -> Self {
        LoginError::KeyringError(e.to_string())
    }
}

impl From<tauri::Error> for LoginError {
    fn from(e: tauri::Error) -> Self {
        LoginError::TauriError(e.to_string())
    }
}

impl From<LoginError> for String {
    fn from(e: LoginError) -> Self {
        e.to_string()
    }
}
