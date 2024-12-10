// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use serde::{Serialize, Deserialize};
use std::fs;
use std::env;
use std::path::PathBuf;
use reqwest::{Client, StatusCode};
use log::{debug, error, LevelFilter};
use std::sync::Arc;
use tauri::{async_runtime::Mutex, Manager};
use tokio::sync::Mutex as TokioMutex;
use tokio_util::sync::CancellationToken;

#[cfg(debug_assertions)]
const LOG_LEVEL: LevelFilter = LevelFilter::Debug;

#[derive(Default)]
struct AppState {
    cancellation_token: Option<CancellationToken>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Header {
    key: String,
    value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Parameter {
    parameter_type: String,
    key: String,
    value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Api {
    id: String,
    name: String,
    method: String,
    url: String,
    headers: Vec<Header>,
    parameters: Vec<Parameter>,
    body: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Collection {
    id: String,
    name: String,
    apis: Vec<Api>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AppData {
    collections: Vec<Collection>,
    apis: Vec<Api>,
}

#[tauri::command]
fn read_data() -> Result<AppData, String> {
    match fs::read(get_file_read()) {
        Ok(content) => {
            let content_str = String::from_utf8(content).map_err(|e| e.to_string())?;
            serde_json::from_str(&content_str).map_err(|e| e.to_string())
        },
        Err(_) => Ok(AppData { collections: vec![], apis: vec![] }),
    }
}

#[tauri::command]
fn save_data(data: String) -> Result<(), String> {
    let app_data: AppData = serde_json::from_str(&data).map_err(|e| e.to_string())?;
    let content = serde_json::to_string_pretty(&app_data).map_err(|e| e.to_string())?;
    fs::write(get_file_read(), content).map_err(|e| format!("Failed to write data: {}", e))?;
    Ok(())
}

#[tauri::command]
fn save_api(data: String) -> Result<(), String> {
    let read_content = fs::read(get_file_read()).map_err(|e| e.to_string())?;
    let read_content_str = String::from_utf8(read_content).map_err(|e| e.to_string())?;
    let mut app_data: AppData = serde_json::from_str(&read_content_str).map_err(|e| e.to_string())?;

    let new_api: Api = serde_json::from_str(&data).map_err(|e| e.to_string())?;

    let mut updated = false;
    for collection in &mut app_data.collections {
        if let Some(existing_api) = collection.apis.iter_mut().find(|api| api.id == new_api.id) {
            *existing_api = new_api.clone();
            updated = true;
            break;
        }
    }
    
    if !updated {
        if let Some(existing_api) = app_data.apis.iter_mut().find(|api| api.id == new_api.id) {
            *existing_api = new_api.clone();
            updated = true;
        }
    }
    
    if !updated {
        return Err("API with the given ID not found.".to_string());
    }

    let updated_content = serde_json::to_string_pretty(&app_data).map_err(|e| e.to_string())?;
    fs::write(get_file_read(), updated_content).map_err(|e| format!("Failed to write data: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn send_request(
    state: tauri::State<'_, Arc<TokioMutex<AppState>>>,
    method: String,
    url: String,
    headers: Vec<Header>,
    body: Option<String>,
) -> Result<serde_json::Value, String> {
    let mut state = state.lock().await;
    
    // Cancel existing task
    if let Some(token) = &state.cancellation_token {
        token.cancel();
    }

    // Start new task
    let token = CancellationToken::new();
    state.cancellation_token = Some(token.clone());

    let token_clone = token.clone();
    let handle = tokio::spawn(async move {
        let client = Client::new();

        let mut req = match method.as_str() {
            "GET" => client.get(&url),
            "POST" => client.post(&url),
            "PUT" => client.put(&url),
            "PATCH" => client.patch(&url),
            "DELETE" => client.delete(&url),
            _ => return Err(format!("Unsupported method: {}", method)),
        };
    
        for header in headers {
            if !header.key.is_empty() && !header.value.is_empty() {
                req = req.header(header.key.clone(), header.value.clone());
            }
        }
    
        if let Some(b) = body {
            req = req.body(b);
        }
    
        let response = tokio::select! {
            res = req.send() => res.map_err(|e| format!("Request failed: {}", e))?,
            _ = token_clone.cancelled() => return Err("Request cancelled".to_string()),
        };

        let status = response.status();

        if status.is_success() {
            response.json::<serde_json::Value>().await.map_err(|e| format!("Failed to parse response: {}", e))
        } else {
            Err(format!("HTTP error: {}", status))
        }
    });


    let result = tokio::select! {
        res = handle => res.map_err(|e| e.to_string())?,
        _ = token.cancelled() => Err("Request cancelled".to_string()),
    };

    state.cancellation_token = None;
    result
}

#[tauri::command]
async fn cancel_request(state: tauri::State<'_, Arc<TokioMutex<AppState>>>) -> Result<(), String> {
    let mut state = state.lock().await;
    if let Some(token) = &state.cancellation_token {
        debug!("Cancelling request...");
        token.cancel();
        state.cancellation_token = None;
        debug!("Request cancelled");
        Ok(())
    } else {
        Err("No request to cancel".to_string())
    }
}

fn ensure_dist_dir_exists() {
    let app_dir = get_file_path();

    if !app_dir.exists() {
        fs::create_dir(&app_dir).expect("Failed to create dist directory");
    }

    if !get_file_read().exists() {
        let empty_data = AppData {
            collections: vec![],
            apis: vec![],
        };
    
        let serialized_data = serde_json::to_string_pretty(&empty_data)
            .expect("Failed to serialize default AppData");
    
        fs::write(get_file_read(), serialized_data)
            .expect("Failed to create api.json file");
    }
}

fn get_file_path() -> PathBuf {
    env::current_dir()
        .expect("Failed to get current directory")
        .join("dist")
}

fn get_file_read()-> PathBuf {
    PathBuf::from(get_file_path()).join("api.json")
}

fn main() {
    ensure_dist_dir_exists();
    tauri::Builder::default()
        .manage(Arc::new(TokioMutex::new(AppState::default())))
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_log::Builder::default().build()) 
        .invoke_handler(tauri::generate_handler![read_data, save_data, save_api, send_request, cancel_request]) 
        .run(tauri::generate_context!()) 
        .expect("error while running tauri application");
}
