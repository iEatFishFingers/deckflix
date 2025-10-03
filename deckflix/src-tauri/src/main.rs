// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;
mod addon_client;
mod torrent_streamer;

use addon_client::AddonClient;
use torrent_streamer::TorrentStreamer;
use models::{Movie, Series, Anime, Stream, SearchResult, VideoMetadata};
use tauri::{State, Manager};
use tauri_plugin_shell::ShellExt;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::path::PathBuf;

// Global state for the addon client and torrent streamer
struct AppState {
    client: Mutex<AddonClient>,
    streamer: Arc<Mutex<TorrentStreamer>>,
}

#[tauri::command]
async fn fetch_popular_movies(state: State<'_, AppState>) -> Result<Vec<Movie>, String> {
    let client = state.client.lock().await;
    client.fetch_popular_movies().await
}

#[tauri::command]
async fn fetch_streams(imdb_id: String, state: State<'_, AppState>) -> Result<Vec<Stream>, String> {
    println!("╔════════════════════════════════════════════════════════════════════");
    println!("║ [RUST] [FETCH_STREAMS_COMMAND] Tauri command called from JavaScript");
    println!("║ [RUST] [FETCH_STREAMS_COMMAND] Received IMDB ID: {}", imdb_id);
    println!("║ [RUST] [FETCH_STREAMS_COMMAND] ID Length: {}", imdb_id.len());
    println!("╚════════════════════════════════════════════════════════════════════");

    let client = state.client.lock().await;
    client.fetch_streams(&imdb_id).await
}

#[tauri::command]
async fn play_video_external(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    stream_url: String,
    metadata: Option<VideoMetadata>
) -> Result<String, String> {
    println!("[RUST] [VIDEO_PLAYER] ==============================================");
    println!("[RUST] [VIDEO_PLAYER] Starting video playback process");
    println!("[RUST] [VIDEO_PLAYER] Stream URL: {}", stream_url);
    if let Some(ref meta) = metadata {
        println!("[RUST] [VIDEO_PLAYER] Title: {}", meta.title);
        println!("[RUST] [VIDEO_PLAYER] Year: {:?}", meta.year);
        println!("[RUST] [VIDEO_PLAYER] Type: {}", meta.content_type);
    }
    println!("[RUST] [VIDEO_PLAYER] ==============================================");

    let shell = app.shell();

    // Check if it's a magnet link
    if stream_url.starts_with("magnet:") {
        println!("[RUST] [VIDEO_PLAYER] Magnet link detected");

        // Extract torrent hash from magnet link
        let hash = extract_torrent_hash(&stream_url)?;
        println!("[RUST] [VIDEO_PLAYER] Extracted hash: {}", hash);

        // Start Peerflix streaming
        let streamer = state.streamer.lock().await;
        let local_url = streamer.start_stream(stream_url.clone()).await?;

        println!("[RUST] [VIDEO_PLAYER] Peerflix stream ready at: {}", local_url);

        println!("[RUST] [VIDEO_PLAYER] Launching external player with peerflix stream");

        // Launch external player with the peerflix stream URL
        // External players (MPV/VLC) can handle streaming torrents much better than built-in player
        return launch_external_player(&shell, &local_url, metadata).await;
    }

    // Direct HTTP URL - launch external player
    println!("[RUST] [VIDEO_PLAYER] Processing direct URL stream...");
    launch_external_player(&shell, &stream_url, metadata).await
}

// Helper function to extract torrent hash from magnet link
fn extract_torrent_hash(magnet_link: &str) -> Result<String, String> {
    // Format: magnet:?xt=urn:btih:HASH&...
    if let Some(start) = magnet_link.find("btih:") {
        let hash_start = start + 5;
        let hash_end = magnet_link[hash_start..]
            .find('&')
            .map(|pos| hash_start + pos)
            .unwrap_or(magnet_link.len());

        let hash = magnet_link[hash_start..hash_end].to_lowercase();

        if hash.len() == 40 || hash.len() == 32 {
            Ok(hash)
        } else {
            Err(format!("Invalid hash length: {}", hash.len()))
        }
    } else {
        Err("Could not find btih: in magnet link".to_string())
    }
}

// Helper function to find video file in torrent hash directory
async fn find_video_file_in_hash_dir(hash: &str) -> Result<PathBuf, String> {
    // Determine base torrent directory based on platform
    let base_dir = if cfg!(target_os = "windows") {
        PathBuf::from("C:/tmp/torrent-stream")
    } else {
        PathBuf::from("/tmp/torrent-stream")
    };

    let hash_dir = base_dir.join(hash);

    println!("[RUST] [VIDEO_FINDER] Looking in directory: {}", hash_dir.display());

    if !hash_dir.exists() {
        return Err(format!("Hash directory does not exist: {}", hash_dir.display()));
    }

    // Video file extensions to look for
    let video_extensions = vec!["mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v"];

    // Search for video files in the hash directory
    let entries = std::fs::read_dir(&hash_dir)
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if let Some(ext_str) = ext.to_str() {
                        if video_extensions.contains(&ext_str.to_lowercase().as_str()) {
                            println!("[RUST] [VIDEO_FINDER] Found video file: {}", path.display());
                            return Ok(path);
                        }
                    }
                }
            }
        }
    }

    Err(format!("No video files found in {}", hash_dir.display()))
}

// Helper function to launch external player with proper arguments
async fn launch_external_player(
    shell: &tauri_plugin_shell::Shell<tauri::Wry>,
    video_path: &str,
    metadata: Option<VideoMetadata>
) -> Result<String, String> {
    // Create media title from metadata
    let media_title = if let Some(ref meta) = metadata {
        if let Some(ref year) = meta.year {
            format!("{} ({})", meta.title, year)
        } else {
            meta.title.clone()
        }
    } else {
        "Unknown Movie - Torrent Stream (Peerflix)".to_string()
    };

    println!("[RUST] [VIDEO_PLAYER] Media title: {}", media_title);

    // Try different video players in order of preference
    // MPV prioritized for Steam Deck (lightweight, better for handheld)
    // Note: MPV requires --flag=value format, VLC accepts --flag value format
    let mpv_title_arg = format!("--force-media-title={}", media_title);
    let vlc_title_arg = format!("--meta-title={}", media_title);

    let players: Vec<(&str, Vec<String>)> = vec![
        ("mpv", vec![mpv_title_arg.clone(), "--fullscreen".to_string(), video_path.to_string()]),
        ("C:\\Program Files\\mpv\\mpv.exe", vec![mpv_title_arg.clone(), "--fullscreen".to_string(), video_path.to_string()]),
        ("vlc", vec![vlc_title_arg.clone(), "--fullscreen".to_string(), "--play-and-exit".to_string(), video_path.to_string()]),
        ("C:\\Program Files\\VideoLAN\\VLC\\vlc.exe", vec![vlc_title_arg.clone(), "--fullscreen".to_string(), "--play-and-exit".to_string(), video_path.to_string()]),
        ("C:\\Program Files (x86)\\VideoLAN\\VLC\\vlc.exe", vec![vlc_title_arg.clone(), "--fullscreen".to_string(), "--play-and-exit".to_string(), video_path.to_string()]),
        ("flatpak run io.mpv.Mpv", vec![mpv_title_arg.clone(), "--fullscreen".to_string(), video_path.to_string()]),
        ("flatpak run org.videolan.VLC", vec![vlc_title_arg.clone(), "--fullscreen".to_string(), "--play-and-exit".to_string(), video_path.to_string()]),
        ("/usr/bin/mpv", vec![mpv_title_arg.clone(), "--fullscreen".to_string(), video_path.to_string()]),
        ("/usr/bin/vlc", vec![vlc_title_arg, "--fullscreen".to_string(), "--play-and-exit".to_string(), video_path.to_string()]),
    ];

    println!("[RUST] [VIDEO_PLAYER] Trying {} different players...", players.len());

    for (index, (player, args)) in players.iter().enumerate() {
        println!("[RUST] [VIDEO_PLAYER] [{}/{}] Trying: {}", index + 1, players.len(), player);

        let result = if player.contains("flatpak run") {
            // Handle Flatpak commands specially
            let parts: Vec<&str> = player.split_whitespace().collect();
            if parts.len() >= 3 {
                let flatpak_app = parts[2];
                shell.command("flatpak")
                    .args(&["run", flatpak_app])
                    .args(args)
                    .spawn()
            } else {
                continue;
            }
        } else {
            // Regular command
            shell.command(player)
                .args(args)
                .spawn()
        };

        match result {
            Ok((mut rx, mut child)) => {
                let success_msg = format!("✅ Launched {} (PID: {:?})", player, child.pid());
                println!("[RUST] [VIDEO_PLAYER] {}", success_msg);
                return Ok(success_msg);
            }
            Err(e) => {
                println!("[RUST] [VIDEO_PLAYER] ❌ Failed: {:?}", e);
                continue;
            }
        }
    }

    println!("[RUST] [VIDEO_PLAYER] ❌ ALL PLAYERS FAILED");
    Err("No video player found. Please install MPV or VLC:\n• Windows: mpv.io or videolan.org\n• Steam Deck: 'sudo pacman -S mpv' or Discover app\n• Linux: 'sudo apt install mpv' or 'sudo dnf install mpv'".to_string())
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

#[tauri::command]
async fn stop_video_stream(state: State<'_, AppState>) -> Result<(), String> {
    let streamer = state.streamer.lock().await;
    streamer.stop_stream().await
}

fn main() {
    // Initialize the addon client and torrent streamer
    let client = AddonClient::new();
    let streamer = TorrentStreamer::new();
    let app_state = AppState {
        client: Mutex::new(client),
        streamer: Arc::new(Mutex::new(streamer)),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            app.manage(app_state);
            Ok(())
        })
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::Destroyed => {
                    println!("[RUST] [CLEANUP] Application window destroyed - cleaning up Peerflix processes");

                    // Kill any remaining peerflix processes
                    #[cfg(target_os = "windows")]
                    {
                        let _ = std::process::Command::new("taskkill")
                            .args(&["/f", "/im", "peerflix.cmd"])
                            .output();
                        let _ = std::process::Command::new("taskkill")
                            .args(&["/f", "/im", "node.exe"])
                            .arg("/fi")
                            .arg("WINDOWTITLE eq peerflix*")
                            .output();
                    }

                    #[cfg(not(target_os = "windows"))]
                    {
                        let _ = std::process::Command::new("pkill")
                            .arg("peerflix")
                            .output();
                    }

                    println!("[RUST] [CLEANUP] Peerflix cleanup completed");
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            fetch_popular_movies,
            fetch_popular_series,
            fetch_popular_anime,
            search_content,
            fetch_streams,
            play_video_external,
            stop_video_stream,
            get_addon_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}