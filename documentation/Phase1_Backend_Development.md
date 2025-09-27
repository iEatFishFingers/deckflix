# Phase 1: Backend API Integration - Development Documentation

## Overview
This phase focused on creating the Rust backend that integrates with Stremio addon APIs to fetch movie data and stream links. The backend provides Tauri commands that the frontend can call to interact with external streaming services.

## What Was Implemented

### 1. Rust Dependencies Setup (`src-tauri/Cargo.toml`)
```toml
[dependencies]
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
tauri = { version = "2.0", features = [] }
tauri-plugin-shell = "2.0"
```

**Purpose**:
- `reqwest`: HTTP client for making API calls to Stremio addons
- `serde`: JSON serialization/deserialization for API responses
- `tokio`: Async runtime for handling concurrent HTTP requests
- `tauri-plugin-shell`: For launching external video players

### 2. Data Models (`src-tauri/src/models.rs`)

**Created comprehensive data structures:**

```rust
// Main movie structure matching Stremio API format
pub struct Movie {
    pub id: String,           // IMDB ID
    pub name: String,         // Movie title
    pub poster: Option<String>, // Poster image URL
    pub background: Option<String>, // Background image
    pub description: Option<String>, // Plot summary
    pub year: Option<String>, // Release year
    pub imdb_rating: Option<String>, // IMDB rating
    pub genre: Option<Vec<String>>, // Genre list
}

// Stream source structure
pub struct Stream {
    pub name: Option<String>,    // Stream provider name
    pub title: String,          // Stream description
    pub url: String,           // Direct stream URL
    pub behavior_hints: Option<StreamBehaviorHints>,
}
```

**Design Decisions:**
- Used `Option<T>` for optional fields since Stremio API responses are inconsistent
- Implemented `Clone` and `Debug` traits for easier data manipulation
- Separated concerns with different structs for different API endpoints

### 3. Stremio API Client (`src-tauri/src/addon_client.rs`)

**HTTP Client Setup:**
```rust
pub struct AddonClient {
    client: Client,
    base_urls: Vec<String>, // Multiple addon sources for reliability
}

impl AddonClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        // Popular public Stremio addons
        let base_urls = vec![
            "https://torrentio.strem.fun".to_string(),
            "https://cinemeta-live.herokuapp.com".to_string(),
        ];

        Self { client, base_urls }
    }
}
```

**API Integration:**
- **Movies Endpoint**: `{addon_url}/catalog/movie/popular.json`
- **Streams Endpoint**: `{addon_url}/stream/movie/{imdb_id}.json`

**Error Handling Strategy:**
```rust
// Graceful fallback - if one addon fails, try the next
for base_url in &self.base_urls {
    match self.fetch_movies_from_addon(base_url, "popular").await {
        Ok(mut movies) => all_movies.append(&mut movies),
        Err(e) => {
            eprintln!("Failed to fetch from {}: {}", base_url, e);
            continue; // Try next addon
        }
    }
}
```

### 4. Tauri Commands (`src-tauri/src/main.rs`)

**Global State Management:**
```rust
struct AppState {
    client: Mutex<AddonClient>, // Thread-safe HTTP client
}
```

**Command Implementations:**
```rust
#[tauri::command]
async fn fetch_popular_movies(state: State<'_, AppState>) -> Result<Vec<Movie>, String>

#[tauri::command]
async fn fetch_streams(imdb_id: String, state: State<'_, AppState>) -> Result<Vec<Stream>, String>

#[tauri::command]
async fn play_video_external(app: tauri::AppHandle, stream_url: String) -> Result<String, String>

#[tauri::command]
async fn get_addon_status() -> Result<String, String>
```

## Challenges Encountered & Solutions

### Challenge 1: Inconsistent API Response Formats
**Problem**: Different Stremio addons return different JSON structures and some fields are sometimes missing.

**Solution**:
- Used `Option<T>` extensively in data models
- Implemented robust JSON parsing with fallbacks:
```rust
let year = meta
    .get("year")
    .and_then(|v| v.as_str())
    .map(|s| s.to_string()); // Returns None if field missing
```

### Challenge 2: Multiple Addon Sources
**Problem**: Single addon might be down or have limited content.

**Solution**:
- Implemented multi-source fetching with graceful fallback
- Deduplication logic to remove duplicate movies from different sources:
```rust
all_movies.sort_by(|a, b| a.id.cmp(&b.id));
all_movies.dedup_by(|a, b| a.id == b.id);
```

### Challenge 3: Video Player Integration
**Problem**: Need to launch external video players (mpv, vlc) from Tauri app.

**Solution**:
- Used `tauri-plugin-shell` for safe process spawning
- Implemented fallback chain for different video players:
```rust
let players = vec!["mpv", "vlc", "xdg-open"];
for player in players {
    match shell.command(player).arg(&stream_url).spawn() {
        Ok(_) => return Ok(format!("Launched {} with stream", player)),
        Err(_) => continue, // Try next player
    }
}
```

### Challenge 4: Async State Management
**Problem**: Tauri commands need shared access to HTTP client across async boundaries.

**Solution**:
- Used `tokio::sync::Mutex` for thread-safe state sharing
- Proper async/await handling in all Tauri commands

### Challenge 5: CORS and Network Issues
**Problem**: Browser security restrictions when making cross-origin requests.

**Solution**:
- Made HTTP requests from Rust backend instead of JavaScript frontend
- Rust bypasses browser CORS restrictions
- Added proper timeout handling for unreliable addon servers

## Key Learning Points

### 1. Stremio Addon Ecosystem
- Addons expose REST APIs with standardized endpoints
- Popular addons: Torrentio (torrents), Cinemeta (metadata)
- API format: `/catalog/{type}/{id}.json` and `/stream/{type}/{imdb_id}.json`

### 2. Rust Error Handling Patterns
```rust
// Convert various error types to String for Tauri
.map_err(|e| format!("Network error: {}", e))?
```

### 3. Async Rust Best Practices
- Use `tokio::sync::Mutex` for shared state in async contexts
- Prefer `Vec<T>` over channels for collecting results
- Handle network timeouts explicitly

### 4. Tauri State Management
```rust
// Global state injection
app.manage(app_state);

// Command parameter injection
async fn command(state: State<'_, AppState>) -> Result<T, String>
```

## API Endpoints Used

### Movie Catalog
```
GET https://cinemeta-live.herokuapp.com/catalog/movie/popular.json
```
**Response**: List of popular movies with metadata

### Stream Sources
```
GET https://torrentio.strem.fun/stream/movie/{imdb_id}.json
```
**Response**: Available stream links for specific movie

## File Structure
```
src-tauri/src/
├── main.rs          # Tauri app setup and commands
├── models.rs        # Data structures
├── addon_client.rs  # HTTP client and API logic
└── lib.rs          # Module declarations
```

## Testing Strategy
- Used `console.log()` in frontend to verify data flow
- Error messages propagated from Rust to JavaScript
- Fallback behavior tested by temporarily breaking addon URLs

## Performance Optimizations
- HTTP client reuse via global state
- Connection timeout to handle slow addons
- Concurrent requests to multiple addons
- Result caching could be added in future iterations

This phase successfully established a robust backend that can fetch movie data from multiple sources and handle network failures gracefully. The modular design makes it easy to add new addon sources or modify the API integration.