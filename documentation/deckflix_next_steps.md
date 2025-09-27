# DeckFlix - Next Development Steps

## 🎯 Development Roadmap

### Phase 1: Backend API Integration (Priority: HIGH)

#### 1.1 Rust Dependencies Setup
**File:** `src-tauri/Cargo.toml`
```toml
[dependencies]
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
tauri = { version = "1.0", features = ["api-all"] }
```

#### 1.2 Data Models Creation
**File:** `src-tauri/src/models.rs`
- Create `Movie` struct for Stremio movie data
- Create `Stream` struct for stream URLs and metadata
- Create `Addon` struct for addon configuration
- Implement JSON serialization/deserialization

#### 1.3 Stremio API Client
**File:** `src-tauri/src/addon_client.rs`
- HTTP client for Stremio addon APIs
- Functions for fetching popular movies
- Functions for getting stream links
- Error handling and response parsing

#### 1.4 Tauri Commands
**File:** `src-tauri/src/main.rs`
- `fetch_popular_movies()` command
- `fetch_streams(imdb_id)` command  
- `play_video_external(stream_url)` command
- `get_addon_catalogs()` command

**Estimated Time:** 2-3 hours

---

### Phase 2: Frontend UI Development (Priority: HIGH)

#### 2.1 Steam Deck UI Layout
**File:** `src/index.html`
- Grid layout for movie browsing (Netflix-style)
- Navigation breadcrumbs
- Stream selection modal
- Status/loading indicators
- Large touch targets for controller navigation

#### 2.2 Steam Deck Optimized Styling
**File:** `src/style.css`
- 1280x800 resolution optimization
- Large fonts and buttons for readability
- Focus indicators for controller navigation
- Dark theme (battery-friendly)
- Grid system for movie cards
- Modal styling for stream selection

#### 2.3 Core App Logic
**File:** `src/main.js`
- Movie data fetching and display
- Stream selection handling
- Navigation state management
- Error handling and user feedback
- Integration with Tauri backend commands

**Estimated Time:** 3-4 hours

---

### Phase 3: Controller Support (Priority: MEDIUM)

#### 3.1 Gamepad Integration
**File:** `src/controller.js`
- Gamepad API implementation
- D-pad/analog stick navigation
- Button mapping (A=select, B=back, bumpers=sections)
- Focus management system
- Haptic feedback (if supported)

#### 3.2 Navigation System
- Grid-based focus system
- Keyboard fallback support
- Smooth transitions between elements
- Section-based navigation (movies, streams, settings)

**Estimated Time:** 2-3 hours

---

### Phase 4: External Video Player Integration (Priority: MEDIUM)

#### 4.1 Player Commands
- Launch mpv/vlc with stream URLs
- Handle player process management
- Error handling for missing players
- Stream quality selection

#### 4.2 Steam Deck Integration
- Proper video player launching on Linux
- Controller passthrough to video player
- Return to app after playback

**Estimated Time:** 1-2 hours

---

### Phase 5: Testing & Optimization (Priority: LOW)

#### 5.1 Development Testing
- Test all Stremio addon integrations
- Verify controller navigation flows
- Performance testing with large movie catalogs
- Error scenario testing

#### 5.2 Steam Deck Preparation
- Cross-compilation setup for Linux
- AppImage/Flatpak packaging
- Installation and deployment testing

**Estimated Time:** 2-3 hours

---

## 🔧 Immediate Next Actions (Start Here)

### Action 1: Rust Dependencies
1. Open `src-tauri/Cargo.toml`
2. Add the four dependencies listed above
3. Run `npm run tauri dev` to verify compilation

### Action 2: Create Basic API Structure
1. Create `src-tauri/src/models.rs`
2. Create `src-tauri/src/addon_client.rs`
3. Add basic structs and HTTP client setup

### Action 3: Test API Connection
1. Implement one simple Tauri command
2. Test from frontend with basic button
3. Verify Rust-JavaScript communication

## 📋 Development Workflow

1. **Keep `npm run tauri dev` running** - provides hot reload
2. **Make incremental changes** - test frequently
3. **Frontend first approach** - build UI, then connect backend
4. **Controller testing** - use browser dev tools gamepad simulator

## 🎮 Steam Deck Considerations

- **Resolution:** 1280x800 (plan UI accordingly)
- **Input:** Primarily controller-based
- **Performance:** Optimize for handheld device limitations
- **Battery:** Dark themes and efficient rendering
- **Deployment:** Linux-compatible build process

## 🚀 Success Criteria

**Phase 1 Complete:** Can fetch and display movie data from Stremio  
**Phase 2 Complete:** Netflix-like grid interface working  
**Phase 3 Complete:** Full controller navigation functional  
**Phase 4 Complete:** Video playback working with external player  
**Phase 5 Complete:** Ready for Steam Deck deployment  

---

**Total Estimated Development Time:** 10-15 hours  
**Target Completion:** Next week  
**Next Session Focus:** Rust backend API integration