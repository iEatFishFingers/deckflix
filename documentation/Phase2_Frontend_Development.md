# Phase 2: Frontend UI Development - Development Documentation

## Overview
This phase involved creating a Steam Deck optimized user interface with Netflix-style movie browsing, stream selection modals, and comprehensive controller support. The frontend communicates with the Rust backend through Tauri's invoke system.

## What Was Implemented

### 1. HTML Structure (`src/index.html`)

**Complete UI Redesign:**
```html
<!doctype html>
<html lang="en">
<head>
    <title>DeckFlix</title>
    <script type="module" src="/app.js" defer></script>
    <script type="module" src="/controller.js" defer></script>
</head>
```

**Key UI Components:**
- **Header**: App branding and status indicators
- **Movies Grid**: Netflix-style card layout
- **Stream Modal**: Selection interface for available streams
- **Loading States**: Spinner animations and feedback
- **Error Handling**: User-friendly error messages
- **Controller Hints**: Visual indicators for button functions

**Semantic Structure:**
```html
<!-- Main app sections -->
<header class="header">...</header>
<main class="main-content">
    <div id="movies-grid" class="movies-grid">
        <!-- Dynamically populated movie cards -->
    </div>
</main>

<!-- Modal for stream selection -->
<div id="stream-modal" class="modal hidden">
    <div class="modal-content">...</div>
</div>
```

### 2. Steam Deck Optimized CSS (`src/styles.css`)

**Design System:**
```css
:root {
    /* Steam Deck specific dimensions */
    --screen-width: 1280px;
    --screen-height: 800px;

    /* Dark theme for battery efficiency */
    --bg-primary: #0a0a0f;
    --bg-secondary: #1a1a20;
    --accent: #00d4ff;

    /* Focus system for controller navigation */
    --focus-color: #00d4ff;
    --focus-shadow: 0 0 0 3px rgba(0, 212, 255, 0.5);
}
```

**Grid Layout System:**
```css
.movies-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 20px;
    padding: 10px;
}

.movie-card {
    aspect-ratio: 2/3; /* Movie poster proportions */
    border-radius: 12px;
    transition: all 0.3s ease;
}
```

**Controller Focus System:**
```css
.focusable:focus,
.focusable.focused {
    outline: none;
    border-color: var(--focus-color);
    box-shadow: var(--focus-shadow);
    transform: scale(1.05);
    z-index: 10;
}
```

### 3. Core Application Logic (`src/app.js`)

**Application State Management:**
```javascript
let appState = {
    movies: [],
    currentMovie: null,
    currentStreams: [],
    isLoading: false,
    focusedElement: null,
    focusIndex: 0
};
```

**Tauri Integration:**
```javascript
// Fetch movies from Rust backend
const movies = await invoke('fetch_popular_movies');

// Get streams for selected movie
const streams = await invoke('fetch_streams', { imdbId: movie.id });

// Launch external video player
const result = await invoke('play_video_external', { streamUrl: stream.url });
```

**Dynamic UI Generation:**
```javascript
function createMovieCard(movie, index) {
    const card = document.createElement('div');
    card.className = 'movie-card focusable';
    card.dataset.movieIndex = index;

    // Movie poster with fallback
    if (movie.poster) {
        const img = document.createElement('img');
        img.src = movie.poster;
        img.onerror = () => {
            poster.innerHTML = '🎬 No Image';
        };
    }

    // Add interaction handlers
    card.addEventListener('click', () => selectMovie(movie));
    card.addEventListener('focus', () => setFocusedElement(card));

    return card;
}
```

**Navigation System:**
```javascript
function handleKeyboard(e) {
    const movieCards = Array.from(elements.moviesGrid.querySelectorAll('.movie-card'));
    const currentIndex = appState.focusIndex;
    let newIndex = currentIndex;

    switch (e.key) {
        case 'ArrowLeft':
            newIndex = Math.max(0, currentIndex - 1);
            break;
        case 'ArrowRight':
            newIndex = Math.min(movieCards.length - 1, currentIndex + 1);
            break;
        case 'ArrowUp':
            const cols = Math.floor(elements.moviesGrid.offsetWidth / 200);
            newIndex = Math.max(0, currentIndex - cols);
            break;
        case 'ArrowDown':
            const colsDown = Math.floor(elements.moviesGrid.offsetWidth / 200);
            newIndex = Math.min(movieCards.length - 1, currentIndex + colsDown);
            break;
    }
}
```

### 4. Controller Support (`src/controller.js`)

**Gamepad API Integration:**
```javascript
class GamepadController {
    constructor() {
        this.gamepads = {};
        this.buttonStates = {};
        this.deadZone = 0.2;

        // Steam Deck button mapping
        this.buttons = {
            A: 0,      // Cross (select)
            B: 1,      // Circle (back)
            X: 2,      // Square (refresh)
            Y: 3,      // Triangle (info)
            LB: 4,     // Left bumper
            RB: 5,     // Right bumper
            DPAD_UP: 12,
            DPAD_DOWN: 13,
            DPAD_LEFT: 14,
            DPAD_RIGHT: 15
        };
    }
}
```

**Input Processing:**
```javascript
processGamepadInput(gamepad) {
    // Process button presses
    this.processButtons(gamepad, now);

    // Process analog stick movement
    this.processAnalogSticks(gamepad, now);
}

handleButtonPress(buttonName, gamepad) {
    // Haptic feedback
    this.vibrate(gamepad, 50);

    switch (buttonName) {
        case 'A':
            this.handleAButton(isModalOpen);
            break;
        case 'B':
            this.handleBButton(isModalOpen);
            break;
        // ... more button mappings
    }
}
```

## Challenges Encountered & Solutions

### Challenge 1: Steam Deck Screen Resolution
**Problem**: Standard web layouts don't work well on 1280x800 handheld screens.

**Solution**:
- Created responsive grid system that adapts to screen size
- Used CSS custom properties for consistent sizing
- Optimized for landscape orientation:
```css
@media (max-width: 1280px) {
    .movies-grid {
        grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
        gap: 15px;
    }
}
```

### Challenge 2: Controller Navigation Logic
**Problem**: Web UIs are designed for mouse/touch, not gamepad navigation.

**Solution**:
- Implemented custom focus management system
- Created grid-based navigation algorithm:
```javascript
// Calculate grid columns dynamically
const cols = Math.floor(elements.moviesGrid.offsetWidth / 200);

// Navigate up/down by full rows
case 'ArrowUp':
    newIndex = Math.max(0, currentIndex - cols);
    break;
case 'ArrowDown':
    newIndex = Math.min(movieCards.length - 1, currentIndex + cols);
    break;
```

### Challenge 3: Async Data Loading and UI States
**Problem**: Managing loading states, errors, and data updates across async operations.

**Solution**:
- Centralized state management pattern
- Clear UI state transitions:
```javascript
async function loadPopularMovies() {
    try {
        setLoadingState(true);
        hideError();

        const movies = await invoke('fetch_popular_movies');
        displayMovies(movies);
        setLoadingState(false);
    } catch (error) {
        showError(error.toString());
        setLoadingState(false);
    }
}
```

### Challenge 4: Modal Focus Management
**Problem**: When modals open, focus needs to move to modal content and be trapped there.

**Solution**:
- Separate keyboard handlers for modal vs main content
- Focus restoration when modal closes:
```javascript
function closeStreamModal() {
    elements.streamModal.classList.add('hidden');

    // Return focus to previously focused movie card
    if (appState.focusedElement) {
        appState.focusedElement.focus();
    }
}
```

### Challenge 5: Image Loading and Fallbacks
**Problem**: Movie poster URLs from APIs sometimes fail to load.

**Solution**:
- Implemented graceful fallback system:
```javascript
if (movie.poster) {
    const img = document.createElement('img');
    img.src = movie.poster;
    img.onerror = () => {
        poster.innerHTML = '🎬 No Image';
    };
    poster.appendChild(img);
} else {
    poster.innerHTML = '🎬 No Image';
}
```

### Challenge 6: Gamepad Input Debouncing
**Problem**: Analog sticks cause rapid repeated navigation events.

**Solution**:
- Implemented timing-based debouncing:
```javascript
processAnalogSticks(gamepad, now) {
    // Prevent too frequent navigation
    if (now - this.lastNavTime < this.repeatRate && this.isNavigating) {
        return;
    }

    // Only trigger on significant movement
    if (Math.abs(leftX) > this.deadZone || Math.abs(leftY) > this.deadZone) {
        this.handleAnalogNavigation(leftX, leftY, now);
    }
}
```

## Key Learning Points

### 1. Steam Deck UI Design Principles
- **Large Touch Targets**: Minimum 44px for finger navigation
- **High Contrast**: Dark themes reduce eye strain and save battery
- **Clear Focus Indicators**: Essential for controller navigation
- **Landscape Optimization**: Design for 16:10 aspect ratio

### 2. Gamepad API Patterns
```javascript
// Check for gamepad support
if (!navigator.getGamepads) {
    console.warn('Gamepad API not supported');
    return;
}

// Poll for input changes
function poll() {
    const gamepads = navigator.getGamepads();
    for (let i = 0; i < gamepads.length; i++) {
        if (gamepads[i]) {
            this.processGamepadInput(gamepads[i]);
        }
    }
    requestAnimationFrame(() => this.poll());
}
```

### 3. Tauri Frontend-Backend Communication
```javascript
// All backend calls return Promises
const { invoke } = window.__TAURI__.core;

// Error handling with try-catch
try {
    const result = await invoke('command_name', { param: value });
} catch (error) {
    console.error('Backend error:', error);
}
```

### 4. CSS Grid for Dynamic Layouts
```css
/* Auto-fitting grid that adapts to content */
grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));

/* Maintain aspect ratios */
aspect-ratio: 2/3;
```

### 5. Focus Management Best Practices
- Always provide visible focus indicators
- Trap focus within modals
- Restore focus when closing overlays
- Use semantic HTML elements when possible

## UI/UX Design Decisions

### Visual Hierarchy
1. **Header**: App branding and status (fixed position)
2. **Main Grid**: Primary content area (scrollable)
3. **Modal**: Overlay for detailed interactions
4. **Controls**: Always-visible navigation hints

### Color Scheme
- **Primary Background**: Dark blue-black (#0a0a0f)
- **Secondary**: Slightly lighter panels (#1a1a20)
- **Accent**: Bright cyan (#00d4ff) for focus and actions
- **Text**: High contrast white/gray

### Typography
- **Large Fonts**: 18px base size for readability
- **System Fonts**: Native font stack for performance
- **Weight Hierarchy**: 400 (normal), 600 (semibold), 700 (bold)

## Performance Optimizations

### 1. DOM Manipulation
```javascript
// Efficient element creation
const fragment = document.createDocumentFragment();
movies.forEach(movie => {
    fragment.appendChild(createMovieCard(movie));
});
elements.moviesGrid.appendChild(fragment);
```

### 2. Event Delegation
```javascript
// Single event listener for all movie cards
elements.moviesGrid.addEventListener('click', (e) => {
    const card = e.target.closest('.movie-card');
    if (card) {
        const index = parseInt(card.dataset.movieIndex);
        selectMovie(appState.movies[index]);
    }
});
```

### 3. CSS Animations
```css
/* Hardware-accelerated transforms */
.movie-card:hover {
    transform: translateY(-5px);
    will-change: transform;
}
```

## File Structure
```
src/
├── index.html       # Main HTML structure
├── styles.css       # Steam Deck optimized styles
├── app.js          # Core application logic
└── controller.js    # Gamepad input handling
```

## Testing Strategies
- **Visual Testing**: Chrome DevTools device emulation at 1280x800
- **Controller Testing**: Browser gamepad simulator extensions
- **Error Testing**: Network disconnection and API failures
- **Focus Testing**: Tab navigation to verify keyboard accessibility

This phase successfully created a fully functional, Steam Deck optimized interface that provides an excellent user experience for browsing and selecting movies with either keyboard or controller input.