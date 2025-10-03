# DeckFlix Quick Start Guide

## Installation (5 minutes)

### Windows
```bash
# 1. Install MPV (recommended) or VLC
# Download from: https://mpv.io or https://videolan.org

# 2. Install Node.js and Peerflix
npm install -g peerflix

# 3. Clone and build DeckFlix
git clone <your-repo-url>
cd deckflix/deckflix
npm install
npm run tauri dev
```

### Linux / Steam Deck
```bash
# 1. Install MPV
sudo pacman -S mpv  # Arch/Steam Deck
# or
sudo apt install mpv  # Ubuntu/Debian

# 2. Install Node.js and Peerflix
sudo pacman -S nodejs npm  # Arch/Steam Deck
npm install -g peerflix

# 3. Clone and build DeckFlix
git clone <your-repo-url>
cd deckflix/deckflix
npm install
npm run tauri dev
```

## First Run

1. **Launch DeckFlix** - App opens to movie grid
2. **Select a movie** - Click or press A button
3. **Choose stream** - Pick quality (1080p, 720p, etc.)
4. **Wait for player** - MPV/VLC will launch automatically
5. **Enjoy!** - Video plays with correct title

## How It Works

```
User selects movie
    ↓
Frontend fetches streams
    ↓
User picks stream (magnet link)
    ↓
Backend extracts torrent hash
    ↓
Peerflix starts downloading
    ↓
Backend finds video file in hash directory
    ↓
MPV/VLC launches with:
  - Correct video file path
  - Movie title: "The Godfather (1972)"
  - Fullscreen mode
    ↓
Video plays!
```

## Troubleshooting (2 minutes)

### "No video player found"
```bash
# Install MPV (recommended)
# Windows: Download from mpv.io
# Linux: sudo apt install mpv
# Steam Deck: sudo pacman -S mpv
```

### "Peerflix not installed"
```bash
npm install -g peerflix
```

### Wrong movie plays
```bash
# Clear torrent cache
rm -rf /tmp/torrent-stream/*  # Linux
rd /s /q C:\tmp\torrent-stream  # Windows
```

### Video shows "Unknown Movie"
- Check console logs for metadata
- Verify appState.currentContent has correct data
- See VIDEO_PLAYBACK_FIXES_SUMMARY.md for details

## Controller Support

| Action | Keyboard | Controller |
|--------|----------|------------|
| Navigate | Arrow Keys | D-Pad / Left Stick |
| Select | Enter | A Button |
| Back | Escape | B Button |
| Search | / | X Button |

## File Locations

**Windows:**
- Torrents: `C:\tmp\torrent-stream\{hash}\`
- Config: `%APPDATA%\deckflix\`

**Linux/Steam Deck:**
- Torrents: `/tmp/torrent-stream/{hash}/`
- Config: `~/.config/deckflix/`

## Console Commands

**Check hash extraction:**
```
[RUST] [VIDEO_PLAYER] Extracted hash: 20edd4cd63eb1833827c8e35011fb8792e4d77d2
```

**Check directory lookup:**
```
[RUST] [VIDEO_FINDER] Looking in directory: /tmp/torrent-stream/20edd4cd...
[RUST] [VIDEO_FINDER] Found video file: /tmp/torrent-stream/.../movie.mp4
```

**Check metadata:**
```
[RUST] [VIDEO_PLAYER] Title: The Godfather
[RUST] [VIDEO_PLAYER] Year: Some("1972")
[RUST] [VIDEO_PLAYER] Media title: The Godfather (1972)
```

**Check player launch:**
```
[RUST] [VIDEO_PLAYER] [1/9] Trying: mpv
[RUST] [VIDEO_PLAYER] ✅ Launched mpv (PID: 12345)
```

## Build for Production

```bash
# Development
npm run tauri dev

# Production build
npm run tauri build

# Output locations:
# Windows: src-tauri/target/release/DeckFlix.exe
# Linux: src-tauri/target/release/deckflix
# Also creates .deb, .AppImage, .msi
```

## Key Features

✅ Correct movie plays every time (hash-based lookup)
✅ Player shows movie title and year
✅ MPV prioritized for Steam Deck
✅ Automatic fallback to VLC
✅ Cross-platform (Windows/Linux)
✅ Controller support
✅ Fullscreen by default

## Performance Tips

- **Use wired connection** for torrents
- **Let movie buffer** ~30 seconds before playing
- **Close other apps** for best performance
- **Steam Deck:** Use MPV (lighter than VLC)

## Next Steps

- **Full testing:** See `TESTING_CHECKLIST.md`
- **Steam Deck setup:** See `STEAM_DECK_DEPLOYMENT.md`
- **Technical details:** See `VIDEO_PLAYBACK_FIXES_SUMMARY.md`

## Quick Debug

```bash
# Check if players installed
which mpv
which vlc

# Check if peerflix installed
which peerflix

# Check torrent directory
ls -la /tmp/torrent-stream/

# Monitor peerflix
ps aux | grep peerflix

# View DeckFlix logs
# (Check console output in terminal)
```

## Common Issues

| Issue | Solution |
|-------|----------|
| Blank screen in built-in player | Use external player (MPV/VLC) |
| Wrong movie plays | Clear cache: `rm -rf /tmp/torrent-stream/*` |
| No title shown | Check metadata in console logs |
| Player doesn't launch | Verify MPV/VLC installed |
| Slow buffering | Check internet connection |

## Support

**Documentation:**
- Testing: `TESTING_CHECKLIST.md`
- Deployment: `STEAM_DECK_DEPLOYMENT.md`
- Technical: `VIDEO_PLAYBACK_FIXES_SUMMARY.md`

**Logs:**
- Console: Run `npm run tauri dev` in terminal
- Rust: `[RUST]` prefix in console
- JavaScript: Standard console.log

**GitHub Issues:**
- Report bugs with console logs
- Include system info (OS, player version)
- Attach screenshots if possible

---

**Enjoy DeckFlix! 🎬**
