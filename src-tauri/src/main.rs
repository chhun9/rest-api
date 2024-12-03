// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use serde::{Serialize, Deserialize};
use std::fs;
use std::env;
use std::path::PathBuf;
use log::{debug, error, info, LevelFilter};

#[cfg(debug_assertions)]
const LOG_LEVEL: LevelFilter = LevelFilter::Debug;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Header {
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
    debug!("{}", data);
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
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_log::Builder::default().build()) 
        .invoke_handler(tauri::generate_handler![read_data, save_data, save_api]) 
        .run(tauri::generate_context!()) 
        .expect("error while running tauri application");
}
