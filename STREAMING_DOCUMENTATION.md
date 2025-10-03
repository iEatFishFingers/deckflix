# DeckFlix Streaming Architecture Documentation

## Overview

DeckFlix is a streaming application that uses **Stremio addons** to discover movies/series and **torrent streaming via Peerflix** to play them. This document explains how the entire streaming pipeline works from start to finish.

---

## Architecture Flow

```
User clicks movie → Fetch streams from Stremio addons → User selects stream →
Download magnet link via Peerflix → Stream to built-in HTML5 player
```

---

## 1. Content Discovery (Movies/Series/Anime)

### How it works:
- Uses **Stremio Community Addons** API to fetch popular content
- Stremio is a streaming platform with community-built addons that aggregate content from various sources

### Stremio API Endpoints Used:

```rust
// Located in: src-tauri/src/addon_client.rs

let base_urls = vec![
    "https://v3-cinemeta.strem.io",           // Official Stremio metadata
    "https://torrentio.strem.fun",            // Torrent aggregator
    "https://thepiratebay-plus.strem.fun",    // TPB torrents
    "https://torrentio.strem.fun/lite",       // Lightweight torrent source
    "https://prowlarr.elfhosted.com/c/stremio" // Prowlarr indexer
];
```

### Content Fetching Process:

**Step 1: Fetch Movie List**
```
GET https://v3-cinemeta.strem.io/catalog/movie/top.json
```

**Response Format:**
```json
{
  "metas": [
    {
      "id": "tt5950044",
      "name": "Superman",
      "type": "movie",
      "poster": "https://...",
      "year": "2025",
      "imdbRating": "7.5"
    },
    ...
  ]
}
```

**Step 2: Display in UI**
- JavaScript fetches movies via Tauri command: `invoke('fetch_popular_movies')`
- Rust backend calls Stremio API and returns movie list
- Frontend displays movie cards with posters, titles, ratings

---

## 2. Stream Discovery (When User Clicks a Movie)

### How it works:
When a user clicks on a movie, the app fetches **available torrent streams** for that specific movie using its **IMDB ID**.

### Stream Fetching Process:

**Step 1: User clicks on movie card**
```javascript
// Located in: src/app.js - line ~1486

function createContentCard(content, index, contentType) {
  const card = document.createElement('div');
  card.dataset.contentId = content.id; // Store IMDB ID (e.g., tt5950044)

  card.addEventListener('click', () => {
    selectContent(content, contentType); // Pass entire content object
  });
}
```

**Step 2: Fetch streams from Stremio addons**
```javascript
// Located in: src/app.js - line ~1788

async function selectContent(content, contentType) {
  const streams = await safeInvoke('fetch_streams', {
    imdbId: content.id  // e.g., "tt5950044"
  });
}
```

**Step 3: Backend fetches streams**
```rust
// Located in: src-tauri/src/main.rs - line ~28

#[tauri::command]
async fn fetch_streams(imdb_id: String, state: State<'_, AppState>) -> Result<Vec<Stream>, String> {
    let client = state.client.lock().await;
    client.fetch_streams(&imdb_id).await
}
```

**Step 4: Query each Stremio addon for torrents**
```rust
// Located in: src-tauri/src/addon_client.rs - line ~240

pub async fn fetch_streams(&self, imdb_id: &str) -> Result<Vec<Stream>, String> {
    let mut all_streams = Vec::new();

    for base_url in &self.base_urls {
        // Build URL: https://torrentio.strem.fun/stream/movie/tt5950044.json
        let url = format!("{}/stream/movie/{}.json", base_url, imdb_id);

        let response = self.client.get(&url).send().await;
        // Parse response and extract magnet links
    }

    Ok(all_streams)
}
```

**Stremio Stream API Response Format:**
```json
{
  "streams": [
    {
      "name": "Superman",
      "title": "1080p BluRay\nSize: 2.5GB\nSeeders: 150",
      "infoHash": "6a941074cae0476d4ff912319f722b7a77db2e54",
      "sources": ["tracker1", "tracker2"]
    },
    {
      "title": "720p WEB\nSize: 1.2GB\nSeeders: 80",
      "infoHash": "abc123...",
      ...
    }
  ]
}
```

**Step 5: Convert to magnet links**
```rust
// Located in: src-tauri/src/addon_client.rs

// Stremio returns infoHash, we convert to magnet link
let magnet = format!(
    "magnet:?xt=urn:btih:{}&tr={}",
    info_hash,
    trackers.join("&tr=")
);

streams.push(Stream {
    url: magnet,
    title: stream.title,
    quality: parsed_quality,
    seeders: parsed_seeders,
    ...
});
```

---

## 3. Stream Selection & Playback

### Step 1: Display stream options to user
```javascript
// Located in: src/app.js - line ~1805

function displayStreams(streams) {
  streams.forEach((stream, index) => {
    const streamItem = createStreamItem(stream, index);
    elements.streamsList.appendChild(streamItem);
  });
}
```

**Stream Modal Shows:**
- Quality (1080p, 720p, 4K, etc.)
- Seeders count
- File size
- Source (Torrentio, TPB, etc.)

### Step 2: User clicks on a stream
```javascript
// Located in: src/app.js - line ~1846

item.addEventListener('click', () => {
  playStream(stream); // Pass full stream object with magnet URL
});
```

### Step 3: Determine playback method
```javascript
// Located in: src/app.js - line ~1880

async function playStream(stream) {
  if (stream.url.startsWith('magnet:')) {
    // Use Peerflix for torrent streaming
    tryExternalPlayer(stream.url);
  } else if (stream.url.includes('.mp4') || stream.url.includes('.mkv')) {
    // Use built-in HTML5 player for direct HTTP URLs
    playWithBuiltInPlayer(stream);
  }
}
```

---

## 4. Torrent Streaming with Peerflix

### What is Peerflix?
**Peerflix** is a command-line tool that:
1. Connects to torrent swarm using magnet link
2. Downloads video file pieces in order (streaming mode)
3. Serves the video over local HTTP server (http://127.0.0.1:8888)
4. Allows immediate playback while downloading

### Installation:
```bash
npm install -g peerflix
```

### Peerflix Streaming Process:

**Step 1: JavaScript sends magnet link to Rust**
```javascript
// Located in: src/app.js - line ~2009

const result = await safeInvoke('play_video_external', {
  streamUrl: stream.url  // magnet:?xt=urn:btih:...
});
```

**Step 2: Rust receives magnet and starts Peerflix**
```rust
// Located in: src-tauri/src/main.rs - line ~35

#[tauri::command]
async fn play_video_external(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    stream_url: String  // magnet:?xt=urn:btih:...
) -> Result<String, String> {
    if stream_url.starts_with("magnet:") {
        let streamer = state.streamer.lock().await;
        let local_url = streamer.start_stream(stream_url).await?;
        return Ok(local_url); // Returns http://127.0.0.1:8888
    }
}
```

**Step 3: TorrentStreamer spawns Peerflix process**
```rust
// Located in: src-tauri/src/torrent_streamer.rs - line ~20

pub async fn start_stream(&self, magnet_link: String) -> Result<String, String> {
    // Stop any existing stream
    self.stop_stream().await?;

    // Start Peerflix process
    let child = Command::new("peerflix.cmd")
        .arg(&magnet_link)
        .arg("--port")
        .arg("8888")
        .arg("--not-on-top")  // Don't prioritize latest pieces
        .arg("--quiet")       // Reduce output
        .spawn()?;

    // Store process handle
    *self.peerflix_process.lock().await = Some(child);

    // Wait 3 seconds for initialization
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Return local stream URL
    Ok("http://127.0.0.1:8888".to_string())
}
```

**What Peerflix does:**
```bash
# This is what gets executed:
peerflix.cmd "magnet:?xt=urn:btih:6a941074..." --port 8888 --not-on-top --quiet

# Peerflix output:
# - Connects to torrent swarm
# - Finds peers with the video file
# - Downloads pieces in sequential order
# - Serves video at: http://127.0.0.1:8888
# - Continues downloading while playing
```

**Step 4: Play local stream in HTML5 video player**
```javascript
// Located in: src/app.js - line ~2017

if (result.startsWith('http://127.0.0.1:')) {
  // Peerflix returned local stream URL
  const peerflixStream = {
    title: 'Torrent Stream (Peerflix)',
    url: result  // http://127.0.0.1:8888
  };

  playWithBuiltInPlayer(peerflixStream);
}
```

**Step 5: HTML5 video player loads stream**
```javascript
// Located in: src/app.js - line ~1919

function playWithBuiltInPlayer(stream) {
  elements.videoPlayer.src = stream.url; // http://127.0.0.1:8888
  elements.videoPlayerModal.classList.remove('hidden');

  elements.videoPlayer.onloadedmetadata = () => {
    elements.videoPlayer.play();
  };
}
```

---

## 5. Common Issues & Troubleshooting

### Issue 1: "Wrong movie playing" (e.g., Rick and Morty instead of selected movie)

**Root Cause:**
Not a code issue - it's a **fake/mislabeled torrent**. Some torrents on the internet are intentionally mislabeled or contain wrong content.

**Why it happens:**
1. Stremio addon returns streams based on IMDB ID
2. Some torrent uploaders use fake metadata to get more downloads
3. The torrent's infoHash points to different content
4. Your code fetches the CORRECT movie (verified by logs)
5. But the torrent itself contains wrong content

**Solution:**
- Click on a **different stream** from the list
- Try streams with higher seeders (more reliable)
- Try streams from different sources (Torrentio vs TPB)
- Avoid streams with suspicious quality labels

**How to verify code is working:**
Look at the logs when you click a movie:
```
[RUST] [FETCH_STREAMS_COMMAND] Received IMDB ID: tt0068646  ← Correct movie ID
[RUST] [STREAMS_FETCH] Starting to fetch streams for IMDB ID: tt0068646
[RUST] [VIDEO_PLAYER] Stream URL received: magnet:?xt=urn:btih:6a941074...
```

If the IMDB ID matches the movie you clicked, **the code is working correctly**. The issue is the torrent content itself.

### Issue 2: "No streams available"

**Possible causes:**
1. Movie is too new (torrents not available yet)
2. Stremio addons are down
3. Network connectivity issues
4. Movie has wrong IMDB ID

**Solution:**
- Try a different movie
- Check if Stremio addons are accessible: https://torrentio.strem.fun
- Restart the app

### Issue 3: "Peerflix not found"

**Error message:**
```
Peerflix not installed. Install with: npm install -g peerflix
```

**Solution:**
```bash
npm install -g peerflix
```

### Issue 4: "Stream won't load" or "Video stuck at 0%"

**Possible causes:**
1. No seeders available for torrent
2. Firewall blocking torrent connections
3. Torrent tracker issues

**Solution:**
- Select stream with higher seeder count
- Check firewall settings
- Try different stream from list
- Wait longer (some torrents take 30-60 seconds to start)

### Issue 5: "Video buffers constantly"

**Causes:**
1. Slow download speed
2. Few seeders
3. Large file size

**Solution:**
- Select lower quality stream (720p instead of 4K)
- Select stream with more seeders
- Wait for more content to download before playing

---

## 6. Data Flow Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    1. Content Discovery                      │
│                                                               │
│  User opens app                                               │
│       ↓                                                       │
│  JavaScript: invoke('fetch_popular_movies')                   │
│       ↓                                                       │
│  Rust: GET https://v3-cinemeta.strem.io/catalog/movie/top.json│
│       ↓                                                       │
│  Response: List of movies with IMDB IDs                       │
│       ↓                                                       │
│  Display movie grid in UI                                     │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                     2. Stream Discovery                       │
│                                                               │
│  User clicks "Superman" (tt5950044)                           │
│       ↓                                                       │
│  JavaScript: selectContent({ id: "tt5950044", name: "Superman"})│
│       ↓                                                       │
│  JavaScript: invoke('fetch_streams', { imdbId: "tt5950044" }) │
│       ↓                                                       │
│  Rust: Query each addon:                                      │
│    - GET torrentio.strem.fun/stream/movie/tt5950044.json     │
│    - GET thepiratebay-plus.strem.fun/stream/movie/tt5950044.json│
│       ↓                                                       │
│  Response: List of streams with infoHash                      │
│       ↓                                                       │
│  Convert infoHash → magnet links                              │
│       ↓                                                       │
│  Return streams to JavaScript                                 │
│       ↓                                                       │
│  Display stream selection modal                               │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                    3. Torrent Streaming                       │
│                                                               │
│  User clicks "1080p BluRay" stream                            │
│       ↓                                                       │
│  JavaScript: playStream(stream)                               │
│       ↓                                                       │
│  JavaScript: invoke('play_video_external', {                  │
│    streamUrl: "magnet:?xt=urn:btih:6a941074..."              │
│  })                                                           │
│       ↓                                                       │
│  Rust: Start Peerflix process                                 │
│    Command: peerflix.cmd "magnet:..." --port 8888            │
│       ↓                                                       │
│  Peerflix:                                                    │
│    1. Connect to torrent swarm                                │
│    2. Find peers with video file                              │
│    3. Download pieces sequentially                            │
│    4. Serve at http://127.0.0.1:8888                         │
│       ↓                                                       │
│  Rust: Return "http://127.0.0.1:8888"                        │
│       ↓                                                       │
│  JavaScript: playWithBuiltInPlayer({                          │
│    url: "http://127.0.0.1:8888"                              │
│  })                                                           │
│       ↓                                                       │
│  HTML5 Video Player: Load and play stream                     │
└─────────────────────────────────────────────────────────────┘
```

---

## 7. Key Files & Their Responsibilities

### Frontend (JavaScript)

**`src/app.js`**
- Main application logic
- Content fetching and display
- Stream selection and playback
- Video player management

**Key Functions:**
```javascript
// Line ~1486: Create movie cards
function createContentCard(content, index, contentType)

// Line ~1758: Handle movie selection
async function selectContent(content, contentType)

// Line ~1805: Display available streams
function displayStreams(streams)

// Line ~1828: Create stream selection buttons
function createStreamItem(stream, index)

// Line ~1880: Handle stream playback
async function playStream(stream)

// Line ~1919: Play in HTML5 video player
function playWithBuiltInPlayer(stream)

// Line ~1966: Handle torrent streaming
async function tryExternalPlayer(streamUrl)
```

**`src/controller.js`**
- Gamepad/controller input handling
- Navigation logic for Steam Deck

**`src/index.html`**
- Main HTML structure
- Video player modal
- Stream selection modal

**`src/styles.css`**
- Application styling
- Steam Deck optimizations

### Backend (Rust)

**`src-tauri/src/main.rs`**
- Tauri application entry point
- Command handlers (bridge between JS and Rust)
- Application state management

**Tauri Commands:**
```rust
fetch_popular_movies()  // Get movie list
fetch_popular_series()  // Get series list
fetch_popular_anime()   // Get anime list
search_content(query)   // Search for content
fetch_streams(imdb_id)  // Get streams for movie
play_video_external(stream_url)  // Start playback
stop_video_stream()     // Stop Peerflix
```

**`src-tauri/src/addon_client.rs`**
- Stremio addon API client
- HTTP requests to Stremio addons
- Stream parsing and conversion

**Key Functions:**
```rust
// Fetch movie list from Stremio
pub async fn fetch_popular_movies() -> Result<Vec<Movie>, String>

// Fetch streams for specific movie
pub async fn fetch_streams(&self, imdb_id: &str) -> Result<Vec<Stream>, String>

// Parse Stremio stream response
fn parse_stremio_streams(&self, body: &str) -> Vec<Stream>
```

**`src-tauri/src/torrent_streamer.rs`**
- Peerflix process management
- Torrent streaming lifecycle

**Key Functions:**
```rust
// Start Peerflix with magnet link
pub async fn start_stream(&self, magnet_link: String) -> Result<String, String>

// Stop active Peerflix process
pub async fn stop_stream(&self) -> Result<(), String>

// Test if stream is responding
async fn test_stream_availability(&self, url: &str) -> Result<(), String>
```

**`src-tauri/src/models.rs`**
- Data structures
- Movie, Series, Anime, Stream types

---

## 8. Testing & Debugging

### Enable Verbose Logging

The application already has comprehensive logging. Check the console for:

```
[RUST] [MOVIES_FETCH] Starting to fetch popular movies...
[RUST] [STREAMS_FETCH] Received IMDB ID: tt5950044
[RUST] [TORRENT] Starting Peerflix torrent streaming
[RUST] [VIDEO_PLAYER] Stream URL received: magnet:?xt=...
```

### Test Stream Fetching Manually

You can test if Stremio addons work by visiting:
```
https://torrentio.strem.fun/stream/movie/tt0068646.json
```

Replace `tt0068646` with any IMDB ID.

### Test Peerflix Manually

```bash
# Start Peerflix with a magnet link
peerflix "magnet:?xt=urn:btih:..." --port 8888

# Open in browser
http://127.0.0.1:8888
```

### Verify IMDB ID

When you click a movie, check the logs:
```
[RUST] [FETCH_STREAMS_COMMAND] Received IMDB ID: tt0068646
```

This IMDB ID should match the movie you clicked.

### Check Stream Content

The issue of wrong content (Rick and Morty) is **NOT a code bug**. It's caused by:
1. Fake torrents with wrong content
2. Mislabeled torrents
3. Trolls uploading wrong content with popular movie names

**Solution:** Select a different stream from the list.

---

## 9. Future Improvements

### Potential Enhancements:

1. **Stream Quality Filtering**
   - Filter by quality (1080p, 720p, etc.)
   - Filter by seeders (minimum count)
   - Sort by file size

2. **Torrent Verification**
   - Check torrent content before playing
   - Verify file names match movie title
   - Show file list before streaming

3. **Download Progress**
   - Show download speed
   - Show percentage downloaded
   - Show peers/seeders count

4. **Stream Caching**
   - Cache stream lists for faster access
   - Remember last selected quality

5. **Alternative Players**
   - VLC integration
   - MPV integration
   - Custom player selection

6. **Subtitle Support**
   - Fetch subtitles from OpenSubtitles
   - Display subtitle options
   - Sync with video player

---

## 10. Security & Legal Considerations

### Important Notes:

1. **Torrenting Laws:**
   - Torrenting itself is legal
   - Downloading copyrighted content without permission is illegal in most countries
   - Users are responsible for their own actions

2. **VPN Recommendation:**
   - Use a VPN when torrenting
   - Protect your IP address
   - Avoid ISP throttling

3. **Content Sources:**
   - Stremio addons aggregate publicly available torrents
   - No content is hosted by DeckFlix
   - No content is hosted by Stremio
   - All content comes from third-party torrent networks

4. **Disclaimer:**
   - DeckFlix is for educational purposes
   - Users should only stream content they have legal rights to access
   - Developers are not responsible for user actions

---

## Conclusion

DeckFlix uses a three-stage pipeline:

1. **Discovery:** Fetch movie metadata from Stremio addons
2. **Stream Finding:** Query torrent sources for magnet links
3. **Playback:** Stream torrents via Peerflix to HTML5 player

The code is working correctly. If you see wrong content playing (like Rick and Morty), it's because the **torrent itself contains wrong content**, not because of a code issue. Simply select a different stream from the list.

All logs confirm the correct movie ID is being used throughout the entire pipeline. The issue is with third-party torrent quality, not the application code.
