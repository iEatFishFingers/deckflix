use std::process::{Child, Command};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct TorrentStreamer {
    peerflix_process: Arc<Mutex<Option<Child>>>,
    stream_url: Arc<Mutex<Option<String>>>,
    stream_port: u16,
}

impl TorrentStreamer {
    pub fn new() -> Self {
        Self {
            peerflix_process: Arc::new(Mutex::new(None)),
            stream_url: Arc::new(Mutex::new(None)),
            stream_port: 8888,
        }
    }

    pub async fn start_stream(&self, magnet_link: String) -> Result<String, String> {
        println!("[RUST] [TORRENT] ================================================");
        println!("[RUST] [TORRENT] Starting Peerflix torrent streaming");
        println!("[RUST] [TORRENT] Magnet: {}", magnet_link);
        println!("[RUST] [TORRENT] ================================================");

        // Stop any existing stream first
        self.stop_stream().await?;

        // Extract infohash from magnet link
        let infohash = if let Some(start) = magnet_link.find("btih:") {
            let hash_start = start + 5;
            let hash_end = magnet_link[hash_start..].find('&').map(|p| hash_start + p).unwrap_or(magnet_link.len());
            magnet_link[hash_start..hash_end].to_string()
        } else {
            return Err("Invalid magnet link: no infohash found".to_string());
        };

        println!("[RUST] [TORRENT] 🔑 Extracted infohash: {}", infohash);

        // Check if peerflix is installed - cross-platform approach
        println!("[RUST] [TORRENT] 🔍 Checking if Peerflix is installed...");

        let peerflix_commands = if cfg!(target_os = "windows") {
            vec![
                "peerflix",
                "peerflix.cmd",
                "C:\\Users\\yoann\\AppData\\Roaming\\npm\\peerflix.cmd",
            ]
        } else {
            vec![
                "peerflix",                           // System PATH
                "/usr/local/bin/peerflix",           // Common Linux location
                "/usr/bin/peerflix",                 // Alternative Linux location
                "/home/deck/.local/bin/peerflix",    // Steam Deck user install
            ]
        };

        let mut peerflix_found = false;
        let mut working_command = "peerflix";

        for cmd in &peerflix_commands {
            println!("[RUST] [TORRENT] 🔍 Trying command: {}", cmd);
            let check = Command::new(cmd)
                .arg("--help")
                .output();

            if check.is_ok() {
                println!("[RUST] [TORRENT] ✅ Peerflix found with command: {}", cmd);
                peerflix_found = true;
                working_command = cmd;
                break;
            } else {
                println!("[RUST] [TORRENT] ❌ Command {} failed: {:?}", cmd, check.err());
            }
        }

        if !peerflix_found {
            println!("[RUST] [TORRENT] ❌ Peerflix not found with any command!");
            return Err("Peerflix not installed. Install with: npm install -g peerflix".to_string());
        }

        println!("[RUST] [TORRENT] ✅ Peerflix found");

        // Extract file index from magnet link if present (&so= parameter)
        let file_index = if let Some(so_pos) = magnet_link.find("&so=") {
            let start = so_pos + 4; // Skip "&so="
            let end = magnet_link[start..].find('&').map(|p| start + p).unwrap_or(magnet_link.len());
            magnet_link[start..end].parse::<usize>().ok()
        } else {
            None
        };

        // Start peerflix process
        println!("[RUST] [TORRENT] 🚀 Starting Peerflix process...");
        if let Some(idx) = file_index {
            println!("[RUST] [TORRENT] 📂 File index detected: {} (will use Peerflix --select)", idx);
            println!("[RUST] [TORRENT] Command: {} \"{}\" --port {} --select {} --not-on-top", working_command, magnet_link, self.stream_port, idx);
        } else {
            println!("[RUST] [TORRENT] Command: {} \"{}\" --port {} --not-on-top", working_command, magnet_link, self.stream_port);
        }

        let mut command = Command::new(working_command);
        command
            .arg(&magnet_link)
            .arg("--port")
            .arg(self.stream_port.to_string())
            .arg("--not-on-top"); // Don't prioritize latest pieces for better streaming

        // Add file selection if we have a file index
        if let Some(idx) = file_index {
            command.arg("--select").arg(idx.to_string());
        }

        command.arg("--quiet"); // Reduce output noise

        let child = command
            .spawn()
            .map_err(|e| format!("Failed to start peerflix: {}", e))?;

        let pid = child.id();
        println!("[RUST] [TORRENT] ✅ Peerflix started successfully (PID: {})", pid);

        // Store process handle
        *self.peerflix_process.lock().await = Some(child);

        // Peerflix downloads to different locations based on OS
        let torrent_dir = if cfg!(target_os = "windows") {
            format!("C:\\tmp\\torrent-stream\\{}", infohash)
        } else {
            // Linux/Steam Deck - peerflix uses /tmp by default
            format!("/tmp/torrent-stream/{}", infohash)
        };

        println!("[RUST] [TORRENT] 📂 Torrent directory: {}", torrent_dir);
        println!("[RUST] [TORRENT] ⏳ Waiting for Peerflix to create torrent directory...");

        // Wait for peerflix to initialize and create the directory
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        // Return the torrent directory path - we'll find the video file later
        *self.stream_url.lock().await = Some(torrent_dir.clone());

        println!("[RUST] [TORRENT] 📺 Ready for video player launch");
        Ok(torrent_dir)
    }

    async fn test_stream_availability(&self, url: &str) -> Result<(), String> {
        // Try to make a HEAD request to test if the stream is responding
        let client = reqwest::Client::new();
        let response = client
            .head(url)
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await
            .map_err(|e| format!("Stream test failed: {}", e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Stream returned status: {}", response.status()))
        }
    }

    pub async fn stop_stream(&self) -> Result<(), String> {
        println!("[RUST] [TORRENT] 🛑 Stopping torrent stream...");

        let mut process = self.peerflix_process.lock().await;
        if let Some(mut child) = process.take() {
            println!("[RUST] [TORRENT] 🔄 Terminating Peerflix process (PID: {})", child.id());

            match child.kill() {
                Ok(_) => {
                    println!("[RUST] [TORRENT] ✅ Peerflix process terminated successfully");

                    // Wait for process to fully exit
                    match child.wait() {
                        Ok(status) => println!("[RUST] [TORRENT] 📋 Process exit status: {}", status),
                        Err(e) => println!("[RUST] [TORRENT] ⚠️  Process wait error: {}", e),
                    }
                }
                Err(e) => {
                    println!("[RUST] [TORRENT] ❌ Failed to kill Peerflix process: {}", e);
                    return Err(format!("Failed to kill peerflix: {}", e));
                }
            }
        } else {
            println!("[RUST] [TORRENT] 📋 No active Peerflix process to stop");
        }

        *self.stream_url.lock().await = None;
        println!("[RUST] [TORRENT] ✅ Stream cleanup completed");
        Ok(())
    }

    pub async fn get_stream_url(&self) -> Option<String> {
        self.stream_url.lock().await.clone()
    }

    pub async fn is_streaming(&self) -> bool {
        let process = self.peerflix_process.lock().await;
        process.is_some()
    }
}

impl Drop for TorrentStreamer {
    fn drop(&mut self) {
        // Ensure cleanup happens when the struct is dropped
        println!("[RUST] [TORRENT] 🧹 TorrentStreamer dropping - cleaning up resources");
    }
}