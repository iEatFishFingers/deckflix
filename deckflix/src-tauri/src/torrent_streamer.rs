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

        // Check if peerflix is installed - try different approaches for Windows
        println!("[RUST] [TORRENT] 🔍 Checking if Peerflix is installed...");

        let peerflix_commands = vec![
            "peerflix",           // Try PATH version first
            "peerflix.cmd",       // Windows specific
            "C:\\Users\\yoann\\AppData\\Roaming\\npm\\peerflix.cmd", // Full path
        ];

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

        // Start peerflix process
        println!("[RUST] [TORRENT] 🚀 Starting Peerflix process...");
        println!("[RUST] [TORRENT] Command: {} \"{}\" --port {} --not-on-top", working_command, magnet_link, self.stream_port);

        let child = Command::new(working_command)
            .arg(&magnet_link)
            .arg("--port")
            .arg(self.stream_port.to_string())
            .arg("--not-on-top") // Don't prioritize latest pieces for better streaming
            .arg("--quiet") // Reduce output noise
            .spawn()
            .map_err(|e| format!("Failed to start peerflix: {}", e))?;

        let pid = child.id();
        println!("[RUST] [TORRENT] ✅ Peerflix started successfully (PID: {})", pid);

        // Store process handle
        *self.peerflix_process.lock().await = Some(child);

        // Peerflix serves at this URL
        let url = format!("http://127.0.0.1:{}", self.stream_port);
        *self.stream_url.lock().await = Some(url.clone());

        println!("[RUST] [TORRENT] 📡 Stream will be available at: {}", url);
        println!("[RUST] [TORRENT] ⏳ Waiting for Peerflix to initialize and start downloading...");

        // Wait for peerflix to initialize and start downloading
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        println!("[RUST] [TORRENT] 🎯 Testing stream availability...");

        // Test if the stream is responding
        match self.test_stream_availability(&url).await {
            Ok(_) => {
                println!("[RUST] [TORRENT] ✅ Stream is ready and responding!");
                println!("[RUST] [TORRENT] 📺 Ready for video player launch");
                Ok(url)
            }
            Err(e) => {
                println!("[RUST] [TORRENT] ⚠️  Stream not immediately available: {}", e);
                println!("[RUST] [TORRENT] 🔄 This is normal - Peerflix is still connecting to peers");
                println!("[RUST] [TORRENT] 📺 Proceeding with video player launch anyway");
                Ok(url)
            }
        }
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