// Steam Deck Controller Support
class GamepadController {
  constructor() {
    this.gamepads = {};
    this.buttonStates = {};
    this.deadZone = 0.2;
    this.repeatDelay = 200; // ms
    this.repeatRate = 100; // ms
    this.lastNavTime = 0;
    this.isNavigating = false;

    // Steam Deck controller mapping
    this.buttons = {
      A: 0,      // Cross button (bottom)
      B: 1,      // Circle button (right)
      X: 2,      // Square button (left)
      Y: 3,      // Triangle button (top)
      LB: 4,     // Left bumper
      RB: 5,     // Right bumper
      LT: 6,     // Left trigger
      RT: 7,     // Right trigger
      SELECT: 8, // View button
      START: 9,  // Menu button
      LS: 10,    // Left stick button
      RS: 11,    // Right stick button
      DPAD_UP: 12,
      DPAD_DOWN: 13,
      DPAD_LEFT: 14,
      DPAD_RIGHT: 15
    };

    this.axes = {
      LEFT_X: 0,
      LEFT_Y: 1,
      RIGHT_X: 2,
      RIGHT_Y: 3
    };

    this.init();
  }

  init() {
    // Check for gamepad support
    if (!navigator.getGamepads) {
      console.warn('Gamepad API not supported');
      return;
    }

    // Listen for gamepad events
    window.addEventListener('gamepadconnected', (e) => {
      console.log('Gamepad connected:', e.gamepad.id);
      this.gamepads[e.gamepad.index] = e.gamepad;
      this.showControllerStatus('Controller Connected: ' + e.gamepad.id);
    });

    window.addEventListener('gamepaddisconnected', (e) => {
      console.log('Gamepad disconnected:', e.gamepad.id);
      delete this.gamepads[e.gamepad.index];
      this.showControllerStatus('Controller Disconnected');
    });

    // Start polling
    this.poll();
  }

  poll() {
    // Update gamepad states
    const gamepads = navigator.getGamepads();
    for (let i = 0; i < gamepads.length; i++) {
      if (gamepads[i]) {
        this.gamepads[i] = gamepads[i];
      }
    }

    // Process input for each connected gamepad
    for (const index in this.gamepads) {
      const gamepad = this.gamepads[index];
      if (gamepad) {
        this.processGamepadInput(gamepad);
      }
    }

    // Continue polling
    requestAnimationFrame(() => this.poll());
  }

  processGamepadInput(gamepad) {
    const now = Date.now();

    // Process buttons
    this.processButtons(gamepad, now);

    // Process analog sticks
    this.processAnalogSticks(gamepad, now);
  }

  processButtons(gamepad, now) {
    // Get current button states
    const prevStates = this.buttonStates[gamepad.index] || {};
    const currentStates = {};

    for (const [name, index] of Object.entries(this.buttons)) {
      const button = gamepad.buttons[index];
      const pressed = button ? button.pressed : false;
      currentStates[name] = pressed;

      // Check for button press (transition from not pressed to pressed)
      if (pressed && !prevStates[name]) {
        this.handleButtonPress(name, gamepad);
      }
    }

    this.buttonStates[gamepad.index] = currentStates;
  }

  processAnalogSticks(gamepad, now) {
    // Prevent too frequent navigation
    if (now - this.lastNavTime < this.repeatRate && this.isNavigating) {
      return;
    }

    const leftX = gamepad.axes[this.axes.LEFT_X];
    const leftY = gamepad.axes[this.axes.LEFT_Y];

    // Check for significant movement outside dead zone
    if (Math.abs(leftX) > this.deadZone || Math.abs(leftY) > this.deadZone) {
      this.handleAnalogNavigation(leftX, leftY, now);
    } else {
      this.isNavigating = false;
    }
  }

  handleButtonPress(buttonName, gamepad) {
    console.log('Button pressed:', buttonName);

    // Vibrate for feedback (if supported)
    this.vibrate(gamepad, 50);

    // Handle button based on current app state
    if (window.DeckFlixApp) {
      const { elements } = window.DeckFlixApp;

      // Check if modal is open
      const isModalOpen = !elements.streamModal.classList.contains('hidden');

      switch (buttonName) {
        case 'A':
          this.handleAButton(isModalOpen);
          break;
        case 'B':
          this.handleBButton(isModalOpen);
          break;
        case 'X':
          this.handleXButton();
          break;
        case 'Y':
          this.handleYButton();
          break;
        case 'LB':
        case 'RB':
          this.handleBumpers(buttonName);
          break;
        case 'START':
          this.handleStartButton();
          break;
        case 'SELECT':
          this.handleSelectButton();
          break;
        case 'DPAD_UP':
          this.simulateKeyPress('ArrowUp');
          break;
        case 'DPAD_DOWN':
          this.simulateKeyPress('ArrowDown');
          break;
        case 'DPAD_LEFT':
          this.simulateKeyPress('ArrowLeft');
          break;
        case 'DPAD_RIGHT':
          this.simulateKeyPress('ArrowRight');
          break;
      }
    }
  }

  handleAnalogNavigation(x, y, now) {
    // Determine dominant direction
    if (Math.abs(x) > Math.abs(y)) {
      // Horizontal movement
      if (x > this.deadZone) {
        this.simulateKeyPress('ArrowRight');
      } else if (x < -this.deadZone) {
        this.simulateKeyPress('ArrowLeft');
      }
    } else {
      // Vertical movement
      if (y > this.deadZone) {
        this.simulateKeyPress('ArrowDown');
      } else if (y < -this.deadZone) {
        this.simulateKeyPress('ArrowUp');
      }
    }

    this.isNavigating = true;
    this.lastNavTime = now;
  }

  handleAButton(isModalOpen) {
    // A button = Select/Confirm
    if (isModalOpen) {
      // In modal, select focused stream
      const focused = document.activeElement;
      if (focused && focused.classList.contains('stream-item')) {
        focused.click();
      }
    } else {
      // In main view, select focused content
      if (window.DeckFlixApp.appState.focusedElement) {
        window.DeckFlixApp.appState.focusedElement.click();
      }
    }
  }

  handleBButton(isModalOpen) {
    // B button = Back/Cancel
    if (isModalOpen) {
      window.DeckFlixApp.closeStreamModal();
    } else {
      // In main view, clear search or refresh content
      if (window.DeckFlixApp && window.DeckFlixApp.elements) {
        const { elements } = window.DeckFlixApp;
        if (!elements.searchSection.classList.contains('hidden')) {
          // Clear search if active
          window.DeckFlixApp.clearSearch();
        } else {
          // Refresh current section
          this.refreshCurrentSection();
        }
      }
    }
  }

  handleXButton() {
    // X button = Refresh current section or toggle section view
    this.refreshCurrentSection();
  }

  handleYButton() {
    // Y button = Show content info or toggle between sections
    if (window.DeckFlixApp) {
      const { appState } = window.DeckFlixApp;
      if (appState.focusedElement) {
        this.showContentInfo(appState.focusedElement);
      }
    }
  }

  handleBumpers(button) {
    // LB/RB = Switch sections
    if (window.DeckFlixApp && window.DeckFlixApp.elements) {
      const { elements, appState } = window.DeckFlixApp;

      // Don't switch sections if search is active
      if (!elements.searchSection.classList.contains('hidden')) {
        return;
      }

      const sections = ['continue-watching', 'movies', 'series', 'anime'];
      let currentSectionIndex = sections.indexOf(appState.currentSection || 'movies');

      if (button === 'LB') {
        // Previous section
        currentSectionIndex = Math.max(0, currentSectionIndex - 1);
      } else if (button === 'RB') {
        // Next section
        currentSectionIndex = Math.min(sections.length - 1, currentSectionIndex + 1);
      }

      const newSection = sections[currentSectionIndex];
      this.focusSection(newSection);
    }
  }

  handleStartButton() {
    // Start = Focus search or toggle search
    if (window.DeckFlixApp && window.DeckFlixApp.elements) {
      const { elements } = window.DeckFlixApp;

      if (document.activeElement === elements.searchInput) {
        // If search is focused, clear it
        window.DeckFlixApp.clearSearch();
        this.focusFirstContent();
      } else {
        // Focus search input
        elements.searchInput.focus();
      }
    }
  }

  handleSelectButton() {
    // Select = Toggle between search and content, or show info
    if (window.DeckFlixApp) {
      const { elements, appState } = window.DeckFlixApp;

      if (document.activeElement === elements.searchInput) {
        // If search is focused, go to first content
        this.focusFirstContent();
      } else if (appState.focusedElement) {
        // Show content info
        this.showContentInfo(appState.focusedElement);
      } else {
        // Toggle controls overlay
        this.toggleControlsOverlay();
      }
    }
  }

  simulateKeyPress(key) {
    // Create and dispatch keyboard event
    const event = new KeyboardEvent('keydown', {
      key: key,
      code: key,
      bubbles: true,
      cancelable: true
    });

    document.dispatchEvent(event);
  }

  vibrate(gamepad, duration = 100) {
    // Haptic feedback (if supported)
    if (gamepad.vibrationActuator) {
      gamepad.vibrationActuator.playEffect('dual-rumble', {
        duration: duration,
        strongMagnitude: 0.3,
        weakMagnitude: 0.1
      }).catch(() => {
        // Vibration not supported, ignore
      });
    }
  }

  refreshCurrentSection() {
    // Trigger refresh for current section
    if (window.DeckFlixApp && window.DeckFlixApp.elements) {
      const { elements, appState } = window.DeckFlixApp;
      const currentSection = appState.currentSection || 'movies';

      // Find the retry button for the current section
      const retryBtn = document.querySelector(`[data-section="${currentSection}"]`);
      if (retryBtn) {
        retryBtn.click();
      } else {
        // Fallback: refresh all content
        if (elements.globalRetry) {
          elements.globalRetry.click();
        }
      }
    }
  }

  showContentInfo(contentElement) {
    if (!contentElement) return;

    // Extract content information from the element
    const title = contentElement.querySelector('.content-title')?.textContent || 'Unknown';
    const year = contentElement.querySelector('.content-year')?.textContent || 'Unknown';
    const rating = contentElement.querySelector('.content-rating')?.textContent || 'N/A';
    const contentType = contentElement.dataset.contentType || 'content';

    const info = `${title}\nType: ${contentType.toUpperCase()}\nYear: ${year}\nRating: ${rating}`;

    // Show temporary info overlay
    this.showTemporaryInfo(info);
  }

  showTemporaryInfo(text) {
    // Create temporary info overlay
    const overlay = document.createElement('div');
    overlay.style.cssText = `
      position: fixed;
      top: 20px;
      left: 20px;
      background: var(--bg-secondary);
      color: var(--text-primary);
      padding: 15px;
      border-radius: 8px;
      border: 2px solid var(--accent);
      z-index: 2000;
      max-width: 350px;
      font-size: 14px;
      line-height: 1.4;
      white-space: pre-line;
      box-shadow: 0 10px 25px rgba(0, 0, 0, 0.5);
    `;
    overlay.textContent = text;

    document.body.appendChild(overlay);

    // Remove after 3 seconds
    setTimeout(() => {
      if (document.body.contains(overlay)) {
        document.body.removeChild(overlay);
      }
    }, 3000);
  }

  toggleControlsOverlay() {
    const overlay = document.querySelector('.controls-overlay');
    if (overlay) {
      overlay.style.opacity = overlay.style.opacity === '0' ? '0.7' : '0';
    }
  }

  showControllerStatus(message) {
    // Show controller connection status
    const status = document.getElementById('addon-status');
    if (status) {
      const originalText = status.textContent;
      status.textContent = message;
      status.style.color = 'var(--accent)';

      // Restore original text after 2 seconds
      setTimeout(() => {
        status.textContent = originalText;
        status.style.color = '';
      }, 2000);
    }
  }

  // Check if any gamepads are connected
  isConnected() {
    return Object.keys(this.gamepads).length > 0;
  }

  // Get the first connected gamepad
  getGamepad() {
    const keys = Object.keys(this.gamepads);
    return keys.length > 0 ? this.gamepads[keys[0]] : null;
  }

  // Focus a specific section
  focusSection(sectionName) {
    if (!window.DeckFlixApp) return;

    const { elements, appState } = window.DeckFlixApp;
    appState.currentSection = sectionName;

    // Find the first content card in the target section
    let targetGrid;
    switch (sectionName) {
      case 'continue-watching':
        targetGrid = elements.continueWatchingGrid;
        break;
      case 'movies':
        targetGrid = elements.moviesGrid;
        break;
      case 'series':
        targetGrid = elements.seriesGrid;
        break;
      case 'anime':
        targetGrid = elements.animeGrid;
        break;
      default:
        targetGrid = elements.moviesGrid;
    }

    if (targetGrid && !targetGrid.classList.contains('hidden')) {
      const firstCard = targetGrid.querySelector('.content-card, .continue-watching-card');
      if (firstCard) {
        window.DeckFlixApp.setFocusedElement(firstCard);
        firstCard.focus();
        firstCard.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
      }
    }
  }

  // Focus the first available content
  focusFirstContent() {
    if (window.DeckFlixApp && window.DeckFlixApp.setFocusToFirstContent) {
      window.DeckFlixApp.setFocusToFirstContent();
    }
  }
}

// Initialize controller when DOM is loaded
let gamepadController;

window.addEventListener('DOMContentLoaded', () => {
  gamepadController = new GamepadController();
  console.log('Gamepad controller initialized');

  // Make it available globally for debugging
  window.GamepadController = gamepadController;
});

// Export for use in other files
window.DeckFlixGamepad = GamepadController;