use crate::{
    login::{LoginManager, LoginState},
    moodle::MoodleService,
};
use std::sync::Arc;
use tauri::{Manager, async_runtime::Mutex};

mod login;
pub mod moodle;

#[tauri::command]
#[specta::specta]
async fn get_site_info(app_handle: tauri::AppHandle) -> Result<moodle::models::SiteInfo, String> {
    let result = MoodleService::instance(&app_handle).get_site_info().await?;
    Ok(result)
}

#[tauri::command]
#[specta::specta]
async fn login(app_handle: tauri::AppHandle) -> Result<(), String> {
    LoginManager::instance(&app_handle).await.login().await?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
async fn logout(app_handle: tauri::AppHandle) -> Result<(), String> {
    LoginManager::instance(&app_handle).await.logout().await?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
async fn get_login_state(app_handle: tauri::AppHandle) -> Result<LoginState, String> {
    Ok(LoginManager::instance(&app_handle).await.get_state())
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder =
        tauri_specta::Builder::<tauri::Wry>::new().commands(tauri_specta::collect_commands![
            get_site_info,
            get_login_state,
            login,
            logout,
        ]);

    #[cfg(debug_assertions)]
    builder
        .export(
            specta_typescript::Typescript::default(),
            "../src/bindings.ts",
        )
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .invoke_handler(builder.invoke_handler())
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
    moodle_service: Arc<moodle::MoodleService>,
}

impl AppState {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self {
            login_manager: Mutex::new(login::LoginManager::new(app_handle.clone())),
            moodle_service: Arc::new(moodle::MoodleService::new(app_handle)),
        }
    }
}
