use tauri::{AppHandle, Manager, async_runtime::Mutex};

use crate::login::FrontendLoginState;

mod login;
pub mod moodle;

#[tauri::command]
async fn get_site_info(
    state: tauri::State<'_, AppState>,
) -> Result<moodle::models::SiteInfo, String> {
    let site_info = state
        .moodle_service
        .get_site_info()
        .await
        .map_err(|e| e.to_string())?;
    Ok(site_info)
}

#[tauri::command]
async fn login(app: tauri::AppHandle) -> Result<(), String> {
    login::LoginManager::login(app)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn logout(app: tauri::AppHandle) -> Result<(), String> {
    login::LoginManager::logout(app)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn get_login_state(state: tauri::State<'_, AppState>) -> Result<FrontendLoginState, String> {
    let state = state
        .login_manager
        .lock()
        .await
        .get_state()
        .get_frontend_state();
    Ok(state)
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            app.manage(AppState::new(app.app_handle().clone()));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            login,
            logout,
            get_login_state,
            get_site_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

struct AppState {
    login_manager: Mutex<login::LoginManager>,
    moodle_service: moodle::MoodleService,
}

impl AppState {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            login_manager: Mutex::new(login::LoginManager::new()),
            moodle_service: moodle::MoodleService::new(app_handle),
        }
    }
}
