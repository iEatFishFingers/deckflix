// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;
mod addon_client;

use addon_client::AddonClient;
use models::{Movie, Series, Anime, Stream, SearchResult};
use tauri::{State, Manager};
use tauri_plugin_shell::ShellExt;
use tokio::sync::Mutex;

// Global state for the addon client
struct AppState {
    client: Mutex<AddonClient>,
}

#[tauri::command]
async fn fetch_popular_movies(state: State<'_, AppState>) -> Result<Vec<Movie>, String> {
    let client = state.client.lock().await;
    client.fetch_popular_movies().await
}

#[tauri::command]
async fn fetch_streams(imdb_id: String, state: State<'_, AppState>) -> Result<Vec<Stream>, String> {
    let client = state.client.lock().await;
    client.fetch_streams(&imdb_id).await
}

#[tauri::command]
async fn play_video_external(app: tauri::AppHandle, stream_url: String) -> Result<String, String> {
    // Use Tauri's shell plugin to open external programs
    let shell = app.shell();
    
    // Try different video players in order of preference
    let players = vec!["mpv", "vlc", "xdg-open"];
    
    for player in players {
        match shell.command(player).arg(&stream_url).spawn() {
            Ok(_) => {
                return Ok(format!("Launched {} with stream", player));
            }
            Err(_) => {
                continue; // Try next player
            }
        }
    }

    Err("No video player found. Please install mpv or vlc.".to_string())
}

#[tauri::command]
async fn fetch_popular_series(state: State<'_, AppState>) -> Result<Vec<Series>, String> {
    let client = state.client.lock().await;
    client.fetch_popular_series().await
}

#[tauri::command]
async fn fetch_popular_anime(state: State<'_, AppState>) -> Result<Vec<Anime>, String> {
    let client = state.client.lock().await;
    client.fetch_popular_anime().await
}

#[tauri::command]
async fn search_content(query: String, state: State<'_, AppState>) -> Result<Vec<SearchResult>, String> {
    let client = state.client.lock().await;
    client.search_content(&query).await
}

#[tauri::command]
async fn get_addon_status() -> Result<String, String> {
    Ok("Ready".to_string())
}

fn main() {
    // Initialize the addon client
    let client = AddonClient::new();
    let app_state = AppState {
        client: Mutex::new(client),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            fetch_popular_movies,
            fetch_popular_series,
            fetch_popular_anime,
            search_content,
            fetch_streams,
            play_video_external,
            get_addon_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}