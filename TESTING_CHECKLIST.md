# DeckFlix Video Playback - Testing Checklist

## Prerequisites
- [ ] MPV or VLC installed on test system
- [ ] Node.js and peerflix installed (`npm install -g peerflix`)
- [ ] Test movie selected with known torrent hash

## Phase 1: Hash Extraction Testing

### Test 1.1: Magnet Link Hash Extraction
- [ ] Select a movie with magnet link stream
- [ ] Check console logs for `[RUST] [VIDEO_PLAYER] Extracted hash:`
- [ ] Verify hash is 40 characters (SHA-1) or 32 characters (MD5)
- [ ] Verify hash is lowercase

**Expected Output:**
```
[RUST] [VIDEO_PLAYER] Magnet link detected
[RUST] [VIDEO_PLAYER] Extracted hash: 20edd4cd63eb1833827c8e35011fb8792e4d77d2
```

## Phase 2: Directory Lookup Testing

### Test 2.1: Windows Path Resolution
- [ ] On Windows, verify it looks in `C:/tmp/torrent-stream/{hash}/`
- [ ] Check console for `[RUST] [VIDEO_FINDER] Looking in directory:`
- [ ] Verify correct hash directory is searched (not random first directory)

### Test 2.2: Linux/Steam Deck Path Resolution
- [ ] On Linux, verify it looks in `/tmp/torrent-stream/{hash}/`
- [ ] Confirm cross-platform path handling works

**Expected Output:**
```
[RUST] [VIDEO_FINDER] Looking in directory: C:/tmp/torrent-stream/20edd4cd63eb1833827c8e35011fb8792e4d77d2
[RUST] [VIDEO_FINDER] Found video file: C:/tmp/torrent-stream/20edd4cd63eb1833827c8e35011fb8792e4d77d2/The.Godfather.1972.1080p.BluRay.mp4
```

## Phase 3: Video File Detection

### Test 3.1: Multiple File Extensions
Test with torrents containing:
- [ ] .mp4 files
- [ ] .mkv files
- [ ] .avi files
- [ ] .webm files

### Test 3.2: Multi-File Torrents
- [ ] Test torrent with multiple files (should find largest video file)
- [ ] Test torrent with non-video files (should skip .txt, .nfo, etc.)

## Phase 4: Metadata Passing

### Test 4.1: MPV Title Display
- [ ] Play a movie using MPV
- [ ] Verify MPV window shows: `"The Godfather (1972)"` (not "Unknown Movie")
- [ ] Check console for `[RUST] [VIDEO_PLAYER] Media title:`

### Test 4.2: VLC Title Display
- [ ] Play a movie using VLC
- [ ] Verify VLC window shows correct title with year
- [ ] Verify `--meta-title` flag is used

**Expected Console Output:**
```
[RUST] [VIDEO_PLAYER] Title: The Godfather
[RUST] [VIDEO_PLAYER] Year: Some("1972")
[RUST] [VIDEO_PLAYER] Type: movie
[RUST] [VIDEO_PLAYER] Media title: The Godfather (1972)
```

## Phase 5: Player Fallback System

### Test 5.1: MPV Priority
- [ ] With both MPV and VLC installed, verify MPV launches first
- [ ] Check console shows `[RUST] [VIDEO_PLAYER] [1/9] Trying: mpv`

### Test 5.2: VLC Fallback
- [ ] Uninstall/disable MPV
- [ ] Verify VLC is used as fallback
- [ ] Check correct VLC flags are used (`--meta-title`, `--fullscreen`, `--play-and-exit`)

### Test 5.3: No Player Installed
- [ ] Temporarily rename both MPV and VLC executables
- [ ] Verify error message shown:
  ```
  No video player found. Please install MPV or VLC:
  • Windows: mpv.io or videolan.org
  • Steam Deck: 'sudo pacman -S mpv' or Discover app
  • Linux: 'sudo apt install mpv' or 'sudo dnf install mpv'
  ```

## Phase 6: Built-in Player Fallback

### Test 6.1: Peerflix Stream URL Fallback
- [ ] If video file not found in hash directory, verify fallback to peerflix stream
- [ ] Check built-in player receives correct URL (`http://127.0.0.1:8888`)
- [ ] Verify player shows correct title

## Phase 7: Error Handling

### Test 7.1: Invalid Magnet Link
- [ ] Test with malformed magnet link (no `btih:`)
- [ ] Verify error message: `"Could not find btih: in magnet link"`

### Test 7.2: Hash Directory Doesn't Exist
- [ ] Test before peerflix creates directory
- [ ] Verify graceful fallback to stream URL

### Test 7.3: No Video Files in Directory
- [ ] Create hash directory with only .txt files
- [ ] Verify error message and fallback behavior

## Phase 8: Steam Deck Specific

### Test 8.1: Resolution Compatibility
- [ ] Launch on Steam Deck (1280x800 resolution)
- [ ] Verify UI elements are correctly sized
- [ ] Verify video plays in fullscreen at correct resolution

### Test 8.2: Controller Support
Test with:
- [ ] Steam Deck built-in controls
- [ ] Xbox controller via Bluetooth
- [ ] PlayStation DualSense controller
- [ ] Keyboard fallback

### Test 8.3: Gaming Mode
- [ ] Launch DeckFlix in Steam Deck Gaming Mode
- [ ] Verify video player launches correctly
- [ ] Verify can return to DeckFlix after video ends

### Test 8.4: Flatpak MPV/VLC
- [ ] Test with Flatpak-installed MPV: `flatpak run io.mpv.Mpv`
- [ ] Test with Flatpak-installed VLC: `flatpak run org.videolan.VLC`
- [ ] Verify correct command structure in logs

## Phase 9: Cross-Platform Testing

### Test 9.1: Windows
- [ ] Test on Windows 10
- [ ] Test on Windows 11
- [ ] Verify C:\tmp\torrent-stream\ path works
- [ ] Test with Windows-installed MPV/VLC

### Test 9.2: Linux (Ubuntu/Debian)
- [ ] Test on Ubuntu/Debian
- [ ] Verify /tmp/torrent-stream/ path works
- [ ] Test with apt-installed MPV/VLC

### Test 9.3: Linux (Arch/Steam Deck)
- [ ] Test on Arch Linux / Steam Deck
- [ ] Test with pacman-installed MPV
- [ ] Test with Flatpak-installed players

## Phase 10: Integration Testing

### Test 10.1: End-to-End Movie Playback
1. [ ] Browse movies in DeckFlix
2. [ ] Select "The Godfather"
3. [ ] Choose a stream
4. [ ] Verify correct torrent hash extracted
5. [ ] Verify correct directory searched
6. [ ] Verify video file found
7. [ ] Verify MPV launches with correct title: "The Godfather (1972)"
8. [ ] Verify video plays correctly
9. [ ] Close player and verify DeckFlix still responsive

### Test 10.2: Multiple Playback Sessions
- [ ] Play movie A
- [ ] Close player
- [ ] Play movie B
- [ ] Verify movie B's hash directory is used (not movie A's)
- [ ] Verify movie B's title is displayed correctly

### Test 10.3: Series Episode Playback
- [ ] Select a TV series
- [ ] Play an episode
- [ ] Verify metadata includes series name and episode info
- [ ] Verify correct video file found

## Debugging Commands

If issues occur, check these directories:

**Windows:**
```cmd
dir C:\tmp\torrent-stream\
dir C:\tmp\torrent-stream\{hash}\
```

**Linux/Steam Deck:**
```bash
ls -la /tmp/torrent-stream/
ls -la /tmp/torrent-stream/{hash}/
```

**Check which players are installed:**
```bash
# MPV
which mpv
mpv --version

# VLC
which vlc
vlc --version

# Peerflix
which peerflix
peerflix --version
```

## Known Issues to Watch For

- [ ] If video shows wrong movie, check hash extraction in logs
- [ ] If "Unknown Movie" appears, check metadata passing in logs
- [ ] If no player launches, check player installation paths
- [ ] If built-in player shows blank screen, check codec support

## Success Criteria

✅ All tests pass
✅ Correct movie plays every time
✅ Player window shows correct title
✅ Works on Windows and Linux
✅ Works on Steam Deck
✅ Graceful error messages when players not installed
