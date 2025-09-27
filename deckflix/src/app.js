const { invoke } = window.__TAURI__.core;

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
  focusedElement: null,
  focusIndex: 0,
  currentSection: 'movies',
  searchQuery: '',
  searchTimeout: null
};

// DOM elements
let elements = {};

// Continue watching storage key
const CONTINUE_WATCHING_KEY = 'deckflix_continue_watching';

// Initialize the application
async function initApp() {
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
    closeModal: document.getElementById('close-modal')
  };

  // Set up event listeners
  setupEventListeners();

  // Initialize focus management
  initializeFocusManagement();

  // Check addon status
  await checkAddonStatus();

  // Load continue watching from localStorage
  loadContinueWatching();

  // Load all content sections
  await loadAllContent();
}

function setupEventListeners() {
  // Search functionality
  elements.searchInput.addEventListener('input', handleSearchInput);
  elements.searchClear.addEventListener('click', clearSearch);

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

  // Keyboard navigation fallback
  document.addEventListener('keydown', handleKeyboard);
}

function initializeFocusManagement() {
  // Initialize focus system for controller navigation
  appState.focusedElement = null;
  appState.focusIndex = 0;
}

async function checkAddonStatus() {
  try {
    const status = await invoke('get_addon_status');
    elements.addonStatus.textContent = status;
    elements.addonStatus.style.color = 'var(--success)';
  } catch (error) {
    console.error('Failed to get addon status:', error);
    elements.addonStatus.textContent = 'Addon connection failed';
    elements.addonStatus.style.color = 'var(--error)';
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
  elements.searchInput.value = '';
  elements.searchClear.classList.add('hidden');
  appState.searchQuery = '';
  hideSearchResults();
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
  try {
    console.log('Searching for:', query);

    // Show search loading
    elements.searchLoading.classList.remove('hidden');
    elements.searchGrid.innerHTML = '';
    elements.searchError.classList.add('hidden');
    showSearchResults();

    const results = await invoke('search_content', { query });
    console.log('Search results:', results.length);

    appState.searchResults = results;

    // Hide loading
    elements.searchLoading.classList.add('hidden');

    if (results.length === 0) {
      elements.searchError.classList.remove('hidden');
      elements.searchResultsCount.textContent = '';
    } else {
      displaySearchResults(results);
      elements.searchResultsCount.textContent = `${results.length} result${results.length !== 1 ? 's' : ''}`;
    }

  } catch (error) {
    console.error('Search failed:', error);
    elements.searchLoading.classList.add('hidden');
    elements.searchError.classList.remove('hidden');
    elements.searchResultsCount.textContent = '';
  }
}

// Content loading functions
async function loadAllContent() {
  try {
    // Show global loading
    elements.globalLoading.classList.remove('hidden');
    elements.globalError.classList.add('hidden');

    // Load all content types in parallel
    const [moviesPromise, seriesPromise, animePromise] = [
      loadMovies(),
      loadSeries(),
      loadAnime()
    ];

    await Promise.allSettled([moviesPromise, seriesPromise, animePromise]);

    // Hide global loading
    elements.globalLoading.classList.add('hidden');

  } catch (error) {
    console.error('Failed to load content:', error);
    elements.globalLoading.classList.add('hidden');
    elements.globalError.classList.remove('hidden');
  }
}

async function loadMovies() {
  try {
    elements.moviesLoading.classList.remove('hidden');
    elements.moviesGrid.classList.add('hidden');
    elements.moviesError.classList.add('hidden');

    console.log('Fetching popular movies...');
    const movies = await invoke('fetch_popular_movies');
    console.log('Movies received:', movies.length);

    appState.movies = movies;
    displayContent(movies, elements.moviesGrid, 'movie');

    elements.moviesLoading.classList.add('hidden');
    elements.moviesGrid.classList.remove('hidden');

  } catch (error) {
    console.error('Failed to load movies:', error);
    elements.moviesLoading.classList.add('hidden');
    elements.moviesError.classList.remove('hidden');
  }
}

async function loadSeries() {
  try {
    elements.seriesLoading.classList.remove('hidden');
    elements.seriesGrid.classList.add('hidden');
    elements.seriesError.classList.add('hidden');

    console.log('Fetching popular series...');
    const series = await invoke('fetch_popular_series');
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
    const anime = await invoke('fetch_popular_anime');
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
  // Clear existing content
  gridElement.innerHTML = '';

  // Create content cards
  items.forEach((item, index) => {
    const card = createContentCard(item, index, contentType);
    gridElement.appendChild(card);
  });
}

function displaySearchResults(results) {
  elements.searchGrid.innerHTML = '';

  results.forEach((item, index) => {
    const card = createContentCard(item, index, item.content_type);
    elements.searchGrid.appendChild(card);
  });
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
  const card = document.createElement('div');
  card.className = 'content-card focusable';
  card.dataset.contentIndex = index;
  card.dataset.contentType = contentType;
  card.tabIndex = 0;

  // Content poster
  const poster = document.createElement('div');
  poster.className = 'content-poster';

  if (content.poster) {
    const img = document.createElement('img');
    img.src = content.poster;
    img.alt = content.name;
    img.onerror = () => {
      poster.innerHTML = getContentIcon(contentType) + ' No Image';
    };
    poster.appendChild(img);
  } else {
    poster.innerHTML = getContentIcon(contentType) + ' No Image';
  }

  // Content info
  const info = document.createElement('div');
  info.className = 'content-info';

  // Content type indicator
  const typeIndicator = document.createElement('div');
  typeIndicator.className = 'content-type';
  typeIndicator.textContent = contentType.toUpperCase();

  const title = document.createElement('div');
  title.className = 'content-title';
  title.textContent = content.name;
  title.title = content.name;

  const details = document.createElement('div');
  details.className = 'content-details';

  const year = document.createElement('span');
  year.className = 'content-year';
  year.textContent = content.year || 'Unknown';

  const rating = document.createElement('span');
  rating.className = 'content-rating';
  rating.textContent = content.imdb_rating || (content.mal_rating || 'N/A');

  details.appendChild(year);
  details.appendChild(rating);

  info.appendChild(typeIndicator);
  info.appendChild(title);
  info.appendChild(details);

  card.appendChild(poster);
  card.appendChild(info);

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
      const firstCard = grid.querySelector('.content-card, .continue-watching-card');
      if (firstCard) {
        setFocusedElement(firstCard);
        firstCard.focus();
        return;
      }
    }
  }
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
  appState.currentContent = content;
  console.log('Selected content:', content.name, 'Type:', contentType);

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
    const streams = await invoke('fetch_streams', { imdbId: content.id });

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
  item.addEventListener('click', () => playStream(stream));

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
    console.log('Playing stream:', stream.url);

    // Show loading state
    const loadingMsg = document.createElement('div');
    loadingMsg.textContent = 'Launching video player...';
    loadingMsg.style.cssText = 'position: fixed; top: 50%; left: 50%; transform: translate(-50%, -50%); background: var(--bg-secondary); padding: 20px; border-radius: 8px; z-index: 2000;';
    document.body.appendChild(loadingMsg);

    const result = await invoke('play_video_external', { streamUrl: stream.url });

    console.log('Player launched:', result);

    // Remove loading message
    document.body.removeChild(loadingMsg);

    // Close modal
    closeStreamModal();

  } catch (error) {
    console.error('Failed to play stream:', error);
    alert('Failed to launch video player: ' + error);
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

// Keyboard navigation fallback
function handleKeyboard(e) {
  // If a modal is open, handle modal navigation
  if (!elements.streamModal.classList.contains('hidden')) {
    handleModalKeyboard(e);
    return;
  }

  // Handle search input focus
  if (e.key === '/' && !elements.streamModal.classList.contains('hidden') === false) {
    elements.searchInput.focus();
    e.preventDefault();
    return;
  }

  // If search input is focused, don't handle grid navigation
  if (document.activeElement === elements.searchInput) {
    return;
  }

  // Content grid navigation
  const allCards = Array.from(document.querySelectorAll('.content-card:not(.hidden), .continue-watching-card:not(.hidden)'));
  if (allCards.length === 0) return;

  const currentIndex = appState.focusIndex;
  let newIndex = currentIndex;

  switch (e.key) {
    case 'ArrowLeft':
      newIndex = Math.max(0, currentIndex - 1);
      break;
    case 'ArrowRight':
      newIndex = Math.min(allCards.length - 1, currentIndex + 1);
      break;
    case 'ArrowUp':
      const cols = Math.floor(window.innerWidth / 200); // Approximate
      newIndex = Math.max(0, currentIndex - cols);
      break;
    case 'ArrowDown':
      const colsDown = Math.floor(window.innerWidth / 200);
      newIndex = Math.min(allCards.length - 1, currentIndex + colsDown);
      break;
    case 'Enter':
    case ' ':
      if (allCards[currentIndex]) {
        allCards[currentIndex].click();
      }
      e.preventDefault();
      break;
    default:
      return;
  }

  if (newIndex !== currentIndex && allCards[newIndex]) {
    setFocusedElement(allCards[newIndex]);
    allCards[newIndex].focus();
    allCards[newIndex].scrollIntoView({ behavior: 'smooth', block: 'nearest' });
  }

  e.preventDefault();
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

// Initialize when DOM is loaded
window.addEventListener('DOMContentLoaded', initApp);

// Export for controller.js
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
  addToContinueWatching
};