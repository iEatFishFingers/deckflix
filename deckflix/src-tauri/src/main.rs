// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;
mod addon_client;
mod torrent_streamer;

use addon_client::AddonClient;
use torrent_streamer::TorrentStreamer;
use models::{Movie, Series, Anime, Stream, SearchResult};
use tauri::{State, Manager};
use tauri_plugin_shell::ShellExt;
use tokio::sync::Mutex;
use std::sync::Arc;

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
    stream_url: String
) -> Result<String, String> {
    println!("[RUST] [VIDEO_PLAYER] ==============================================");
    println!("[RUST] [VIDEO_PLAYER] Starting video playback process");
    println!("[RUST] [VIDEO_PLAYER] Stream URL received from JavaScript: {}", stream_url);
    println!("[RUST] [VIDEO_PLAYER] Stream type: {}", if stream_url.starts_with("magnet:") { "Magnet Link" } else { "Direct URL" });
    println!("[RUST] [VIDEO_PLAYER] URL length: {} characters", stream_url.len());
    println!("[RUST] [VIDEO_PLAYER] First 100 chars of URL: {}", &stream_url.chars().take(100).collect::<String>());

    let shell = app.shell();

    // Check if it's a magnet link
    if stream_url.starts_with("magnet:") {
        println!("[RUST] [VIDEO_PLAYER] ================================================");
        println!("[RUST] [VIDEO_PLAYER] Magnet link detected - starting Peerflix streaming");
        println!("[RUST] [VIDEO_PLAYER] ================================================");

        // Start Peerflix streaming
        let streamer = state.streamer.lock().await;
        let torrent_dir = streamer.start_stream(stream_url).await?;

        println!("[RUST] [VIDEO_PLAYER] 🎯 Torrent directory: {}", torrent_dir);
        println!("[RUST] [VIDEO_PLAYER] 🔍 Waiting for video file to appear in directory...");

        // Find the largest video file in the torrent directory
        use std::fs;
        use std::path::Path;
        use std::time::Duration;

        let path = Path::new(&torrent_dir);

        // Wait for directory to be created (max 30 seconds)
        let mut attempts = 0;
        let max_attempts = 60; // 60 attempts * 500ms = 30 seconds
        while !path.exists() && attempts < max_attempts {
            println!("[RUST] [VIDEO_PLAYER] ⏳ Waiting for torrent directory to be created... (attempt {}/{})", attempts + 1, max_attempts);
            tokio::time::sleep(Duration::from_millis(500)).await;
            attempts += 1;
        }

        if !path.exists() {
            return Err(format!("Torrent directory was not created after {} seconds: {}", max_attempts / 2, torrent_dir));
        }

        println!("[RUST] [VIDEO_PLAYER] ✅ Torrent directory exists");

        let video_extensions = vec!["mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v"];
        let mut video_files: Vec<(String, u64)> = Vec::new();

        // Recursively search for video files
        fn find_videos(dir: &Path, extensions: &[&str], files: &mut Vec<(String, u64)>) -> std::io::Result<()> {
            if dir.is_dir() {
                for entry in fs::read_dir(dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() {
                        find_videos(&path, extensions, files)?;
                    } else if let Some(ext) = path.extension() {
                        if extensions.contains(&ext.to_str().unwrap_or("").to_lowercase().as_str()) {
                            if let Ok(metadata) = fs::metadata(&path) {
                                files.push((path.to_string_lossy().to_string(), metadata.len()));
                                println!("[RUST] [VIDEO_PLAYER] Found video: {} ({} bytes)", path.display(), metadata.len());
                            }
                        }
                    }
                }
            }
            Ok(())
        }

        // Wait for video files to appear (max 60 seconds)
        let mut video_attempts = 0;
        let max_video_attempts = 120; // 120 attempts * 500ms = 60 seconds

        while video_files.is_empty() && video_attempts < max_video_attempts {
            video_files.clear();
            if let Err(e) = find_videos(path, &video_extensions, &mut video_files) {
                println!("[RUST] [VIDEO_PLAYER] ⚠️  Error searching for videos: {}", e);
            }

            if video_files.is_empty() {
                println!("[RUST] [VIDEO_PLAYER] ⏳ Waiting for video files to appear... (attempt {}/{}, found {} files)",
                    video_attempts + 1, max_video_attempts, video_files.len());
                tokio::time::sleep(Duration::from_millis(500)).await;
                video_attempts += 1;
            }
        }

        if video_files.is_empty() {
            return Err(format!("No video files found in torrent directory after {} seconds: {}", max_video_attempts / 2, torrent_dir));
        }

        println!("[RUST] [VIDEO_PLAYER] ✅ Found {} video file(s)", video_files.len());

        // Wait for the video file to have some data downloaded (at least 10 MB)
        let min_file_size = 10 * 1024 * 1024; // 10 MB
        let mut size_attempts = 0;
        let max_size_attempts = 60; // 60 attempts * 500ms = 30 seconds

        // Sort by size (largest first) and take the largest one
        video_files.sort_by(|a, b| b.1.cmp(&a.1));
        let video_path = video_files[0].0.clone();

        println!("[RUST] [VIDEO_PLAYER] 📦 Selected video: {}", video_path);
        println!("[RUST] [VIDEO_PLAYER] ⏳ Waiting for video file to have sufficient data downloaded...");

        while size_attempts < max_size_attempts {
            if let Ok(metadata) = fs::metadata(&video_path) {
                let current_size = metadata.len();
                println!("[RUST] [VIDEO_PLAYER] 📊 Current file size: {:.2} MB", current_size as f64 / 1024.0 / 1024.0);

                if current_size >= min_file_size {
                    println!("[RUST] [VIDEO_PLAYER] ✅ Video file has sufficient data ({:.2} MB >= 10 MB)", current_size as f64 / 1024.0 / 1024.0);
                    break;
                }
            }

            tokio::time::sleep(Duration::from_millis(500)).await;
            size_attempts += 1;
        }

        println!("[RUST] [VIDEO_PLAYER] 🎬 Ready to launch MPV with video file");

        println!("[RUST] [VIDEO_PLAYER] 🎬 Selected video file: {}", video_path);
        println!("[RUST] [VIDEO_PLAYER] 📺 Launching MPV player with video file");

        // Launch MPV with the video file path - cross-platform support
        let mpv_players: Vec<&str> = if cfg!(target_os = "windows") {
            vec![
                "mpv",
                "C:\\Program Files\\mpv\\mpv.exe",
                "C:\\Program Files (x86)\\mpv\\mpv.exe",
            ]
        } else {
            vec![
                "mpv",                              // System PATH
                "flatpak run io.mpv.Mpv",          // Steam Deck Flatpak MPV
                "/usr/bin/mpv",                    // Linux standard location
                "/usr/local/bin/mpv",              // Alternative Linux location
                "/app/bin/mpv",                    // Flatpak location
            ]
        };

        for player in mpv_players.iter() {
            println!("[RUST] [VIDEO_PLAYER] Trying MPV at: {}", player);

            // Handle Flatpak commands specially
            let result = if player.contains("flatpak run") {
                let parts: Vec<&str> = player.split_whitespace().collect();
                if parts.len() >= 3 {
                    let flatpak_app = parts[2];
                    println!("[RUST] [VIDEO_PLAYER] Launching Flatpak app: {}", flatpak_app);
                    shell.command("flatpak")
                        .args(&["run", flatpak_app])
                        .arg(&video_path)
                        .spawn()
                } else {
                    continue;
                }
            } else {
                shell.command(player).arg(&video_path).spawn()
            };

            match result {
                Ok((mut rx, mut child)) => {
                    let success_msg = format!("Successfully launched MPV with video file (PID: {:?})", child.pid());
                    println!("[RUST] [VIDEO_PLAYER] ✅ {}", success_msg);
                    return Ok(success_msg);
                }
                Err(e) => {
                    println!("[RUST] [VIDEO_PLAYER] ❌ Failed to launch {}: {:?}", player, e);
                    continue;
                }
            }
        }

        return Err("MPV player not found. Please install MPV from mpv.io or via your package manager".to_string());
    }

    // Direct HTTP URL - cross-platform video player support
    println!("[RUST] [VIDEO_PLAYER] Processing direct URL stream...");

    // Try different video players in order of preference - cross-platform
    let players: Vec<&str> = if cfg!(target_os = "windows") {
        vec![
            "vlc",                                          // System VLC
            "C:\\Program Files\\VideoLAN\\VLC\\vlc.exe",    // Windows VLC 64-bit
            "C:\\Program Files (x86)\\VideoLAN\\VLC\\vlc.exe", // Windows VLC 32-bit
            "mpv",                                          // System MPV
            "C:\\Program Files\\mpv\\mpv.exe",             // Windows MPV
        ]
    } else {
        vec![
            "mpv",                                          // System MPV (first choice for Linux)
            "flatpak run io.mpv.Mpv",                      // Steam Deck Flatpak MPV
            "vlc",                                          // System VLC
            "flatpak run org.videolan.VLC",                // Steam Deck Flatpak VLC
            "/usr/bin/mpv",                                 // Explicit path for Linux MPV
            "/usr/bin/vlc",                                 // Explicit path for Linux VLC
            "/usr/local/bin/mpv",                          // Alternative MPV location
            "xdg-open"                                      // Fallback for Linux
        ]
    };

    println!("[RUST] [VIDEO_PLAYER] Available video players to try: {:?}", players);
    println!("[RUST] [VIDEO_PLAYER] Starting player detection and launch sequence...");
    
    for (index, player) in players.iter().enumerate() {
        println!("[RUST] [VIDEO_PLAYER] ----------------------------------------------");
        println!("[RUST] [VIDEO_PLAYER] Attempting player {}/{}: {}", index + 1, players.len(), player);

        let result = if player.contains("flatpak run") {
            // Handle Flatpak commands specially
            let parts: Vec<&str> = player.split_whitespace().collect();
            println!("[RUST] [VIDEO_PLAYER] Detected Flatpak command, parsing: {:?}", parts);

            if parts.len() >= 3 {
                let flatpak_app = parts[2];
                println!("[RUST] [VIDEO_PLAYER] Launching Flatpak app: {}", flatpak_app);
                println!("[RUST] [VIDEO_PLAYER] Command: flatpak run {} \"{}\"", flatpak_app, stream_url);

                shell.command("flatpak")
                    .args(&["run", flatpak_app])
                    .arg(&stream_url)
                    .spawn()
            } else {
                println!("[RUST] [VIDEO_PLAYER] Invalid Flatpak command format, skipping");
                continue;
            }
        } else {
            // Regular command
            println!("[RUST] [VIDEO_PLAYER] Regular command launch");
            println!("[RUST] [VIDEO_PLAYER] Command: {} \"{}\"", player, stream_url);
            shell.command(player).arg(&stream_url).spawn()
        };

        match result {
            Ok((mut rx, mut child)) => {
                let success_msg = format!("Successfully launched {} with stream (PID: {:?})", player, child.pid());
                println!("[RUST] [VIDEO_PLAYER] ✅ {}", success_msg);
                println!("[RUST] [VIDEO_PLAYER] Video player should now be opening...");
                println!("[RUST] [VIDEO_PLAYER] ==============================================");
                return Ok(success_msg);
            }
            Err(e) => {
                println!("[RUST] [VIDEO_PLAYER] ❌ Failed to launch {}: {:?}", player, e);
                println!("[RUST] [VIDEO_PLAYER] Error type: {}", e);
                continue; // Try next player
            }
        }
    }

    println!("[RUST] [VIDEO_PLAYER] ❌ ALL PLAYERS FAILED");
    println!("[RUST] [VIDEO_PLAYER] No video player could be launched successfully");
    println!("[RUST] [VIDEO_PLAYER] Tried {} different players", players.len());
    println!("[RUST] [VIDEO_PLAYER] ==============================================");

    Err("No video player found. Please install:\n• Windows: Download MPV from mpv.io or VLC from videolan.org\n• Steam Deck: Run 'sudo pacman -S mpv' or install via Discover app".to_string())
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