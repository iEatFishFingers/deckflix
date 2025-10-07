// Tauri API will be initialized when ready - we'll access it dynamically

// Application state
let appState = {
  movies: [],
  series: [],
  anime: [],
  searchResults: [],
  continueWatching: [],
  currentContent: null,
  currentStreams: [],
  isLoading: false,
  isSearching: false,
  focusedElement: null,
  focusIndex: 0,
  currentSection: 'movies',
  searchQuery: '',
  searchTimeout: null,
  searchFilter: 'all'  // 'all', 'movies', 'series', 'anime'
};

// DOM elements
let elements = {};

// Continue watching storage key
const CONTINUE_WATCHING_KEY = 'deckflix_continue_watching';

// Safe Tauri invoke function with proper error handling
async function safeInvoke(command, args = {}) {
  try {
    // Check if Tauri is available
    if (typeof window.__TAURI__ === 'undefined') {
      throw new Error('Tauri API not available - app may not be running in Tauri context');
    }

    if (typeof window.__TAURI__.core === 'undefined') {
      throw new Error('Tauri core API not available');
    }

    if (typeof window.__TAURI__.core.invoke === 'undefined') {
      throw new Error('Tauri invoke function not available');
    }

    // Use the Tauri invoke function
    const result = await window.__TAURI__.core.invoke(command, args);
    return result;
  } catch (error) {
    DEBUG.error('TAURI_INVOKE', `Failed to invoke command: ${command}`, error);
    throw error;
  }
}

// Wait for Tauri to be ready
function waitForTauri(timeout = 10000) {
  return new Promise((resolve, reject) => {
    const startTime = Date.now();

    function checkTauri() {
      if (window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke) {
        DEBUG.log('TAURI_INIT', 'Tauri API is ready');
        resolve(true);
      } else if (Date.now() - startTime > timeout) {
        reject(new Error(`Tauri API not ready after ${timeout}ms`));
      } else {
        setTimeout(checkTauri, 100);
      }
    }

    checkTauri();
  });
}

// Debug logging utility
const DEBUG = {
  log: (category, message, data = null) => {
    const timestamp = new Date().toISOString();
    const logMessage = `[${timestamp}] [${category}] ${message}`;
    console.log(logMessage, data || '');

    // Add to debug panel if it exists
    if (window.debugPanel) {
      window.debugPanel.addLog(category, message, data);
    }
  },
  error: (category, message, error = null) => {
    const timestamp = new Date().toISOString();
    const logMessage = `[${timestamp}] [ERROR] [${category}] ${message}`;
    console.error(logMessage, error || '');

    // Add to debug panel if it exists
    if (window.debugPanel) {
      window.debugPanel.addError(category, message, error);
    }
  }
};

// Initialize the application
async function initApp() {
  DEBUG.log('APP_INIT', 'Starting DeckFlix application initialization');

  try {
    // Wait for Tauri to be ready before proceeding
    DEBUG.log('APP_INIT', 'Waiting for Tauri API to be ready...');
    await waitForTauri(10000);
    DEBUG.log('APP_INIT', 'Tauri API is ready, continuing initialization...');
    DEBUG.log('APP_INIT', 'Caching DOM elements...');
    // Cache DOM elements
    elements = {
      // Global elements
      globalLoading: document.getElementById('global-loading'),
      globalError: document.getElementById('global-error'),
      globalRetry: document.getElementById('global-retry'),
      addonStatus: document.getElementById('addon-status'),

      // Search elements
      searchInput: document.getElementById('search-input'),
      searchClear: document.getElementById('search-clear'),
      searchSection: document.getElementById('search-section'),
      searchGrid: document.getElementById('search-grid'),
      searchLoading: document.getElementById('search-loading'),
      searchError: document.getElementById('search-error'),
      searchResultsCount: document.getElementById('search-results-count'),
      searchTabs: document.querySelectorAll('.search-tab'),

      // Continue watching elements
      continueWatchingSection: document.getElementById('continue-watching-section'),
      continueWatchingGrid: document.getElementById('continue-watching-grid'),

      // Movies section
      moviesSection: document.getElementById('movies-section'),
      moviesGrid: document.getElementById('movies-grid'),
      moviesLoading: document.getElementById('movies-loading'),
      moviesError: document.getElementById('movies-error'),

      // Series section
      seriesSection: document.getElementById('series-section'),
      seriesGrid: document.getElementById('series-grid'),
      seriesLoading: document.getElementById('series-loading'),
      seriesError: document.getElementById('series-error'),

      // Anime section
      animeSection: document.getElementById('anime-section'),
      animeGrid: document.getElementById('anime-grid'),
      animeLoading: document.getElementById('anime-loading'),
      animeError: document.getElementById('anime-error'),

      // Modal elements
      streamModal: document.getElementById('stream-modal'),
      modalMovieTitle: document.getElementById('modal-movie-title'),
      streamsLoading: document.getElementById('streams-loading'),
      streamsList: document.getElementById('streams-list'),
      streamsError: document.getElementById('streams-error'),
      closeModal: document.getElementById('close-modal'),

      // Video player elements
      videoPlayerModal: document.getElementById('video-player-modal'),
      videoPlayer: document.getElementById('video-player'),
      videoTitle: document.getElementById('video-title'),
      videoLoading: document.getElementById('video-loading'),
      videoError: document.getElementById('video-error'),
      videoErrorMessage: document.getElementById('video-error-message'),
      closeVideo: document.getElementById('close-video'),
      retryVideo: document.getElementById('retry-video')
    };

    // Validate DOM elements
    const missingElements = [];
    for (const [key, element] of Object.entries(elements)) {
      if (!element) {
        missingElements.push(key);
      }
    }

    if (missingElements.length > 0) {
      DEBUG.error('APP_INIT', 'Missing DOM elements', missingElements);
    } else {
      DEBUG.log('APP_INIT', 'All DOM elements cached successfully');
    }

    DEBUG.log('APP_INIT', 'Setting up event listeners...');
    // Set up event listeners
    setupEventListeners();

    DEBUG.log('APP_INIT', 'Initializing focus management...');
    // Initialize focus management
    initializeFocusManagement();

    DEBUG.log('APP_INIT', 'Creating debug panel...');
    // Create debug panel
    createDebugPanel();

    DEBUG.log('APP_INIT', 'Checking addon status...');
    // Check addon status
    await checkAddonStatus();

    DEBUG.log('APP_INIT', 'Loading continue watching data...');
    // TEMPORARY: Clear localStorage to remove any cached bad data
    console.log('🧹 Clearing localStorage to fix ID caching issue...');
    localStorage.clear();
    // Load continue watching from localStorage
    loadContinueWatching();

    DEBUG.log('APP_INIT', 'Loading all content sections...');
    // Load all content sections
    await loadAllContent();

    DEBUG.log('APP_INIT', 'Application initialization complete');
  } catch (error) {
    DEBUG.error('APP_INIT', 'Failed to initialize application', error);
    showGlobalError('Application initialization failed: ' + error.message);
  }
}

function setupEventListeners() {
  // Search functionality
  elements.searchInput.addEventListener('input', handleSearchInput);
  elements.searchClear.addEventListener('click', clearSearch);

  // Search tabs
  elements.searchTabs.forEach(tab => {
    tab.addEventListener('click', (e) => {
      const filterType = e.target.dataset.tab;

      // Update active tab
      elements.searchTabs.forEach(t => t.classList.remove('active'));
      e.target.classList.add('active');

      // Update filter and redisplay results
      appState.searchFilter = filterType;
      displaySearchResults(appState.searchResults);
    });
  });

  // Retry buttons for each section
  document.querySelectorAll('.retry-btn').forEach(btn => {
    btn.addEventListener('click', (e) => {
      const section = e.target.dataset.section;
      retrySection(section);
    });
  });

  // Global retry
  elements.globalRetry.addEventListener('click', loadAllContent);

  // Modal close
  elements.closeModal.addEventListener('click', closeStreamModal);

  // Click outside modal to close
  elements.streamModal.addEventListener('click', (e) => {
    if (e.target === elements.streamModal) {
      closeStreamModal();
    }
  });

  // Video player event listeners
  elements.closeVideo.addEventListener('click', closeVideoPlayer);
  elements.retryVideo.addEventListener('click', () => {
    // Try external player as fallback
    if (window.currentStreamUrl) {
      tryExternalPlayer(window.currentStreamUrl);
    }
  });

  // Click outside video modal to close
  elements.videoPlayerModal.addEventListener('click', (e) => {
    if (e.target === elements.videoPlayerModal) {
      closeVideoPlayer();
    }
  });

  // Video player keyboard controls
  elements.videoPlayer.addEventListener('keydown', handleVideoKeyboard);

  // Keyboard navigation fallback
  document.addEventListener('keydown', handleKeyboard);
}

function initializeFocusManagement() {
  // Initialize focus system for controller navigation
  appState.focusedElement = null;
  appState.focusIndex = 0;
}

// Create debug panel
function createDebugPanel() {
  DEBUG.log('DEBUG_PANEL', 'Creating debug panel...');

  const debugPanel = document.createElement('div');
  debugPanel.id = 'debug-panel';
  debugPanel.className = 'debug-panel hidden';
  debugPanel.innerHTML = `
    <div class="debug-header">
      <h3>Debug Panel</h3>
      <button id="debug-toggle" class="debug-toggle">Toggle</button>
      <button id="debug-clear" class="debug-clear">Clear</button>
    </div>
    <div class="debug-content">
      <div class="debug-section">
        <h4>App State</h4>
        <div id="debug-state"></div>
      </div>
      <div class="debug-section">
        <h4>API Status</h4>
        <div id="debug-api"></div>
      </div>
      <div class="debug-section">
        <h4>Logs</h4>
        <div id="debug-logs"></div>
      </div>
    </div>
  `;

  document.body.appendChild(debugPanel);

  // Add debug panel to window for global access
  window.debugPanel = {
    element: debugPanel,
    isVisible: false,
    logs: [],

    addLog: (category, message, data) => {
      const log = { category, message, data, timestamp: new Date().toISOString() };
      window.debugPanel.logs.push(log);
      window.debugPanel.updateLogs();
    },

    addError: (category, message, error) => {
      const log = { category, message, error, timestamp: new Date().toISOString(), isError: true };
      window.debugPanel.logs.push(log);
      window.debugPanel.updateLogs();
    },

    updateState: () => {
      const stateDiv = document.getElementById('debug-state');
      if (stateDiv) {
        stateDiv.innerHTML = `
          <div>Movies loaded: ${appState.movies.length}</div>
          <div>Series loaded: ${appState.series.length}</div>
          <div>Anime loaded: ${appState.anime.length}</div>
          <div>Search results: ${appState.searchResults.length}</div>
          <div>Is loading: ${appState.isLoading}</div>
          <div>Current section: ${appState.currentSection}</div>
          <div>Focused element: ${appState.focusedElement ? appState.focusedElement.tagName : 'None'}</div>
        `;
      }
    },

    updateAPI: (status, details) => {
      const apiDiv = document.getElementById('debug-api');
      if (apiDiv) {
        apiDiv.innerHTML = `
          <div>Status: ${status}</div>
          <div>Details: ${details}</div>
          <div>Last update: ${new Date().toLocaleTimeString()}</div>
        `;
      }
    },

    updateLogs: () => {
      const logsDiv = document.getElementById('debug-logs');
      if (logsDiv) {
        const recentLogs = window.debugPanel.logs.slice(-20);
        logsDiv.innerHTML = recentLogs.map(log => {
          const time = new Date(log.timestamp).toLocaleTimeString();
          const errorClass = log.isError ? 'debug-error' : '';
          return `<div class="debug-log ${errorClass}">[${time}] [${log.category}] ${log.message}</div>`;
        }).join('');
        logsDiv.scrollTop = logsDiv.scrollHeight;
      }
    },

    toggle: () => {
      window.debugPanel.isVisible = !window.debugPanel.isVisible;
      if (window.debugPanel.isVisible) {
        debugPanel.classList.remove('hidden');
        window.debugPanel.updateState();
      } else {
        debugPanel.classList.add('hidden');
      }
    },

    clear: () => {
      window.debugPanel.logs = [];
      window.debugPanel.updateLogs();
    }
  };

  // Set up debug panel event listeners
  document.getElementById('debug-toggle').addEventListener('click', window.debugPanel.toggle);
  document.getElementById('debug-clear').addEventListener('click', window.debugPanel.clear);

  // Add keyboard shortcut to toggle debug panel (F12)
  document.addEventListener('keydown', (e) => {
    if (e.key === 'F12') {
      e.preventDefault();
      window.debugPanel.toggle();
    }
  });

  DEBUG.log('DEBUG_PANEL', 'Debug panel created successfully');
}

async function checkAddonStatus() {
  DEBUG.log('ADDON_STATUS', 'Checking addon status...');

  try {
    DEBUG.log('ADDON_STATUS', 'Invoking get_addon_status command...');
    const status = await safeInvoke('get_addon_status');

    DEBUG.log('ADDON_STATUS', 'Addon status received', { status });
    elements.addonStatus.textContent = status;
    elements.addonStatus.style.color = 'var(--success)';

    if (window.debugPanel) {
      window.debugPanel.updateAPI('Connected', status);
    }
  } catch (error) {
    DEBUG.error('ADDON_STATUS', 'Failed to get addon status', error);
    elements.addonStatus.textContent = 'Addon connection failed';
    elements.addonStatus.style.color = 'var(--error)';

    if (window.debugPanel) {
      window.debugPanel.updateAPI('Failed', error.message);
    }
  }
}

function showGlobalError(message) {
  DEBUG.error('GLOBAL', 'Showing global error', { message });

  if (elements.globalLoading) {
    elements.globalLoading.classList.add('hidden');
  }

  if (elements.globalError) {
    elements.globalError.classList.remove('hidden');
    const errorContent = elements.globalError.querySelector('.error-content p');
    if (errorContent) {
      errorContent.textContent = message;
    }
  }
}

// Search functionality
function handleSearchInput(e) {
  const query = e.target.value.trim();
  appState.searchQuery = query;

  // Show/hide clear button
  if (query) {
    elements.searchClear.classList.remove('hidden');
  } else {
    elements.searchClear.classList.add('hidden');
    hideSearchResults();
    return;
  }

  // Debounce search with 300ms delay
  if (appState.searchTimeout) {
    clearTimeout(appState.searchTimeout);
  }

  appState.searchTimeout = setTimeout(() => {
    performSearch(query);
  }, 300);
}

function clearSearch() {
  DEBUG.log('SEARCH', 'Clearing search and returning to main content');

  elements.searchInput.value = '';
  elements.searchClear.classList.add('hidden');
  appState.searchQuery = '';
  appState.searchResults = [];
  appState.isSearching = false;
  appState.searchFilter = 'all';

  // Reset tabs to 'all'
  elements.searchTabs.forEach(tab => {
    if (tab.dataset.tab === 'all') {
      tab.classList.add('active');
    } else {
      tab.classList.remove('active');
    }
  });

  // Clear search loading and error states
  elements.searchLoading.classList.add('hidden');
  elements.searchError.classList.add('hidden');
  elements.searchResultsCount.textContent = '';

  // Clear timeout if active
  if (appState.searchTimeout) {
    clearTimeout(appState.searchTimeout);
    appState.searchTimeout = null;
  }

  hideSearchResults();

  // Update debug panel
  if (window.debugPanel) {
    window.debugPanel.updateState();
    window.debugPanel.updateAPI('Search Cleared', 'Returned to main content');
  }

  DEBUG.log('SEARCH', 'Search cleared successfully');
}

function hideSearchResults() {
  elements.searchSection.classList.add('hidden');
  showContentSections();
}

function showSearchResults() {
  elements.searchSection.classList.remove('hidden');
  hideContentSections();
}

function hideContentSections() {
  elements.continueWatchingSection.classList.add('hidden');
  elements.moviesSection.classList.add('hidden');
  elements.seriesSection.classList.add('hidden');
  elements.animeSection.classList.add('hidden');
}

function showContentSections() {
  if (appState.continueWatching.length > 0) {
    elements.continueWatchingSection.classList.remove('hidden');
  }
  elements.moviesSection.classList.remove('hidden');
  elements.seriesSection.classList.remove('hidden');
  elements.animeSection.classList.remove('hidden');
}

async function performSearch(query) {
  DEBUG.log('SEARCH', `Starting comprehensive search for: "${query}"`);

  try {
    // Reset search filter to 'all' for new searches
    appState.searchFilter = 'all';
    elements.searchTabs.forEach(tab => {
      if (tab.dataset.tab === 'all') {
        tab.classList.add('active');
      } else {
        tab.classList.remove('active');
      }
    });

    // Show search loading
    DEBUG.log('SEARCH', 'Showing search loading UI');
    elements.searchLoading.classList.remove('hidden');
    elements.searchGrid.innerHTML = '';
    elements.searchError.classList.add('hidden');
    showSearchResults();

    // Update app state
    appState.isSearching = true;
    if (window.debugPanel) {
      window.debugPanel.updateState();
    }

    DEBUG.log('SEARCH', 'Invoking backend search command...');
    const startTime = Date.now();
    const results = await safeInvoke('search_content', { query });
    const duration = Date.now() - startTime;

    DEBUG.log('SEARCH', `Search completed in ${duration}ms`, {
      resultsCount: results.length,
      query: query,
      duration: `${duration}ms`
    });

    // Apply intelligent ranking to search results
    const rankedResults = processSearchResults(results, query);
    DEBUG.log('SEARCH', `Applied intelligent ranking`, {
      originalCount: results.length,
      rankedCount: rankedResults.length,
      topResults: rankedResults.slice(0, 3).map(r => ({ name: r.name, score: r.relevanceScore }))
    });

    appState.searchResults = rankedResults;
    appState.isSearching = false;

    // Hide loading
    elements.searchLoading.classList.add('hidden');

    if (rankedResults.length === 0) {
      DEBUG.log('SEARCH', 'No results found, showing error message');
      elements.searchError.classList.remove('hidden');
      elements.searchResultsCount.textContent = '';

      // Update error message to be more descriptive
      const errorMsg = elements.searchError.querySelector('p');
      if (errorMsg) {
        errorMsg.textContent = `No results found for "${query}". Try different keywords or check spelling.`;
      }
    } else {
      DEBUG.log('SEARCH', `Displaying ${rankedResults.length} ranked search results`);
      displaySearchResults(rankedResults);
      elements.searchResultsCount.textContent = `${rankedResults.length} result${rankedResults.length !== 1 ? 's' : ''} for "${query}"`;

      // Log content type breakdown
      const contentTypes = rankedResults.reduce((acc, result) => {
        acc[result.content_type] = (acc[result.content_type] || 0) + 1;
        return acc;
      }, {});
      DEBUG.log('SEARCH', 'Content type breakdown', contentTypes);
    }

    if (window.debugPanel) {
      window.debugPanel.updateState();
      window.debugPanel.updateAPI('Search Complete', `${rankedResults.length} results found for "${query}"`);
    }

  } catch (error) {
    DEBUG.error('SEARCH', 'Search failed', error);
    elements.searchLoading.classList.add('hidden');
    elements.searchError.classList.remove('hidden');
    elements.searchResultsCount.textContent = '';
    appState.isSearching = false;

    // Update error message with actual error
    const errorMsg = elements.searchError.querySelector('p');
    if (errorMsg) {
      errorMsg.textContent = `Search failed: ${error.message || 'Unknown error'}. Please try again.`;
    }

    if (window.debugPanel) {
      window.debugPanel.updateState();
      window.debugPanel.updateAPI('Search Failed', error.message || 'Unknown error');
    }
  }
}

// Search ranking and processing functions
function preprocessSearchTerm(searchTerm) {
  if (!searchTerm) return '';

  let processed = searchTerm.toLowerCase().trim();

  // Remove common articles
  processed = processed.replace(/^(the|a|an)\s+/i, '');

  // Handle plurals (simple approach)
  const pluralMap = {
    'cars': 'car',
    'movies': 'movie',
    'shows': 'show',
    'series': 'series'
  };

  if (pluralMap[processed]) {
    processed = pluralMap[processed];
  }

  return processed;
}

function calculateRelevanceScore(movie, searchTerm) {
  const title = (movie.name || movie.title || '').toLowerCase().trim();
  const originalSearch = searchTerm.toLowerCase().trim();
  const processedSearch = preprocessSearchTerm(searchTerm);

  DEBUG.log('RANKING', `Scoring "${title}" against search "${originalSearch}"`, {
    title,
    originalSearch,
    processedSearch
  });

  let score = 0;

  // Test both original and processed search terms
  const searchTerms = [originalSearch, processedSearch].filter(Boolean);

  for (const term of searchTerms) {
    // Exact match (highest priority)
    if (title === term) {
      score = Math.max(score, 100);
      DEBUG.log('RANKING', `Exact match found for "${title}" with "${term}"`);
      continue;
    }

    // Title starts with search term
    if (title.startsWith(term)) {
      score = Math.max(score, 90);
      DEBUG.log('RANKING', `Starts with match for "${title}" with "${term}"`);
      continue;
    }

    // Title starts with search term after "the "
    if (title.startsWith(`the ${term}`)) {
      score = Math.max(score, 90);
      DEBUG.log('RANKING', `Starts with "the" match for "${title}" with "${term}"`);
      continue;
    }

    // Contains as whole word at start
    if (title.startsWith(`${term} `)) {
      score = Math.max(score, 85);
      DEBUG.log('RANKING', `Word boundary start match for "${title}" with "${term}"`);
      continue;
    }

    // Contains as whole word with spaces
    if (title.includes(` ${term} `)) {
      score = Math.max(score, 80);
      DEBUG.log('RANKING', `Word boundary middle match for "${title}" with "${term}"`);
      continue;
    }

    // Contains anywhere in title
    if (title.includes(term)) {
      score = Math.max(score, 70);
      DEBUG.log('RANKING', `Contains match for "${title}" with "${term}"`);
      continue;
    }
  }

  // Franchise/sequel detection boost
  const franchiseBoost = detectFranchiseBoost(title, originalSearch);
  if (franchiseBoost > 0) {
    score += franchiseBoost;
    DEBUG.log('RANKING', `Franchise boost +${franchiseBoost} for "${title}"`);
  }

  DEBUG.log('RANKING', `Final score for "${title}": ${score}`);
  return score;
}

function detectFranchiseBoost(title, searchTerm) {
  const searchLower = searchTerm.toLowerCase();
  const titleLower = title.toLowerCase();

  // Detect numbered sequels
  const sequelPatterns = [
    new RegExp(`${searchLower}\s+\d+`, 'i'),        // "cars 2", "cars 3"
    new RegExp(`${searchLower}\s+ii+`, 'i'),        // "cars ii", "cars iii"
    new RegExp(`${searchLower}\s*:\s*`, 'i'),       // "cars: "
    new RegExp(`${searchLower}\s*-\s*`, 'i'),       // "cars - "
    new RegExp(`${searchLower}.*\s+(film|movie)`, 'i') // "cars film", "cars movie"
  ];

  for (const pattern of sequelPatterns) {
    if (pattern.test(titleLower)) {
      return 15; // Boost for sequels/related content
    }
  }

  return 0;
}

function getSecondaryScore(movie) {
  let score = 0;

  // Year score (prefer newer content, but not too heavily)
  const year = parseInt(movie.year) || 0;
  if (year > 0) {
    const currentYear = new Date().getFullYear();
    const yearDiff = currentYear - year;
    score += Math.max(0, 20 - (yearDiff * 0.5)); // Newer gets slight boost
  }

  // IMDB rating score
  const rating = parseFloat(movie.imdb_rating) || 0;
  if (rating > 0) {
    score += rating; // 0-10 points based on rating
  }

  // Content type preference (movies slightly preferred)
  switch (movie.content_type) {
    case 'movie': score += 2; break;
    case 'series': score += 1; break;
    case 'anime': score += 1; break;
  }

  return score;
}

function processSearchResults(results, searchTerm) {
  DEBUG.log('RANKING', `Processing ${results.length} search results for intelligent ranking`);

  if (!results || results.length === 0) {
    return [];
  }

  // Score each result
  const scoredResults = results.map((movie, index) => {
    const relevanceScore = calculateRelevanceScore(movie, searchTerm);
    const secondaryScore = getSecondaryScore(movie);

    return {
      ...movie,
      relevanceScore,
      secondaryScore,
      originalIndex: index
    };
  });

  // Filter out very low relevance results
  const filteredResults = scoredResults.filter(result => result.relevanceScore >= 10);

  DEBUG.log('RANKING', `Filtered from ${scoredResults.length} to ${filteredResults.length} results (relevance >= 10)`);

  // Sort by relevance score (highest first), then by secondary score
  filteredResults.sort((a, b) => {
    // Primary sort: relevance score
    if (b.relevanceScore !== a.relevanceScore) {
      return b.relevanceScore - a.relevanceScore;
    }

    // Secondary sort: secondary score (year, rating, content type)
    if (b.secondaryScore !== a.secondaryScore) {
      return b.secondaryScore - a.secondaryScore;
    }

    // Tertiary sort: original order (maintain stable sort)
    return a.originalIndex - b.originalIndex;
  });

  // Group franchise results together
  const groupedResults = groupFranchiseResults(filteredResults, searchTerm);

  DEBUG.log('RANKING', 'Ranking complete', {
    originalCount: results.length,
    filteredCount: filteredResults.length,
    finalCount: groupedResults.length,
    topResults: groupedResults.slice(0, 5).map(r => ({
      name: r.name,
      relevanceScore: r.relevanceScore,
      secondaryScore: r.secondaryScore,
      year: r.year
    }))
  });

  return groupedResults;
}

function groupFranchiseResults(results, searchTerm) {
  // Simple franchise grouping - group results that start with the same base title
  const groups = new Map();
  const standalone = [];

  const searchLower = searchTerm.toLowerCase();

  results.forEach(result => {
    const titleLower = (result.name || result.title || '').toLowerCase();

    // Check if this is part of a franchise
    let franchiseKey = null;

    // Look for franchise patterns
    if (titleLower.startsWith(searchLower)) {
      franchiseKey = searchLower;
    } else if (titleLower.includes(searchLower)) {
      // Extract potential franchise name
      const parts = titleLower.split(/[:\-]/);
      if (parts.length > 1 && parts[0].includes(searchLower)) {
        franchiseKey = parts[0].trim();
      }
    }

    if (franchiseKey) {
      if (!groups.has(franchiseKey)) {
        groups.set(franchiseKey, []);
      }
      groups.get(franchiseKey).push(result);
    } else {
      standalone.push(result);
    }
  });

  // Flatten groups back to array, with franchise items in chronological order
  const finalResults = [];

  // Add grouped franchise results first (highest relevance groups first)
  const sortedGroups = Array.from(groups.entries()).sort((a, b) => {
    const avgScoreA = a[1].reduce((sum, item) => sum + item.relevanceScore, 0) / a[1].length;
    const avgScoreB = b[1].reduce((sum, item) => sum + item.relevanceScore, 0) / b[1].length;
    return avgScoreB - avgScoreA;
  });

  sortedGroups.forEach(([franchiseKey, groupResults]) => {
    // Sort franchise items by year (oldest first for chronological order)
    groupResults.sort((a, b) => {
      const yearA = parseInt(a.year) || 9999;
      const yearB = parseInt(b.year) || 9999;
      return yearA - yearB;
    });

    finalResults.push(...groupResults);
  });

  // Add standalone results
  finalResults.push(...standalone);

  return finalResults;
}

// Content loading functions
async function loadAllContent() {
  DEBUG.log('CONTENT_LOAD', 'Starting to load all content sections...');

  try {
    // Show global loading
    DEBUG.log('CONTENT_LOAD', 'Showing global loading screen');
    elements.globalLoading.classList.remove('hidden');
    elements.globalError.classList.add('hidden');

    appState.isLoading = true;
    if (window.debugPanel) {
      window.debugPanel.updateState();
    }

    // Tauri is already ready at this point (checked in initApp)
    DEBUG.log('CONTENT_LOAD', 'Tauri invoke function available, proceeding with content loading');

    // Load all content types in parallel
    DEBUG.log('CONTENT_LOAD', 'Creating parallel loading promises...');
    const [moviesPromise, seriesPromise, animePromise] = [
      loadMovies(),
      loadSeries(),
      loadAnime()
    ];

    DEBUG.log('CONTENT_LOAD', 'Waiting for all content loading promises to settle...');
    const results = await Promise.allSettled([moviesPromise, seriesPromise, animePromise]);

    // Log results of each loading attempt
    results.forEach((result, index) => {
      const types = ['movies', 'series', 'anime'];
      if (result.status === 'fulfilled') {
        DEBUG.log('CONTENT_LOAD', `${types[index]} loaded successfully`);
      } else {
        DEBUG.error('CONTENT_LOAD', `${types[index]} loading failed`, result.reason);
      }
    });

    // Hide global loading
    DEBUG.log('CONTENT_LOAD', 'Hiding global loading screen');
    elements.globalLoading.classList.add('hidden');
    appState.isLoading = false;

    if (window.debugPanel) {
      window.debugPanel.updateState();
    }

    DEBUG.log('CONTENT_LOAD', 'All content loading completed');

  } catch (error) {
    DEBUG.error('CONTENT_LOAD', 'Failed to load content', error);
    elements.globalLoading.classList.add('hidden');
    showGlobalError('Failed to load content: ' + error.message);
    appState.isLoading = false;

    if (window.debugPanel) {
      window.debugPanel.updateState();
    }
  }
}

async function loadMovies(retryCount = 0) {
  const maxRetries = 2;
  DEBUG.log('MOVIES_LOAD', `Loading movies (attempt ${retryCount + 1}/${maxRetries + 1})`);

  try {
    DEBUG.log('MOVIES_LOAD', 'Setting up UI for movies loading...');
    elements.moviesLoading.classList.remove('hidden');
    elements.moviesGrid.classList.add('hidden');
    elements.moviesError.classList.add('hidden');

    DEBUG.log('MOVIES_LOAD', 'Invoking fetch_popular_movies command...');
    const startTime = Date.now();

    const movies = await safeInvoke('fetch_popular_movies');

    const endTime = Date.now();
    const duration = endTime - startTime;

    DEBUG.log('MOVIES_LOAD', `Movies received in ${duration}ms`, {
      count: movies.length,
      duration: `${duration}ms`,
      sampleMovies: movies.slice(0, 3).map(m => ({ id: m.id, name: m.name }))
    });

    if (movies.length === 0) {
      throw new Error('No movies available from the Cinemeta API');
    }

    // Validate movie data structure
    const invalidMovies = movies.filter(movie => !movie.id || !movie.name);
    if (invalidMovies.length > 0) {
      DEBUG.error('MOVIES_LOAD', 'Found invalid movie data', { invalidCount: invalidMovies.length });
    }

    DEBUG.log('MOVIES_LOAD', 'Updating app state with movies...');
    appState.movies = movies;

    DEBUG.log('MOVIES_LOAD', 'Rendering movie cards...');
    displayContent(movies, elements.moviesGrid, 'movie');

    DEBUG.log('MOVIES_LOAD', 'Showing movies grid...');
    elements.moviesLoading.classList.add('hidden');
    elements.moviesGrid.classList.remove('hidden');

    // Clear any previous error state
    appState.moviesRetryCount = 0;

    if (window.debugPanel) {
      window.debugPanel.updateState();
      window.debugPanel.updateAPI('Movies Loaded', `${movies.length} movies loaded successfully`);
    }

    DEBUG.log('MOVIES_LOAD', 'Movies loading completed successfully');

  } catch (error) {
    DEBUG.error('MOVIES_LOAD', `Failed to load movies on attempt ${retryCount + 1}`, error);
    elements.moviesLoading.classList.add('hidden');

    // Auto-retry with exponential backoff
    if (retryCount < maxRetries) {
      const delay = Math.pow(2, retryCount) * 1000; // 1s, 2s, 4s
      DEBUG.log('MOVIES_LOAD', `Retrying movie fetch in ${delay}ms...`);

      setTimeout(() => {
        loadMovies(retryCount + 1);
      }, delay);
    } else {
      // Show error with detailed message
      DEBUG.error('MOVIES_LOAD', 'All retry attempts failed, showing error UI');
      elements.moviesError.classList.remove('hidden');

      const errorMsg = elements.moviesError.querySelector('p');
      if (errorMsg) {
        errorMsg.textContent = `Unable to load movies from Cinemeta API. ${error.message || 'Please check your internet connection.'}`;
      }

      appState.moviesRetryCount = retryCount;

      if (window.debugPanel) {
        window.debugPanel.updateAPI('Movies Failed', error.message || 'Unknown error');
        window.debugPanel.updateState();
      }
    }
  }
}

async function loadSeries() {
  try {
    elements.seriesLoading.classList.remove('hidden');
    elements.seriesGrid.classList.add('hidden');
    elements.seriesError.classList.add('hidden');

    console.log('Fetching popular series...');
    const series = await safeInvoke('fetch_popular_series');
    console.log('Series received:', series.length);

    appState.series = series;
    displayContent(series, elements.seriesGrid, 'series');

    elements.seriesLoading.classList.add('hidden');
    elements.seriesGrid.classList.remove('hidden');

  } catch (error) {
    console.error('Failed to load series:', error);
    elements.seriesLoading.classList.add('hidden');
    elements.seriesError.classList.remove('hidden');
  }
}

async function loadAnime() {
  try {
    elements.animeLoading.classList.remove('hidden');
    elements.animeGrid.classList.add('hidden');
    elements.animeError.classList.add('hidden');

    console.log('Fetching popular anime...');
    const anime = await safeInvoke('fetch_popular_anime');
    console.log('Anime received:', anime.length);

    appState.anime = anime;
    displayContent(anime, elements.animeGrid, 'anime');

    elements.animeLoading.classList.add('hidden');
    elements.animeGrid.classList.remove('hidden');

  } catch (error) {
    console.error('Failed to load anime:', error);
    elements.animeLoading.classList.add('hidden');
    elements.animeError.classList.remove('hidden');
  }
}

async function retrySection(section) {
  switch (section) {
    case 'movies':
      await loadMovies();
      break;
    case 'series':
      await loadSeries();
      break;
    case 'anime':
      await loadAnime();
      break;
  }
}

function displayContent(items, gridElement, contentType) {
  DEBUG.log('UI_RENDER', `Rendering ${contentType} content`, {
    itemCount: items.length,
    gridElementId: gridElement.id,
    contentType
  });

  // Validate grid element
  if (!gridElement) {
    DEBUG.error('UI_RENDER', 'Grid element is null or undefined', { contentType });
    return;
  }

  // Clear existing content
  DEBUG.log('UI_RENDER', `Clearing existing content from ${gridElement.id}`);
  gridElement.innerHTML = '';

  // Create content cards
  let successCount = 0;
  let errorCount = 0;

  items.forEach((item, index) => {
    try {
      DEBUG.log('UI_RENDER', `Creating card ${index + 1}/${items.length}`, {
        itemId: item.id,
        itemName: item.name,
        contentType
      });

      const card = createContentCard(item, index, contentType);
      if (card) {
        gridElement.appendChild(card);
        successCount++;
      } else {
        DEBUG.error('UI_RENDER', `Failed to create card for item ${index}`, item);
        errorCount++;
      }
    } catch (error) {
      DEBUG.error('UI_RENDER', `Error creating card for item ${index}`, { item, error });
      errorCount++;
    }
  });

  DEBUG.log('UI_RENDER', `Completed rendering ${contentType}`, {
    total: items.length,
    success: successCount,
    errors: errorCount,
    gridElementId: gridElement.id
  });

  // Verify cards were actually added to DOM
  const actualCardCount = gridElement.children.length;
  if (actualCardCount !== successCount) {
    DEBUG.error('UI_RENDER', 'Card count mismatch', {
      expected: successCount,
      actual: actualCardCount
    });
  }

  if (window.debugPanel) {
    window.debugPanel.updateState();
  }
}

function displaySearchResults(results) {
  DEBUG.log('SEARCH_DISPLAY', `Starting to display ${results.length} search results`);

  // Validate and filter search results first
  const validResults = validateAndFilterSearchResults(results);
  DEBUG.log('SEARCH_DISPLAY', `Filtered to ${validResults.length} valid results from ${results.length} total`);

  // Filter by tab selection
  let filteredResults = validResults;
  if (appState.searchFilter !== 'all') {
    filteredResults = validResults.filter(result => {
      // IMPORTANT: Rust serializes content_type as "type" in JSON
      const contentType = result.type || 'movie';
      // Map 'movies' to 'movie' for comparison
      const filterType = appState.searchFilter === 'movies' ? 'movie' : appState.searchFilter;
      return contentType === filterType;
    });
    DEBUG.log('SEARCH_DISPLAY', `Tab filter '${appState.searchFilter}' reduced results to ${filteredResults.length}`);
  }

  elements.searchGrid.innerHTML = '';

  let successCount = 0;
  let fallbackCount = 0;
  let skippedCount = 0;
  let animeCount = 0;
  let movieCount = 0;
  let seriesCount = 0;

  filteredResults.forEach((result, index) => {
    try {
      DEBUG.log('SEARCH_DISPLAY', `Processing result ${index + 1}/${validResults.length}`, {
        name: result.name || result.title,
        contentType: result.content_type,
        id: result.id,
        hasMinimumData: !!(result.name || result.title)
      });

      const card = createSearchResultCard(result, index);
      if (card) {
        elements.searchGrid.appendChild(card);

        if (card.classList.contains('fallback-card')) {
          fallbackCount++;
        } else {
          successCount++;
        }

        // Count content types
        const cardType = result.content_type || 'movie';
        switch (cardType) {
          case 'anime': animeCount++; break;
          case 'movie': movieCount++; break;
          case 'series': seriesCount++; break;
        }
      } else {
        DEBUG.error('SEARCH_DISPLAY', `Card creation returned null for result ${index}`);
        skippedCount++;
      }
    } catch (error) {
      DEBUG.error('SEARCH_DISPLAY', `Failed to process result ${index}`, { result, error });
      skippedCount++;
    }
  });

  const totalDisplayed = successCount + fallbackCount;

  DEBUG.log('SEARCH_DISPLAY', 'Search results display complete', {
    originalTotal: results.length,
    filteredTotal: validResults.length,
    displayed: totalDisplayed,
    successful: successCount,
    fallback: fallbackCount,
    skipped: skippedCount,
    breakdown: { anime: animeCount, movies: movieCount, series: seriesCount }
  });

  // Update UI with results count
  if (totalDisplayed === 0) {
    DEBUG.log('SEARCH_DISPLAY', 'No cards were displayed, showing error state');
    elements.searchError.classList.remove('hidden');
    const errorMsg = elements.searchError.querySelector('p');
    if (errorMsg) {
      errorMsg.textContent = `Found ${results.length} results but none could be displayed. Please try a different search.`;
    }
  }

  // Update debug panel if available
  if (window.debugPanel) {
    window.debugPanel.updateState();
  }
}

function validateAndFilterSearchResults(results) {
  DEBUG.log('SEARCH_VALIDATE', `Validating ${results.length} search results`);

  const validResults = [];
  let invalidCount = 0;
  const issues = [];

  results.forEach((result, index) => {
    const validationIssues = [];

    // Check for basic required data
    if (!result) {
      validationIssues.push('Result is null/undefined');
    } else {
      // Check for at least one identifier
      if (!result.id && !result.imdb_id) {
        validationIssues.push('No ID field');
      }

      // Check for at least one name field
      if (!result.name && !result.title) {
        validationIssues.push('No name/title field');
      }

      // Log data structure for debugging
      DEBUG.log('SEARCH_VALIDATE', `Result ${index} structure`, {
        keys: result ? Object.keys(result) : [],
        hasId: !!(result?.id || result?.imdb_id),
        hasName: !!(result?.name || result?.title),
        hasPoster: !!result?.poster,
        contentType: result?.content_type || 'unknown'
      });
    }

    if (validationIssues.length === 0) {
      validResults.push(result);
    } else {
      invalidCount++;
      issues.push(`Result ${index}: ${validationIssues.join(', ')}`);
      DEBUG.log('SEARCH_VALIDATE', `Result ${index} invalid: ${validationIssues.join(', ')}`, result);
    }
  });

  DEBUG.log('SEARCH_VALIDATE', 'Validation complete', {
    original: results.length,
    valid: validResults.length,
    invalid: invalidCount,
    issues: issues.slice(0, 5) // Show first 5 issues
  });

  return validResults;
}

function createSearchResultCard(result, index) {
  DEBUG.log('CARD_CREATE', `Creating search result card for index ${index}`, {
    rawResult: result,
    hasId: !!result.id,
    hasName: !!result.name,
    hasTitle: !!result.title,
    hasPoster: !!result.poster,
    contentType: result.content_type,
    resultKeys: Object.keys(result)
  });

  try {
    // Provide robust fallbacks for all required fields
    const cardId = result.id || result.imdb_id || `search-result-${index}`;
    const cardName = result.name || result.title || "Unknown Title";
    const cardPoster = result.poster;
    const cardYear = result.year || result.releaseInfo?.split('-')[0] || "N/A";
    const cardRating = result.imdb_rating || result.rating || null;
    const cardType = result.content_type || "movie";
    const cardDescription = result.description || null;

    DEBUG.log('CARD_CREATE', `Using processed data for card ${index}`, {
      id: cardId,
      name: cardName,
      poster: cardPoster,
      year: cardYear,
      rating: cardRating,
      type: cardType,
      hasDescription: !!cardDescription
    });

    const card = document.createElement('div');
    card.className = 'content-card focusable search-result-card';
    card.dataset.contentIndex = index;
    card.dataset.contentType = cardType;
    card.dataset.cardId = cardId;
    card.tabIndex = 0;

    // Enhanced poster with robust error handling
    const poster = document.createElement('div');
    poster.className = 'content-poster search-result-poster';

    // Always start with a placeholder
    poster.innerHTML = `
      <div class="no-image-placeholder">
        <div class="content-icon">${getContentIcon(cardType)}</div>
        <div class="no-image-text">No Image</div>
      </div>
    `;

    if (cardPoster && cardPoster.trim()) {
      DEBUG.log('CARD_CREATE', `Loading poster for card ${index}: ${cardPoster}`);

      const img = document.createElement('img');
      img.className = 'poster-image';
      img.src = cardPoster;
      img.alt = cardName;
      img.style.display = 'none'; // Hide until loaded

      img.onload = () => {
        try {
          DEBUG.log('CARD_CREATE', `Poster loaded successfully for card ${index}`);
          poster.innerHTML = '';
          img.style.display = 'block';
          img.style.opacity = '0';
          poster.appendChild(img);
          setTimeout(() => {
            img.style.transition = 'opacity 0.3s ease';
            img.style.opacity = '1';
          }, 50);
        } catch (error) {
          DEBUG.error('CARD_CREATE', `Error displaying loaded poster for card ${index}`, error);
        }
      };

      img.onerror = () => {
        DEBUG.log('CARD_CREATE', `Poster failed to load for card ${index}, keeping placeholder`);
        // Keep the placeholder that's already there
      };
    } else {
      DEBUG.log('CARD_CREATE', `No poster URL for card ${index}, using placeholder`);
    }

    // Enhanced info section with robust data handling
    const info = document.createElement('div');
    info.className = 'content-info search-result-info';

    // Content type indicator with anime-specific styling
    const typeIndicator = document.createElement('div');
    typeIndicator.className = `content-type ${cardType === 'anime' ? 'content-type-anime' : ''}`;
    typeIndicator.textContent = cardType.toUpperCase();

    const title = document.createElement('div');
    title.className = 'content-title search-result-title';
    title.textContent = cardName;
    title.title = cardName;

    const details = document.createElement('div');
    details.className = 'content-details search-result-details';

    const year = document.createElement('span');
    year.className = 'content-year';
    year.textContent = cardYear;

    const rating = document.createElement('span');
    rating.className = 'content-rating';
    rating.textContent = cardRating ? `★ ${cardRating}` : 'N/A';

    details.appendChild(year);
    details.appendChild(rating);

    info.appendChild(typeIndicator);
    info.appendChild(title);
    info.appendChild(details);

    card.appendChild(poster);
    card.appendChild(info);

    // Enhanced tooltip with description
    if (cardDescription) {
      card.title = `${cardName} (${cardType.toUpperCase()})\n${cardDescription.substring(0, 200)}${cardDescription.length > 200 ? '...' : ''}`;
    } else {
      card.title = `${cardName} (${cardType.toUpperCase()})`;
    }

    // Add click listener with robust content object
    const contentObject = {
      id: cardId,
      name: cardName,
      poster: cardPoster,
      year: cardYear,
      imdb_rating: cardRating,
      content_type: cardType,
      description: cardDescription
    };

    card.addEventListener('click', () => {
      DEBUG.log('CARD_CLICK', `Clicked on search result card: ${cardName}`, contentObject);
      selectContent(contentObject, cardType);
    });

    // Add focus listener
    card.addEventListener('focus', () => {
      setFocusedElement(card);
    });

    DEBUG.log('CARD_CREATE', `Successfully created card ${index} for "${cardName}"`);
    return card;

  } catch (error) {
    DEBUG.error('CARD_CREATE', `Failed to create card for index ${index}`, {
      error: error.message,
      stack: error.stack,
      result: result
    });

    // Create a minimal fallback card
    return createFallbackCard(result, index);
  }
}

function createFallbackCard(result, index) {
  DEBUG.log('CARD_FALLBACK', `Creating fallback card for index ${index}`);

  try {
    const card = document.createElement('div');
    card.className = 'content-card focusable search-result-card fallback-card';
    card.dataset.contentIndex = index;
    card.dataset.contentType = 'movie';
    card.tabIndex = 0;

    const poster = document.createElement('div');
    poster.className = 'content-poster';
    poster.innerHTML = `
      <div class="no-image-placeholder">
        <div class="content-icon">🎬</div>
        <div class="no-image-text">No Image</div>
      </div>
    `;

    const info = document.createElement('div');
    info.className = 'content-info';

    const title = document.createElement('div');
    title.className = 'content-title';
    title.textContent = result?.name || result?.title || `Unknown Title ${index}`;

    const details = document.createElement('div');
    details.className = 'content-details';
    details.innerHTML = '<span class="content-year">N/A</span><span class="content-rating">N/A</span>';

    info.appendChild(title);
    info.appendChild(details);
    card.appendChild(poster);
    card.appendChild(info);

    DEBUG.log('CARD_FALLBACK', `Fallback card created successfully for index ${index}`);
    return card;

  } catch (fallbackError) {
    DEBUG.error('CARD_FALLBACK', `Even fallback card creation failed for index ${index}`, fallbackError);
    return null;
  }
}

function displayContinueWatching() {
  if (appState.continueWatching.length === 0) {
    elements.continueWatchingSection.classList.add('hidden');
    return;
  }

  elements.continueWatchingGrid.innerHTML = '';

  appState.continueWatching.forEach((item, index) => {
    const card = createContinueWatchingCard(item, index);
    elements.continueWatchingGrid.appendChild(card);
  });

  elements.continueWatchingSection.classList.remove('hidden');
}

function createContentCard(content, index, contentType) {
  console.log(`📝 Creating card ${index}:`, {
    id: content.id,
    name: content.name,
    contentType: contentType
  });

  const card = document.createElement('div');
  card.className = 'content-card focusable movie-card';
  card.dataset.contentIndex = index;
  card.dataset.contentType = contentType;
  card.dataset.contentId = content.id; // Store the ID
  card.tabIndex = 0;

  // Content poster with enhanced loading
  const poster = document.createElement('div');
  poster.className = 'content-poster movie-poster';

  // Loading placeholder
  const loadingPlaceholder = document.createElement('div');
  loadingPlaceholder.className = 'image-loading';
  loadingPlaceholder.innerHTML = `
    <div class="loading-spinner-small"></div>
    <div class="loading-text">Loading...</div>
  `;
  poster.appendChild(loadingPlaceholder);

  if (content.poster) {
    const img = document.createElement('img');
    img.className = 'poster-image';
    img.src = content.poster;
    img.alt = content.name;

    // Enhanced image loading with fade-in effect
    img.onload = () => {
      poster.removeChild(loadingPlaceholder);
      img.style.opacity = '0';
      poster.appendChild(img);
      setTimeout(() => {
        img.style.transition = 'opacity 0.3s ease';
        img.style.opacity = '1';
      }, 50);
    };

    img.onerror = () => {
      poster.removeChild(loadingPlaceholder);
      poster.innerHTML = `
        <div class="no-image-placeholder">
          <div class="content-icon">${getContentIcon(contentType)}</div>
          <div class="no-image-text">No Image</div>
        </div>
      `;
    };
  } else {
    poster.removeChild(loadingPlaceholder);
    poster.innerHTML = `
      <div class="no-image-placeholder">
        <div class="content-icon">${getContentIcon(contentType)}</div>
        <div class="no-image-text">No Image</div>
      </div>
    `;
  }

  // Content info with improved layout
  const info = document.createElement('div');
  info.className = 'content-info movie-info';

  // Content type indicator
  const typeIndicator = document.createElement('div');
  typeIndicator.className = 'content-type';
  typeIndicator.textContent = contentType.toUpperCase();

  const title = document.createElement('div');
  title.className = 'content-title movie-title';
  title.textContent = content.name;
  title.title = content.name;

  const details = document.createElement('div');
  details.className = 'content-details movie-details';

  const year = document.createElement('span');
  year.className = 'content-year movie-year';
  year.textContent = content.year || 'Unknown';

  const rating = document.createElement('span');
  rating.className = 'content-rating movie-rating';
  const ratingValue = content.imdb_rating || content.mal_rating;
  rating.textContent = ratingValue ? `★ ${ratingValue}` : 'N/A';

  details.appendChild(year);
  details.appendChild(rating);

  info.appendChild(typeIndicator);
  info.appendChild(title);
  info.appendChild(details);

  card.appendChild(poster);
  card.appendChild(info);

  // Enhanced hover effect with description preview
  if (content.description) {
    card.title = `${content.name}\n${content.description.substring(0, 200)}${content.description.length > 200 ? '...' : ''}`;
  }

  // Add click listener
  card.addEventListener('click', () => selectContent(content, contentType));

  // Add focus listener
  card.addEventListener('focus', () => {
    setFocusedElement(card);
  });

  return card;
}

function createContinueWatchingCard(item, index) {
  const card = document.createElement('div');
  card.className = 'continue-watching-card focusable';
  card.dataset.contentIndex = index;
  card.dataset.contentType = item.content_type;
  card.tabIndex = 0;

  // Content poster with progress
  const poster = document.createElement('div');
  poster.className = 'content-poster';

  // Progress bar background
  const progressBg = document.createElement('div');
  progressBg.className = 'progress-background';

  // Progress bar
  const progressBar = document.createElement('div');
  progressBar.className = 'progress-bar';
  progressBar.style.width = `${(item.progress || 0) * 100}%`;

  progressBg.appendChild(progressBar);

  if (item.poster) {
    const img = document.createElement('img');
    img.src = item.poster;
    img.alt = item.name;
    img.onerror = () => {
      poster.innerHTML = getContentIcon(item.content_type) + ' No Image';
      poster.appendChild(progressBg);
    };
    poster.appendChild(img);
  } else {
    poster.innerHTML = getContentIcon(item.content_type) + ' No Image';
  }

  poster.appendChild(progressBg);

  // Content info
  const info = document.createElement('div');
  info.className = 'content-info';

  const title = document.createElement('div');
  title.className = 'content-title';
  title.textContent = item.name;
  title.title = item.name;

  const progress = document.createElement('div');
  progress.className = 'content-details';
  progress.style.fontSize = '11px';
  progress.textContent = `${Math.round((item.progress || 0) * 100)}% watched`;

  info.appendChild(title);
  info.appendChild(progress);

  card.appendChild(poster);
  card.appendChild(info);

  // Add click listener
  card.addEventListener('click', () => selectContent(item, item.content_type));

  // Add focus listener
  card.addEventListener('focus', () => {
    setFocusedElement(card);
  });

  return card;
}

function getContentIcon(contentType) {
  switch (contentType) {
    case 'movie': return '🎬';
    case 'series': return '📺';
    case 'anime': return '🌸';
    default: return '🎬';
  }
}

// Continue watching functionality
function loadContinueWatching() {
  try {
    const stored = localStorage.getItem(CONTINUE_WATCHING_KEY);
    if (stored) {
      appState.continueWatching = JSON.parse(stored);
      displayContinueWatching();
    }
  } catch (error) {
    console.error('Failed to load continue watching:', error);
    appState.continueWatching = [];
  }
}

function saveContinueWatching() {
  try {
    localStorage.setItem(CONTINUE_WATCHING_KEY, JSON.stringify(appState.continueWatching));
  } catch (error) {
    console.error('Failed to save continue watching:', error);
  }
}

function addToContinueWatching(content, contentType, progress = 0) {
  const existingIndex = appState.continueWatching.findIndex(item => item.id === content.id);

  const continueItem = {
    id: content.id,
    name: content.name,
    poster: content.poster,
    content_type: contentType,
    progress: progress,
    last_watched: new Date().toISOString()
  };

  if (existingIndex >= 0) {
    appState.continueWatching[existingIndex] = continueItem;
  } else {
    appState.continueWatching.unshift(continueItem);
    // Keep only the latest 10 items
    appState.continueWatching = appState.continueWatching.slice(0, 10);
  }

  saveContinueWatching();
  displayContinueWatching();
}

function setFocusToFirstContent() {
  // Try to focus the first content card in any visible grid
  const grids = [elements.searchGrid, elements.continueWatchingGrid, elements.moviesGrid, elements.seriesGrid, elements.animeGrid];

  for (const grid of grids) {
    if (grid && !grid.classList.contains('hidden')) {
      const firstCard = grid.querySelector('.content-card, .continue-watching-card, .movie-card');
      if (firstCard) {
        setFocusedElement(firstCard);
        firstCard.focus();
        appState.focusIndex = 0;
        return true;
      }
    }
  }
  return false;
}

function setFocusedElement(element) {
  // Remove focus from previous element
  if (appState.focusedElement) {
    appState.focusedElement.classList.remove('focused');
  }

  // Set new focused element
  appState.focusedElement = element;
  if (element) {
    element.classList.add('focused');
    appState.focusIndex = parseInt(element.dataset.contentIndex) || 0;
  }
}

async function selectContent(content, contentType) {
  // EXTREME DEBUGGING: Log everything
  console.error('========== SELECT CONTENT CALLED ==========');
  console.error('typeof content:', typeof content);
  console.error('content keys:', Object.keys(content));
  console.error('content.id VALUE:', content.id);
  console.error('content.name VALUE:', content.name);
  console.error('Full content:', JSON.parse(JSON.stringify(content)));
  console.error('==========================================');

  appState.currentContent = content;
  console.log('🎬 Selected content:', content.name, 'Type:', contentType);
  console.log('🔍 Content object:', content);
  console.log('🆔 IMDB ID:', content.id);
  console.log('📊 Full content data:', JSON.stringify(content, null, 2));

  // Add to continue watching
  addToContinueWatching(content, contentType, 0);

  // Show modal
  elements.modalMovieTitle.textContent = content.name;
  elements.streamModal.classList.remove('hidden');

  // Show loading in modal
  elements.streamsLoading.classList.remove('hidden');
  elements.streamsList.classList.add('hidden');
  elements.streamsError.classList.add('hidden');

  try {
    console.log('Fetching streams for:', content.id);
    const streams = await safeInvoke('fetch_streams', { imdbId: content.id });

    console.log('Streams received:', streams.length);
    appState.currentStreams = streams;

    if (streams.length === 0) {
      throw new Error('No streams available');
    }

    displayStreams(streams);

  } catch (error) {
    console.error('Failed to load streams:', error);
    showStreamsError();
  }
}

function displayStreams(streams) {
  // Hide loading
  elements.streamsLoading.classList.add('hidden');

  // Clear existing streams
  elements.streamsList.innerHTML = '';

  // Create stream items
  streams.forEach((stream, index) => {
    const streamItem = createStreamItem(stream, index);
    elements.streamsList.appendChild(streamItem);
  });

  // Show streams list
  elements.streamsList.classList.remove('hidden');

  // Focus first stream
  const firstStream = elements.streamsList.querySelector('.stream-item');
  if (firstStream) {
    firstStream.focus();
  }
}

function createStreamItem(stream, index) {
  const item = document.createElement('div');
  item.className = 'stream-item focusable';
  item.tabIndex = 0;
  item.dataset.streamIndex = index;

  const title = document.createElement('div');
  title.className = 'stream-title';
  title.textContent = stream.title || stream.name || 'Unknown Stream';

  const quality = document.createElement('div');
  quality.className = 'stream-quality';
  quality.textContent = extractQuality(stream.title) || 'Quality: Unknown';

  item.appendChild(title);
  item.appendChild(quality);

  // Add click listener
  item.addEventListener('click', () => {
    console.log('Stream item clicked:', stream.title);
    playStream(stream);
  });

  return item;
}

function extractQuality(title) {
  if (!title) return null;

  // Common quality indicators
  const qualityPatterns = [
    /4K|2160p/i,
    /1080p/i,
    /720p/i,
    /480p/i,
    /CAM/i,
    /TS/i,
    /HDRip/i,
    /BluRay/i,
    /WEBRip/i
  ];

  for (const pattern of qualityPatterns) {
    const match = title.match(pattern);
    if (match) {
      return `Quality: ${match[0]}`;
    }
  }

  return null;
}

async function playStream(stream) {
  try {
    console.log('═══════════════════════════════════════════════════════════');
    console.log('🎯 [PLAY_STREAM] Playing stream:', stream.title);
    console.log('🔗 [PLAY_STREAM] Stream URL:', stream.url);
    console.log('🎬 [PLAY_STREAM] Movie:', appState.currentContent?.name);
    console.log('🆔 [PLAY_STREAM] Movie IMDB ID:', appState.currentContent?.id);
    console.log('📦 [PLAY_STREAM] Full stream object:', JSON.stringify(stream, null, 2));
    console.log('═══════════════════════════════════════════════════════════');

    // Store current stream URL for retry functionality
    window.currentStreamUrl = stream.url;

    // Check if it's a direct video URL that can be played in HTML5 video
    if (stream.url && (stream.url.startsWith('http://') || stream.url.startsWith('https://')) &&
        !stream.url.startsWith('magnet:') &&
        (stream.url.includes('.mp4') || stream.url.includes('.mkv') || stream.url.includes('.webm') || stream.url.includes('video'))) {
      console.log('🎬 Using built-in video player for direct video URL');
      playWithBuiltInPlayer(stream);
    } else if (stream.url && stream.url.startsWith('magnet:')) {
      console.log('🧲 Using external player for magnet link');
      tryExternalPlayer(stream.url);
    } else {
      console.log('❓ Unknown stream type, trying external player');
      console.log('🔍 URL details:', {
        url: stream.url,
        startsWithHttp: stream.url?.startsWith('http'),
        includesVideo: stream.url?.includes('video'),
        includesMp4: stream.url?.includes('.mp4')
      });
      tryExternalPlayer(stream.url);
    }

    // Close stream selection modal
    closeStreamModal();

  } catch (error) {
    console.error('Failed to play stream:', error);
    alert('Failed to launch video player: ' + (error.message || error));
  }
}

// Built-in HTML5 video player function
function playWithBuiltInPlayer(stream) {
  console.log('Opening built-in video player for:', stream.url);

  // Set video title to show both movie and stream quality
  const movieTitle = appState.currentContent ? appState.currentContent.name : 'Unknown Movie';
  const streamTitle = stream.title || '';
  elements.videoTitle.textContent = `${movieTitle} - ${streamTitle}`;

  // Show video player modal
  elements.videoPlayerModal.classList.remove('hidden');

  // Show loading state
  elements.videoLoading.classList.remove('hidden');
  elements.videoError.classList.add('hidden');
  elements.videoPlayer.classList.add('hidden');

  // IMPORTANT: Clear the video source first to prevent browser caching
  // Then set new source and force reload
  elements.videoPlayer.src = '';
  elements.videoPlayer.load(); // Force clear

  // Now set the new video source
  elements.videoPlayer.src = stream.url;
  elements.videoPlayer.load(); // Force reload with new source

  // Video event handlers
  elements.videoPlayer.onloadstart = () => {
    console.log('Video loading started');
    elements.videoLoading.classList.remove('hidden');
  };

  elements.videoPlayer.oncanplay = () => {
    console.log('Video can start playing');
    elements.videoLoading.classList.add('hidden');
    elements.videoPlayer.classList.remove('hidden');
    elements.videoPlayer.focus();
  };

  elements.videoPlayer.onerror = (e) => {
    console.error('Video playback error:', e);
    showVideoError('This video format is not supported by the built-in player.');
  };

  elements.videoPlayer.onended = () => {
    console.log('Video playback ended');
    closeVideoPlayer();
  };

  // Attempt to load and play
  elements.videoPlayer.load();
}

// External player fallback function
async function tryExternalPlayer(streamUrl) {
  try {
    console.log('Attempting external video player for:', streamUrl);

    // Show appropriate loading state based on stream type
    const isMagnet = streamUrl.startsWith('magnet:');
    const loadingMsg = document.createElement('div');
    loadingMsg.id = 'stream-status';

    if (isMagnet) {
      loadingMsg.innerHTML = `
        <div style="text-align: center;">
          <div class="loading-spinner" style="margin: 0 auto 15px auto;"></div>
          <div style="font-size: 18px; margin-bottom: 10px;">Starting torrent stream...</div>
          <div style="font-size: 14px; color: var(--text-secondary);">Setting up Peerflix and downloading...</div>
          <div style="font-size: 12px; color: var(--text-secondary); margin-top: 10px;">This may take a few seconds</div>
        </div>
      `;
    } else {
      loadingMsg.innerHTML = `
        <div style="text-align: center;">
          <div class="loading-spinner" style="margin: 0 auto 15px auto;"></div>
          <div style="font-size: 18px;">Launching external video player...</div>
        </div>
      `;
    }

    loadingMsg.style.cssText = `
      position: fixed;
      top: 50%;
      left: 50%;
      transform: translate(-50%, -50%);
      background: var(--bg-secondary);
      padding: 30px;
      border-radius: 12px;
      z-index: 2000;
      color: var(--text-primary);
      box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
      border: 1px solid var(--border-color);
      min-width: 320px;
    `;
    document.body.appendChild(loadingMsg);

    const result = await safeInvoke('play_video_external', { streamUrl: streamUrl });
    console.log('Play video result:', result);

    // Remove loading message
    hideStatus();

    // For magnet links, Peerflix returns the local HTTP stream URL
    // Play it in the built-in player
    if (isMagnet && result && typeof result === 'string' && result.startsWith('http://127.0.0.1:')) {
      console.log('🎬 Opening Peerflix stream in built-in player:', result);

      // Create a fake stream object for the built-in player
      const peerflixStream = {
        title: 'Torrent Stream (Peerflix)',
        url: result
      };

      playWithBuiltInPlayer(peerflixStream);
    } else if (isMagnet) {
      // Fallback: If we didn't get a URL, show success message
      showStatus('Torrent stream started!', 3000);
    }

  } catch (error) {
    // Remove loading message if it exists
    hideStatus();

    console.error('Failed to launch external player:', error);

    // Show user-friendly error message
    const errorMsg = error.message || error;
    if (errorMsg === 'undefined' || errorMsg.includes('undefined')) {
      showError('External video player not available. Please install MPV or VLC media player.');
    } else if (errorMsg.includes('No video player found')) {
      showError('No video player found. Please install:\n• Windows: VLC or MPV\n• Steam Deck: Install via Discover app');
    } else {
      showError('Failed to launch external video player: ' + errorMsg);
    }
  }
}

// Helper functions for status messages
function showStatus(message, duration = 0) {
  hideStatus(); // Remove any existing status

  const statusDiv = document.createElement('div');
  statusDiv.id = 'stream-status';
  statusDiv.innerHTML = `
    <div style="text-align: center;">
      <div style="font-size: 18px;">${message}</div>
    </div>
  `;
  statusDiv.style.cssText = `
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    background: var(--bg-secondary);
    padding: 20px 40px;
    border-radius: 8px;
    z-index: 2000;
    color: var(--text-primary);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
    border: 1px solid var(--border-color);
  `;
  document.body.appendChild(statusDiv);

  // Auto-hide after duration if specified
  if (duration > 0) {
    setTimeout(hideStatus, duration);
  }
}

function showError(message) {
  hideStatus(); // Remove any existing status

  const errorDiv = document.createElement('div');
  errorDiv.id = 'stream-status';
  errorDiv.innerHTML = `
    <div style="text-align: center;">
      <div style="font-size: 18px; color: #ff6b6b; margin-bottom: 10px;">⚠️ Error</div>
      <div style="font-size: 14px; white-space: pre-line;">${message}</div>
      <button onclick="hideStatus()" style="
        margin-top: 15px;
        padding: 8px 16px;
        background: var(--accent-color);
        color: white;
        border: none;
        border-radius: 4px;
        cursor: pointer;
        font-size: 14px;
      ">OK</button>
    </div>
  `;
  errorDiv.style.cssText = `
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    background: var(--bg-secondary);
    padding: 25px;
    border-radius: 8px;
    z-index: 2000;
    color: var(--text-primary);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
    border: 1px solid #ff6b6b;
    max-width: 400px;
  `;
  document.body.appendChild(errorDiv);
}

function hideStatus() {
  const status = document.getElementById('stream-status');
  if (status) status.remove();
}

// Video player control functions
function closeVideoPlayer() {
  console.log('Closing video player');

  // Pause and reset video
  if (elements.videoPlayer) {
    elements.videoPlayer.pause();
    elements.videoPlayer.src = '';
    elements.videoPlayer.load();
  }

  // Hide video modal
  elements.videoPlayerModal.classList.add('hidden');

  // Clear stored stream URL
  window.currentStreamUrl = null;

  // Return focus to main content
  if (appState.focusedElement) {
    appState.focusedElement.focus();
  }
}

function showVideoError(message) {
  console.error('Video error:', message);

  elements.videoLoading.classList.add('hidden');
  elements.videoPlayer.classList.add('hidden');
  elements.videoError.classList.remove('hidden');
  elements.videoErrorMessage.textContent = message;
}

// Video keyboard controls
function handleVideoKeyboard(e) {
  const video = elements.videoPlayer;

  switch (e.key) {
    case ' ':
    case 'k':
      // Play/Pause
      if (video.paused) {
        video.play();
      } else {
        video.pause();
      }
      e.preventDefault();
      break;

    case 'f':
    case 'F11':
      // Fullscreen
      if (video.requestFullscreen) {
        video.requestFullscreen();
      }
      e.preventDefault();
      break;

    case 'Escape':
      // Close player
      closeVideoPlayer();
      e.preventDefault();
      break;

    case 'ArrowLeft':
      // Seek backward 10s
      video.currentTime = Math.max(0, video.currentTime - 10);
      e.preventDefault();
      break;

    case 'ArrowRight':
      // Seek forward 10s
      video.currentTime = Math.min(video.duration, video.currentTime + 10);
      e.preventDefault();
      break;

    case 'ArrowUp':
      // Volume up
      video.volume = Math.min(1, video.volume + 0.1);
      e.preventDefault();
      break;

    case 'ArrowDown':
      // Volume down
      video.volume = Math.max(0, video.volume - 0.1);
      e.preventDefault();
      break;

    case 'm':
      // Mute/unmute
      video.muted = !video.muted;
      e.preventDefault();
      break;
  }
}

function closeStreamModal() {
  elements.streamModal.classList.add('hidden');
  appState.currentContent = null;
  appState.currentStreams = [];

  // Return focus to content
  if (appState.focusedElement) {
    appState.focusedElement.focus();
  }
}

function showStreamsError() {
  elements.streamsLoading.classList.add('hidden');
  elements.streamsList.classList.add('hidden');
  elements.streamsError.classList.remove('hidden');
}

// Enhanced keyboard navigation for Steam Deck controller
function handleKeyboard(e) {
  // If a modal is open, handle modal navigation
  if (!elements.streamModal.classList.contains('hidden')) {
    handleModalKeyboard(e);
    return;
  }

  // Handle search input focus with '/' key
  if (e.key === '/' && document.activeElement !== elements.searchInput) {
    elements.searchInput.focus();
    e.preventDefault();
    return;
  }

  // If search input is focused, don't handle grid navigation
  if (document.activeElement === elements.searchInput) {
    if (e.key === 'Escape') {
      elements.searchInput.blur();
      setFocusToFirstContent();
      e.preventDefault();
    }
    return;
  }

  // Get all visible sections with their cards
  const sections = [];

  // Continue Watching
  if (!elements.continueWatchingSection.classList.contains('hidden')) {
    const cards = Array.from(elements.continueWatchingGrid.querySelectorAll('.continue-watching-card:not(.hidden)'));
    if (cards.length > 0) {
      sections.push({ name: 'continue-watching', cards: cards, grid: elements.continueWatchingGrid });
    }
  }

  // Movies
  if (!elements.moviesSection.classList.contains('hidden')) {
    const cards = Array.from(elements.moviesGrid.querySelectorAll('.content-card:not(.hidden), .movie-card:not(.hidden)'));
    if (cards.length > 0) {
      sections.push({ name: 'movies', cards: cards, grid: elements.moviesGrid });
    }
  }

  // Series
  if (!elements.seriesSection.classList.contains('hidden')) {
    const cards = Array.from(elements.seriesGrid.querySelectorAll('.content-card:not(.hidden), .movie-card:not(.hidden)'));
    if (cards.length > 0) {
      sections.push({ name: 'series', cards: cards, grid: elements.seriesGrid });
    }
  }

  // Anime
  if (!elements.animeSection.classList.contains('hidden')) {
    const cards = Array.from(elements.animeGrid.querySelectorAll('.content-card:not(.hidden), .movie-card:not(.hidden)'));
    if (cards.length > 0) {
      sections.push({ name: 'anime', cards: cards, grid: elements.animeGrid });
    }
  }

  // Search results
  if (!elements.searchSection.classList.contains('hidden')) {
    const cards = Array.from(elements.searchGrid.querySelectorAll('.content-card:not(.hidden), .movie-card:not(.hidden)'));
    if (cards.length > 0) {
      sections.push({ name: 'search', cards: cards, grid: elements.searchGrid });
    }
  }

  if (sections.length === 0) return;

  // Find current section and card index within that section
  const focusedCard = appState.focusedElement;
  let currentSectionIndex = 0;
  let currentCardIndex = 0;

  if (focusedCard) {
    for (let i = 0; i < sections.length; i++) {
      const cardIndex = sections[i].cards.indexOf(focusedCard);
      if (cardIndex !== -1) {
        currentSectionIndex = i;
        currentCardIndex = cardIndex;
        break;
      }
    }
  }

  switch (e.key) {
    case 'ArrowLeft':
      // Move to previous card in current section
      if (currentCardIndex > 0) {
        const newCard = sections[currentSectionIndex].cards[currentCardIndex - 1];
        setFocusedElement(newCard);
        newCard.focus();
        newCard.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'center' });
        e.preventDefault();
      }
      break;

    case 'ArrowRight':
      // Move to next card in current section
      if (currentCardIndex < sections[currentSectionIndex].cards.length - 1) {
        const newCard = sections[currentSectionIndex].cards[currentCardIndex + 1];
        setFocusedElement(newCard);
        newCard.focus();
        newCard.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'center' });
        e.preventDefault();
      }
      break;

    case 'ArrowUp':
      // Move to previous section, try to keep similar horizontal position
      if (currentSectionIndex > 0) {
        const newSectionIndex = currentSectionIndex - 1;
        const newSection = sections[newSectionIndex];
        // Try to maintain horizontal position, or go to last card if section is smaller
        const newCardIndex = Math.min(currentCardIndex, newSection.cards.length - 1);
        const newCard = newSection.cards[newCardIndex];
        setFocusedElement(newCard);
        newCard.focus();
        newCard.scrollIntoView({ behavior: 'smooth', block: 'center', inline: 'center' });
        e.preventDefault();
      }
      break;

    case 'ArrowDown':
      // Move to next section, try to keep similar horizontal position
      if (currentSectionIndex < sections.length - 1) {
        const newSectionIndex = currentSectionIndex + 1;
        const newSection = sections[newSectionIndex];
        // Try to maintain horizontal position, or go to last card if section is smaller
        const newCardIndex = Math.min(currentCardIndex, newSection.cards.length - 1);
        const newCard = newSection.cards[newCardIndex];
        setFocusedElement(newCard);
        newCard.focus();
        newCard.scrollIntoView({ behavior: 'smooth', block: 'center', inline: 'center' });
        e.preventDefault();
      }
      break;

    case 'Enter':
    case ' ':
      if (focusedCard) {
        focusedCard.click();
      }
      e.preventDefault();
      break;

    case 'Escape':
      // Clear search if active, otherwise blur current element
      if (appState.searchQuery) {
        clearSearch();
      } else if (appState.focusedElement) {
        appState.focusedElement.blur();
        setFocusedElement(null);
      }
      e.preventDefault();
      break;

    default:
      return;
  }
}

function handleModalKeyboard(e) {
  const streamItems = Array.from(elements.streamsList.querySelectorAll('.stream-item'));

  switch (e.key) {
    case 'Escape':
      closeStreamModal();
      break;
    case 'ArrowUp':
      if (streamItems.length > 0) {
        const focused = document.activeElement;
        const index = streamItems.indexOf(focused);
        const newIndex = Math.max(0, index - 1);
        streamItems[newIndex].focus();
      }
      e.preventDefault();
      break;
    case 'ArrowDown':
      if (streamItems.length > 0) {
        const focused = document.activeElement;
        const index = streamItems.indexOf(focused);
        const newIndex = Math.min(streamItems.length - 1, index + 1);
        streamItems[newIndex].focus();
      }
      e.preventDefault();
      break;
    case 'Enter':
    case ' ':
      const focused = document.activeElement;
      if (focused && focused.classList.contains('stream-item')) {
        const index = streamItems.indexOf(focused);
        const stream = appState.currentStreams[index];
        playStream(stream);
      }
      e.preventDefault();
      break;
  }
}

// Test Cinemeta v3 API directly
async function testCinemetaAPI() {
  DEBUG.log('API_TEST', 'Testing Cinemeta v3 API directly...');

  try {
    const url = 'https://v3-cinemeta.strem.io/catalog/movie/popular.json';
    DEBUG.log('API_TEST', `Fetching from v3 endpoint: ${url}`);

    const response = await fetch(url);
    DEBUG.log('API_TEST', `Response status: ${response.status} ${response.statusText}`);

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const data = await response.json();
    DEBUG.log('API_TEST', 'Response received', {
      hasMetasField: !!data.metas,
      metasCount: data.metas ? data.metas.length : 0,
      responseKeys: Object.keys(data),
      sampleMetas: data.metas ? data.metas.slice(0, 3) : []
    });

    if (window.debugPanel) {
      window.debugPanel.updateAPI('Direct API Test', `Success: ${data.metas ? data.metas.length : 0} movies found`);
    }

    return data;
  } catch (error) {
    DEBUG.error('API_TEST', 'Direct Cinemeta API test failed', error);

    if (window.debugPanel) {
      window.debugPanel.updateAPI('Direct API Test', `Failed: ${error.message}`);
    }

    throw error;
  }
}

// Initialize when DOM is loaded
window.addEventListener('DOMContentLoaded', async () => {
  await initApp();

  // Test Cinemeta API directly after initialization
  setTimeout(async () => {
    DEBUG.log('API_TEST', 'Running direct Cinemeta API test...');
    try {
      await testCinemetaAPI();
    } catch (error) {
      DEBUG.error('API_TEST', 'Direct API test failed during initialization', error);
    }
  }, 2000);
});

// Export for controller.js
// Make status functions globally available
window.showStatus = showStatus;
window.showError = showError;
window.hideStatus = hideStatus;

window.DeckFlixApp = {
  appState,
  elements,
  setFocusedElement,
  selectContent,
  playStream,
  closeStreamModal,
  handleKeyboard,
  handleModalKeyboard,
  performSearch,
  clearSearch,
  addToContinueWatching,
  showStatus,
  showError,
  hideStatus
};