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
    println!("[RUST] [VIDEO_PLAYER] Stream URL: {}", stream_url);
    println!("[RUST] [VIDEO_PLAYER] Stream type: {}", if stream_url.starts_with("magnet:") { "Magnet Link" } else { "Direct URL" });
    println!("[RUST] [VIDEO_PLAYER] URL length: {} characters", stream_url.len());

    let shell = app.shell();

    // Check if it's a magnet link
    if stream_url.starts_with("magnet:") {
        println!("[RUST] [VIDEO_PLAYER] ================================================");
        println!("[RUST] [VIDEO_PLAYER] Magnet link detected - starting Peerflix streaming");
        println!("[RUST] [VIDEO_PLAYER] ================================================");

        // Start Peerflix streaming
        let streamer = state.streamer.lock().await;
        let local_url = streamer.start_stream(stream_url).await?;

        println!("[RUST] [VIDEO_PLAYER] 🎯 Peerflix stream ready at: {}", local_url);
        println!("[RUST] [VIDEO_PLAYER] ⏳ Waiting additional time for buffering...");

        // Wait longer for better buffering before launching player
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        println!("[RUST] [VIDEO_PLAYER] 🚀 Launching video player...");

        // Try external video players first (better streaming experience)
        let players = vec![
            ("mpv", vec!["--keep-open=yes", "--cache=yes", "--demuxer-readahead-secs=20"]),
            ("vlc", vec!["--intf", "dummy"]),
        ];

        for (player, args) in players {
            println!("[RUST] [VIDEO_PLAYER] 🎬 Trying {}", player);

            let mut cmd = shell.command(player);
            for arg in args {
                cmd = cmd.arg(arg);
            }
            cmd = cmd.arg(&local_url);

            match cmd.spawn() {
                Ok((_, child)) => {
                    let success_msg = format!("✅ Successfully launched {} with Peerflix stream (PID: {:?})", player, child.pid());
                    println!("[RUST] [VIDEO_PLAYER] {}", success_msg);
                    println!("[RUST] [VIDEO_PLAYER] 📺 Video should start playing shortly");
                    println!("[RUST] [VIDEO_PLAYER] 🔄 Peerflix will continue downloading in background");
                    return Ok(success_msg);
                }
                Err(e) => {
                    println!("[RUST] [VIDEO_PLAYER] ❌ {} failed: {:?}", player, e);
                    continue;
                }
            }
        }

        // Fallback to browser if external players fail
        println!("[RUST] [VIDEO_PLAYER] 🌐 External players failed, trying browser...");

        let browsers = vec![
            "start",                                    // Windows default browser
            "cmd /c start \"\"",                        // Windows alternative
            "xdg-open",                                 // Linux default
            "open",                                     // macOS default
        ];

        for browser in browsers {
            println!("[RUST] [VIDEO_PLAYER] 🌐 Trying browser: {}", browser);

            let result = if browser.contains("cmd /c start") {
                shell.command("cmd")
                    .arg("/c")
                    .arg("start")
                    .arg("")
                    .arg(&local_url)
                    .spawn()
            } else if browser == "start" {
                shell.command("start")
                    .arg(&local_url)
                    .spawn()
            } else {
                shell.command(browser)
                    .arg(&local_url)
                    .spawn()
            };

            match result {
                Ok((_, _)) => {
                    let success_msg = format!("✅ Successfully opened Peerflix stream in browser using: {}", browser);
                    println!("[RUST] [VIDEO_PLAYER] {}", success_msg);
                    println!("[RUST] [VIDEO_PLAYER] 🌐 Stream should open in your default browser");
                    return Ok(success_msg);
                }
                Err(e) => {
                    println!("[RUST] [VIDEO_PLAYER] ❌ Browser {} failed: {:?}", browser, e);
                    continue;
                }
            }
        }

        return Err(format!("Could not launch video player or browser. Please manually open: {}", local_url));
    }

    // Direct HTTP URL - use existing code
    println!("[RUST] [VIDEO_PLAYER] Processing direct URL stream...");
    
    // Try different video players in order of preference
    // Prioritizing VLC with Windows-specific paths
    let players = vec![
        "vlc",                                          // System VLC (first choice)
        "C:\\Program Files\\VideoLAN\\VLC\\vlc.exe",    // Windows VLC default install
        "C:\\Program Files (x86)\\VideoLAN\\VLC\\vlc.exe", // Windows VLC 32-bit install
        "mpv",                                          // System MPV
        "flatpak run org.videolan.VLC",                // Steam Deck Flatpak VLC
        "flatpak run io.mpv.Mpv",                      // Steam Deck Flatpak MPV
        "/usr/bin/vlc",                                 // Explicit path for Linux VLC
        "/usr/bin/mpv",                                 // Explicit path for Linux MPV
        "xdg-open"                                      // Fallback for Linux
    ];

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