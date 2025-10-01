# DeckFlix Documentation

## Overview

DeckFlix is a streaming application built with Tauri that provides access to movies, TV series, and anime content. The application is optimized for Steam Deck and desktop usage, featuring a modern web-based UI with a Rust backend for performance and cross-platform compatibility.

## Architecture

### Technology Stack
- **Frontend**: Vanilla HTML, CSS, and JavaScript
- **Backend**: Rust with Tauri framework
- **HTTP Client**: reqwest for async networking
- **Serialization**: serde for JSON handling
- **UI Framework**: Native web technologies optimized for gamepad navigation

### Project Structure
```
deckflix/
├── src/                     # Frontend source files
│   ├── app.js              # Main application logic
│   ├── controller.js       # Gamepad/keyboard input handling
│   ├── index.html          # Main HTML structure
│   └── styles.css          # CSS styling optimized for Steam Deck
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── main.rs         # Tauri application entry point
│   │   ├── models.rs       # Data structures and types
│   │   ├── addon_client.rs # HTTP client for streaming services
│   │   └── lib.rs          # Library definitions
│   └── Cargo.toml          # Rust dependencies
├── package.json            # Node.js project configuration
└── README.md              # Basic project information
```

## Core Components

### Backend (Rust)

#### Main Application (`main.rs`)
- **Entry Point**: Initializes Tauri application with global state management
- **Commands**: Exposes async functions to frontend via Tauri's IPC system
- **Video Player Integration**: Handles launching external video players (VLC, MPV) with cross-platform support
- **State Management**: Uses `tokio::sync::Mutex` for thread-safe addon client access

**Key Functions:**
- `fetch_popular_movies()` - Retrieves trending movies from streaming addons
- `fetch_popular_series()` - Retrieves trending TV series
- `fetch_popular_anime()` - Retrieves trending anime content
- `search_content()` - Searches across all content types
- `fetch_streams()` - Gets streaming links for specific content
- `play_video_external()` - Launches external video players

#### Data Models (`models.rs`)
Defines the core data structures for content representation:

- **Movie**: Movie metadata with IMDB info, cast, genre, etc.
- **Series**: TV series with season/episode data and network info
- **Anime**: Anime content with MyAnimeList ratings and studio info
- **Stream**: Streaming link information with quality, seeders, and source type
- **SearchResult**: Unified search result across all content types
- **ContinueWatchingItem**: User progress tracking for localStorage

**Content Trait**: Provides common interface for all content types with methods for ID, name, poster, description, year, and rating access.

#### Addon Client (`addon_client.rs`)
- **HTTP Client**: Manages connections to multiple streaming addon services
- **Source Management**: Handles fallback between different addon providers
- **Error Handling**: Robust error handling with detailed logging
- **Performance Optimization**: Implements timeouts, deduplication, and result limiting

**Supported Addon Sources:**
- `v3-cinemeta.strem.io` - Primary metadata source
- `torrentio.strem.fun` - Main torrent streams
- `thepiratebay-plus.strem.fun` - PirateBay torrents
- `prowlarr.elfhosted.com` - Multi-indexer torrents

### Frontend (JavaScript)

#### Application Logic (`app.js`)
- **State Management**: Central application state for movies, series, anime, and search results
- **Tauri Integration**: Safe async communication with Rust backend
- **Continue Watching**: localStorage-based progress tracking
- **Debug System**: Comprehensive logging for development and troubleshooting

**Key Features:**
- Async content loading with loading states
- Search functionality with debounced input
- Continue watching persistence
- Error handling and user feedback

#### Controller System (`controller.js`)
- **Gamepad Support**: Xbox/PlayStation controller navigation
- **Keyboard Navigation**: Full keyboard accessibility
- **Focus Management**: Visual focus indicators for TV/gamepad usage
- **Input Mapping**: Configurable button mappings for different controllers

**Navigation Features:**
- Grid-based content navigation
- Section switching (Movies/Series/Anime)
- Search input handling
- Video player controls

#### User Interface (`index.html` & `styles.css`)
- **Responsive Design**: Optimized for Steam Deck's 1280x800 resolution
- **Grid Layout**: Card-based content display with hover effects
- **Dark Theme**: Steam Deck-friendly dark color scheme
- **Accessibility**: High contrast, readable fonts, and clear navigation

## Key Features

### Content Discovery
- Browse popular movies, TV series, and anime
- Search across all content types
- Category-based filtering
- Continue watching functionality

### Streaming Integration
- Multiple addon source support
- Torrent and direct stream handling
- Quality selection and metadata
- External video player integration

### Platform Optimization
- Steam Deck gamepad navigation
- Cross-platform video player support
- Performance optimized for handheld devices
- Debug panel for troubleshooting

### User Experience
- Responsive grid layouts
- Loading states and error handling
- Persistent continue watching list
- Keyboard and gamepad accessibility

## API Reference

### Tauri Commands

All commands are async and accessible from the frontend via `window.__TAURI__.core.invoke()`:

```javascript
// Fetch content
await safeInvoke('fetch_popular_movies')
await safeInvoke('fetch_popular_series')
await safeInvoke('fetch_popular_anime')

// Search
await safeInvoke('search_content', { query: 'search term' })

// Streaming
await safeInvoke('fetch_streams', { imdb_id: 'tt1234567' })
await safeInvoke('play_video_external', { stream_url: 'magnet:...' })

// Status
await safeInvoke('get_addon_status')
```

### Data Structures

#### Movie/Series/Anime
```rust
struct Movie {
    id: String,
    name: String,
    poster: Option<String>,
    background: Option<String>,
    description: Option<String>,
    year: Option<String>,
    imdb_rating: Option<String>,
    genre: Option<Vec<String>>,
    director: Option<Vec<String>>,
    cast: Option<Vec<String>>,
    runtime: Option<String>,
    country: Option<String>,
    language: Option<String>,
}
```

#### Stream
```rust
struct Stream {
    name: Option<String>,
    title: String,
    url: String,
    quality: Option<String>,
    size: Option<String>,
    seeders: Option<u32>,
    leechers: Option<u32>,
    source: Option<String>,
    language: Option<String>,
    subtitles: Option<Vec<String>>,
}
```

## Development

### Dependencies

**Rust Dependencies:**
- `tauri` - Application framework
- `reqwest` - HTTP client with JSON support
- `serde` - Serialization framework
- `tokio` - Async runtime
- `regex` - Regular expressions
- `urlencoding` - URL encoding utilities

**Frontend Dependencies:**
- `@tauri-apps/cli` - Development tools

### Building and Running

```bash
# Development mode
npm run tauri dev

# Production build
npm run tauri build

# Install dependencies
npm install
```

### Configuration

The application uses Tauri's configuration system with default capabilities for shell access (required for launching external video players).

## Security Considerations

- External video player execution is limited to known, safe applications
- HTTP requests are made only to trusted streaming addon sources
- No direct file system access beyond standard Tauri permissions
- Input validation for all user-provided search queries

## Performance Optimizations

- Result limiting (50 items max) for Steam Deck performance
- HTTP request timeouts (10 seconds)
- Duplicate content deduplication
- Efficient state management with minimal re-renders
- CSS optimizations for handheld device performance

## Future Enhancements

The codebase is structured to support future features such as:
- User accounts and preferences
- Custom addon source management
- Offline content caching
- Subtitle support integration
- Enhanced gamepad customization
- Multi-language support

---

*This documentation covers the current state of DeckFlix as of the latest codebase analysis. For the most up-to-date information, refer to the source code and commit history.*