# DeckFlix - Steam Deck Deployment Guide

## Overview
This guide covers deploying DeckFlix to Steam Deck, including installation, configuration, and troubleshooting specific to the handheld gaming device.

## Prerequisites

### Required on Steam Deck
1. **Node.js and npm** (for peerflix)
2. **MPV video player** (recommended for Steam Deck)
3. **DeckFlix app binary**

## Installation Steps

### Step 1: Switch to Desktop Mode
1. Press Steam button
2. Navigate to Power → Switch to Desktop

### Step 2: Install MPV (Recommended)

**Option A: Using Pacman (Recommended)**
```bash
# Disable read-only filesystem
sudo steamos-readonly disable

# Install MPV
sudo pacman -S mpv

# Re-enable read-only (optional but recommended)
sudo steamos-readonly enable
```

**Option B: Using Flatpak**
```bash
# Open Discover app (Steam Deck's app store)
# Search for "MPV"
# Click Install

# Or via terminal:
flatpak install flathub io.mpv.Mpv
```

### Step 3: Install Node.js and Peerflix

**Option A: Using Pacman**
```bash
sudo steamos-readonly disable
sudo pacman -S nodejs npm
npm install -g peerflix
sudo steamos-readonly enable
```

**Option B: Using Flatpak**
```bash
flatpak install flathub org.freedesktop.Sdk.Extension.node18
# Note: Peerflix may need additional configuration with Flatpak Node.js
```

### Step 4: Install DeckFlix

**Option A: From Binary Release**
```bash
# Download DeckFlix AppImage or .deb package
cd ~/Downloads
chmod +x DeckFlix.AppImage
./DeckFlix.AppImage
```

**Option B: Build from Source**
```bash
# Clone repository
git clone https://github.com/yourusername/deckflix.git
cd deckflix/deckflix

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install dependencies
npm install

# Build Tauri app
npm run tauri build

# Binary will be in src-tauri/target/release/
```

### Step 5: Add to Steam (Gaming Mode)

1. In Desktop Mode, open Steam
2. Click "Games" → "Add a Non-Steam Game to My Library"
3. Click "Browse" and navigate to DeckFlix binary
4. Select DeckFlix and click "Add Selected Programs"
5. Right-click DeckFlix in library → Properties
6. Configure settings:
   - **Target:** `/path/to/DeckFlix`
   - **Start In:** `/path/to/deckflix/directory`
   - **Launch Options:** (leave empty or add `--fullscreen`)

### Step 6: Controller Configuration (Gaming Mode)

1. In Steam Library, select DeckFlix
2. Click Controller icon (🎮)
3. Choose "Browse Configs"
4. Select "Gamepad with Mouse Trackpad"
5. Customize mappings:
   - **D-Pad/Left Stick:** Navigate content grid
   - **A Button:** Select/Play
   - **B Button:** Back/Cancel
   - **X Button:** Search
   - **Y Button:** Close modal
   - **Left/Right Triggers:** Scroll content
   - **Start:** Pause video
   - **Select:** Open debug panel

## Steam Deck Specific Configuration

### Resolution Settings

DeckFlix is optimized for Steam Deck's **1280x800** resolution. The CSS already handles this:

```css
@media (max-width: 1280px) {
  /* Steam Deck optimizations applied automatically */
}
```

### Performance Optimization

**Recommended Settings in Steam:**
- **Framerate Limit:** 60 FPS
- **Refresh Rate:** 60 Hz
- **TDP Limit:** 10W (DeckFlix is lightweight)
- **GPU Clock:** Default (not needed for video streaming UI)

### Video Player Settings

**MPV Configuration for Steam Deck:**

Create `~/.config/mpv/mpv.conf`:
```ini
# Steam Deck optimizations
hwdec=auto
vo=gpu
profile=gpu-hq

# Performance
video-sync=display-resample
interpolation=yes
tscale=oversample

# Quality
scale=ewa_lanczossharp
cscale=ewa_lanczossharp

# Fullscreen by default
fullscreen=yes

# OSD settings for handheld
osd-font-size=32
osd-border-size=2
```

### Peerflix Configuration

Create `~/.peerflixrc`:
```json
{
  "connections": 50,
  "tmp": "/tmp/torrent-stream",
  "port": 8888,
  "buffer": "2MB"
}
```

## Troubleshooting

### Issue 1: MPV Not Found
**Symptoms:** Error message "No video player found"

**Solutions:**
```bash
# Check if MPV is installed
which mpv

# If Flatpak:
flatpak list | grep -i mpv

# Verify DeckFlix can find it
flatpak run io.mpv.Mpv --version
```

### Issue 2: Peerflix Not Starting
**Symptoms:** "Peerflix not installed" error

**Solutions:**
```bash
# Check peerflix installation
which peerflix
peerflix --version

# Reinstall if needed
npm install -g peerflix

# Check permissions
ls -la $(which peerflix)
```

### Issue 3: Video File Not Found
**Symptoms:** "No video files found in hash directory"

**Solutions:**
```bash
# Check torrent-stream directory
ls -la /tmp/torrent-stream/

# Check specific hash directory (replace {hash} with actual hash from logs)
ls -la /tmp/torrent-stream/{hash}/

# Monitor peerflix process
ps aux | grep peerflix

# Check peerflix logs (if running in terminal)
journalctl -f | grep peerflix
```

### Issue 4: Wrong Movie Plays
**Symptoms:** Playing "Movie A" but "Movie B" appears

**Solutions:**
```bash
# Clear torrent-stream cache
rm -rf /tmp/torrent-stream/*

# Check DeckFlix logs for hash extraction
# Should show: [RUST] [VIDEO_PLAYER] Extracted hash: {correct_hash}

# Verify hash directory matches selected movie
```

### Issue 5: Controller Not Working
**Symptoms:** Steam Deck buttons don't navigate DeckFlix

**Solutions:**
1. Verify controller configuration in Steam
2. Test in Desktop Mode with keyboard first
3. Check Steam Input is enabled for DeckFlix
4. Try "Gamepad with Mouse Trackpad" template

### Issue 6: Performance Issues
**Symptoms:** Laggy UI or stuttering video

**Solutions:**
```bash
# Close other apps in Gaming Mode
# Reduce Steam Deck TDP if overheating
# Check network speed (torrent streaming requires good connection)

# Monitor system resources
htop

# Check peerflix download speed
# (Look in peerflix output for download rate)
```

## Network Configuration

### Port Forwarding for Better Torrenting
Configure router to forward ports for better peer connections:
- **Port Range:** 6881-6889
- **Protocol:** TCP + UDP

### VPN Considerations
If using VPN on Steam Deck:
- Enable port forwarding on VPN
- Use VPN with P2P support
- Some VPNs may slow torrent speeds

## File Locations

**Important paths on Steam Deck:**
```
DeckFlix Config: ~/.config/deckflix/
MPV Config: ~/.config/mpv/mpv.conf
Peerflix Config: ~/.peerflixrc
Torrent Cache: /tmp/torrent-stream/
Logs: ~/.local/share/deckflix/logs/
```

## Gaming Mode Launch Script

Create a launch script for better Gaming Mode integration:

**`~/deckflix-launcher.sh`:**
```bash
#!/bin/bash

# Set environment variables
export XDG_CURRENT_DESKTOP=gamescope

# Ensure torrent directory exists
mkdir -p /tmp/torrent-stream

# Check if peerflix is running, kill old instances
pkill -f peerflix

# Launch DeckFlix
cd ~/deckflix
./DeckFlix
```

Make executable:
```bash
chmod +x ~/deckflix-launcher.sh
```

Use this script as the Steam shortcut target.

## Performance Tips

1. **Pre-cache popular movies:** Let torrents download fully before watching
2. **Use wired connection:** Ethernet via USB-C dock for best torrent speeds
3. **Close Steam overlay:** Press Steam button → Settings → disable overlay for DeckFlix
4. **Limit background downloads:** Pause Steam downloads while streaming

## Known Limitations

- **Bluetooth audio delay:** May have slight delay with wireless headphones during video playback
- **Gaming Mode suspend:** Videos won't resume after suspend (by design)
- **Battery life:** Torrenting + video = ~2-3 hours battery life
- **Storage:** Downloaded torrents consume SSD space in `/tmp` (cleared on reboot)

## Recommended Setup

**Ideal Steam Deck DeckFlix Setup:**
```
1. MPV installed via pacman (fastest, most compatible)
2. Peerflix installed globally via npm
3. DeckFlix added to Steam with custom artwork
4. Controller config: "Gamepad with Mouse Trackpad"
5. Ethernet connection via dock (for torrenting)
6. External controller or built-in controls
```

## Updating DeckFlix

**From Desktop Mode:**
```bash
cd ~/deckflix
git pull origin master
npm install
npm run tauri build
```

**Binary updates:**
- Download new AppImage
- Replace old binary
- Update Steam shortcut if path changed

## Steam Deck Shortcuts Summary

| Action | Steam Deck Button |
|--------|-------------------|
| Navigate Grid | D-Pad / Left Stick |
| Select Movie | A Button |
| Go Back | B Button |
| Search | X Button |
| Close Modal | Y Button |
| Scroll Content | L/R Triggers |
| Video: Play/Pause | Steam Button (in MPV) |
| Video: Seek | L/R Trackpad |
| Video: Fullscreen | Select Button |
| Video: Exit | B Button |

## Support

If issues persist:
1. Check DeckFlix logs: `~/.local/share/deckflix/logs/`
2. Enable debug panel: Press F12 in app
3. Check console output: Run DeckFlix from terminal
4. Report issues: GitHub repository

## Additional Resources

- Steam Deck documentation: https://help.steampowered.com/
- MPV manual: https://mpv.io/manual/stable/
- Peerflix GitHub: https://github.com/mafintosh/peerflix
- Tauri documentation: https://tauri.app/
