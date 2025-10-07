use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Movie {
    pub id: String,
    pub name: String,
    pub poster: Option<String>,
    pub background: Option<String>,
    pub description: Option<String>,
    pub year: Option<String>,
    pub imdb_rating: Option<String>,
    pub genre: Option<Vec<String>>,
    #[serde(rename = "type")]
    pub content_type: Option<String>,
    pub director: Option<Vec<String>>,
    pub cast: Option<Vec<String>>,
    pub runtime: Option<String>,
    pub country: Option<String>,
    pub language: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Series {
    pub id: String,
    pub name: String,
    pub poster: Option<String>,
    pub background: Option<String>,
    pub description: Option<String>,
    pub year: Option<String>,
    pub imdb_rating: Option<String>,
    pub genre: Option<Vec<String>>,
    #[serde(rename = "type")]
    pub content_type: Option<String>,
    pub director: Option<Vec<String>>,
    pub cast: Option<Vec<String>>,
    pub runtime: Option<String>,
    pub country: Option<String>,
    pub language: Option<String>,
    // Series-specific fields
    pub seasons: Option<u32>,
    pub episodes: Option<u32>,
    pub status: Option<String>, // "Ended", "Continuing", etc.
    pub network: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Anime {
    pub id: String,
    pub name: String,
    pub poster: Option<String>,
    pub background: Option<String>,
    pub description: Option<String>,
    pub year: Option<String>,
    pub imdb_rating: Option<String>,
    pub genre: Option<Vec<String>>,
    #[serde(rename = "type")]
    pub content_type: Option<String>,
    pub director: Option<Vec<String>>,
    pub cast: Option<Vec<String>>,
    pub runtime: Option<String>,
    pub country: Option<String>,
    pub language: Option<String>,
    // Anime-specific fields
    pub seasons: Option<u32>,
    pub episodes: Option<u32>,
    pub status: Option<String>,
    pub studio: Option<String>,
    pub mal_rating: Option<String>, // MyAnimeList rating
    pub anime_type: Option<String>, // TV, Movie, OVA, etc.
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Stream {
    pub name: Option<String>,
    pub title: String,
    pub url: String,
    pub behavior_hints: Option<StreamBehaviorHints>,
    pub quality: Option<String>,
    pub size: Option<String>,
    pub seeders: Option<u32>,
    pub leechers: Option<u32>,
    pub source: Option<String>, // torrent, direct, etc.
    pub language: Option<String>,
    pub subtitles: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StreamBehaviorHints {
    pub not_web_ready: Option<bool>,
    pub proxy_headers: Option<serde_json::Value>,
}

// Content trait for shared functionality
pub trait Content {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn poster(&self) -> Option<&str>;
    fn description(&self) -> Option<&str>;
    fn year(&self) -> Option<&str>;
    fn imdb_rating(&self) -> Option<&str>;
    fn content_type(&self) -> &str;
}

impl Content for Movie {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { &self.name }
    fn poster(&self) -> Option<&str> { self.poster.as_deref() }
    fn description(&self) -> Option<&str> { self.description.as_deref() }
    fn year(&self) -> Option<&str> { self.year.as_deref() }
    fn imdb_rating(&self) -> Option<&str> { self.imdb_rating.as_deref() }
    fn content_type(&self) -> &str { "movie" }
}

impl Content for Series {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { &self.name }
    fn poster(&self) -> Option<&str> { self.poster.as_deref() }
    fn description(&self) -> Option<&str> { self.description.as_deref() }
    fn year(&self) -> Option<&str> { self.year.as_deref() }
    fn imdb_rating(&self) -> Option<&str> { self.imdb_rating.as_deref() }
    fn content_type(&self) -> &str { "series" }
}

impl Content for Anime {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { &self.name }
    fn poster(&self) -> Option<&str> { self.poster.as_deref() }
    fn description(&self) -> Option<&str> { self.description.as_deref() }
    fn year(&self) -> Option<&str> { self.year.as_deref() }
    fn imdb_rating(&self) -> Option<&str> { self.imdb_rating.as_deref() }
    fn content_type(&self) -> &str { "anime" }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub id: String,
    pub name: String,
    pub poster: Option<String>,
    pub year: Option<String>,
    pub imdb_rating: Option<String>,
    #[serde(rename = "type")]
    pub content_type: String, // "movie", "series", "anime"
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StremioResponse<T> {
    pub metas: Option<Vec<T>>,
    pub streams: Option<Vec<T>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub metas: Option<Vec<SearchResult>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Addon {
    pub name: String,
    pub base_url: String,
    pub manifest: AddonManifest,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddonManifest {
    pub id: String,
    pub version: String,
    pub name: String,
    pub description: String,
    pub resources: Vec<String>,
    pub types: Vec<String>,
    pub catalogs: Vec<AddonCatalog>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddonCatalog {
    #[serde(rename = "type")]
    pub catalog_type: String,
    pub id: String,
    pub name: String,
}

// Continue watching item for localStorage persistence
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContinueWatchingItem {
    pub id: String,
    pub name: String,
    pub poster: Option<String>,
    pub content_type: String, // "movie", "series", "anime"
    pub progress: f64, // 0.0 to 1.0
    pub last_watched: String, // ISO timestamp
    pub season: Option<u32>,
    pub episode: Option<u32>,
}