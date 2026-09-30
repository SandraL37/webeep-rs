use std::{collections::HashMap, sync::Arc};

use serde::de::DeserializeOwned;
use tauri::{AppHandle, Manager};
use tauri_plugin_http::reqwest;

use crate::{
    AppState,
    login::{LoginManager, LoginState},
    moodle::{error::MoodleError, models::SiteInfo},
};

pub mod error;
pub mod models;

pub struct MoodleService {
    app_handle: AppHandle,
    http: reqwest::Client,
}

impl MoodleService {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        }
    }

    pub async fn call<T: DeserializeOwned>(
        &self,
        wsfunction: &str,
        params: Option<HashMap<&str, &str>>,
    ) -> Result<T, MoodleError> {
        let token = match LoginManager::instance(&self.app_handle).await.get_state() {
            LoginState::Logged { token } => Ok(token),
            _ => Err(MoodleError::Unauthenticated),
        }?;

        let mut form = HashMap::new();
        form.insert("wstoken", token.as_str());
        form.insert("wsfunction", wsfunction);
        form.insert("moodlewsrestformat", "json");
        form.insert("moodlewssettingfilter", "true");
        form.insert("moodlewssettinglang", "it");

        if let Some(extra) = params {
            for (k, v) in extra {
                form.insert(k, v);
            }
        }

        let response = self
            .http
            .post("https://webeep.polimi.it/webservice/rest/server.php")
            .form(&form)
            .send()
            .await?;

        let bytes = response.bytes().await?;

        // Check for Moodle API exception format
        if let Ok(err_val) = serde_json::from_slice::<serde_json::Value>(&bytes) {
            if let Some(err_code) = err_val.get("errorcode").and_then(|v| v.as_str()) {
                if err_code == "invalidtoken" {
                    return Err(MoodleError::InvalidToken);
                }
                let msg = err_val["message"].as_str().unwrap_or("Unknown error");
                return Err(MoodleError::Moodle(msg.to_string()));
            }
        }

        Ok(serde_json::from_slice(&bytes)?)
    }

    pub async fn get_site_info(&self) -> Result<SiteInfo, MoodleError> {
        self.call::<SiteInfo>("core_webservice_get_site_info", None)
            .await
    }
}

impl MoodleService {
    pub fn instance(app_handle: &AppHandle) -> Arc<Self> {
        app_handle.state::<AppState>().moodle_service.clone()
    }
}
