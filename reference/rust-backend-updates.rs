// src-tauri/src/main.rs - Updated with multi-content support

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;
use tokio::sync::Mutex as AsyncMutex;

mod models;
mod addon_client;

use models::*;
use addon_client::AddonClient;

// Global state for the HTTP client
struct AppState {
    client: AsyncMutex<AddonClient>,
}

#[tauri::command]
async fn fetch_popular_movies(state: State<'_, AppState>) -> Result<Vec<Movie>, String> {
    let client = state.client.lock().await;
    client.fetch_movies("popular").await
}

#[tauri::command]
async fn fetch_popular_series(state: State<'_, AppState>) -> Result<Vec<Series>, String> {
    let client = state.client.lock().await;
    client.fetch_series("popular").await
}

#[tauri::command]
async fn fetch_popular_anime(state: State<'_, AppState>) -> Result<Vec<Anime>, String> {
    let client = state.client.lock().await;
    client.fetch_anime("popular").await
}

#[tauri::command]
async fn fetch_streams(imdb_id: String, state: State<'_, AppState>) -> Result<Vec<Stream>, String> {
    let client = state.client.lock().await;
    client.fetch_streams(&imdb_id).await
}

#[tauri::command]
async fn search_content(query: String, state: State<'_, AppState>) -> Result<SearchResults, String> {
    let client = state.client.lock().await;
    client.search_content(&query).await
}

#[tauri::command]
async fn play_video_external(app: tauri::AppHandle, stream_url: String) -> Result<String, String> {
    let shell = app.shell();
    
    // Try different video players in order of preference
    let players = vec!["mpv", "vlc", "xdg-open"];
    
    for player in players {
        match shell.command(player).arg(&stream_url).spawn() {
            Ok(_) => {
                return Ok(format!("Launched {} with stream", player));
            }
            Err(_) => {
                continue; // Try next player
            }
        }
    }
    
    Err("No video player found. Please install mpv or vlc.".to_string())
}

#[tauri::command]
async fn get_addon_status() -> Result<String, String> {
    Ok("Addons operational".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState {
        client: AsyncMutex::new(AddonClient::new()),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            fetch_popular_movies,
            fetch_popular_series,
            fetch_popular_anime,
            fetch_streams,
            search_content,
            play_video_external,
            get_addon_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// src-tauri/src/models.rs - Updated data models

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Movie {
    pub id: String,
    pub name: String,
    pub poster: Option<String>,
    pub background: Option<String>,
    pub description: Option<String>,
    pub year: Option<String>,
    pub imdb_rating: Option<String>,
    pub genre: Option<Vec<String>>,
    pub director: Option<String>,
    pub cast: Option<Vec<String>>,
    pub runtime: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Series {
    pub id: String,
    pub name: String,
    pub poster: Option<String>,
    pub background: Option<String>,
    pub description: Option<String>,
    pub year: Option<String>,
    pub imdb_rating: Option<String>,
    pub genre: Option<Vec<String>>,
    pub director: Option<String>,
    pub cast: Option<Vec<String>>,
    pub seasons: Option<i32>,
    pub episodes: Option<i32>,
    pub status: Option<String>, // "Ongoing", "Completed", "Cancelled"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anime {
    pub id: String,
    pub name: String,
    pub poster: Option<String>,
    pub background: Option<String>,
    pub description: Option<String>,
    pub year: Option<String>,
    pub imdb_rating: Option<String>,
    pub genre: Option<Vec<String>>,
    pub studio: Option<String>,
    pub episodes: Option<i32>,
    pub status: Option<String>,
    pub mal_rating: Option<String>, // MyAnimeList rating
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stream {
    pub name: Option<String>,
    pub title: String,
    pub url: String,
    pub quality: Option<String>,
    pub size: Option<String>,
    pub seeds: Option<i32>,
    pub behavior_hints: Option<StreamBehaviorHints>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamBehaviorHints {
    pub not_web_ready: Option<bool>,
    pub bingeGroup: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResults {
    pub movies: Vec<Movie>,
    pub series: Vec<Series>,
    pub anime: Vec<Anime>,
}

// src-tauri/src/addon_client.rs - Updated HTTP client

use reqwest::{Client, Error as ReqwestError};
use serde_json::Value;
use std::time::Duration;
use crate::models::*;

pub struct AddonClient {
    client: Client,
    base_urls: Vec<String>,
}

impl AddonClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("DeckFlix/1.0")
            .build()
            .expect("Failed to create HTTP client");

        // Multiple addon sources for reliability
        let base_urls = vec![
            "https://cinemeta-live.herokuapp.com".to_string(),
            "https://torrentio.strem.fun".to_string(),
            "https://stremio-addon-metadata.herokuapp.com".to_string(),
        ];

        Self { client, base_urls }
    }

    pub async fn fetch_movies(&self, catalog: &str) -> Result<Vec<Movie>, String> {
        let mut all_movies = Vec::new();

        for base_url in &self.base_urls {
            match self.fetch_movies_from_addon(base_url, catalog).await {
                Ok(mut movies) => {
                    all_movies.append(&mut movies);
                    if all_movies.len() >= 50 {
                        break; // Enough content loaded
                    }
                }
                Err(e) => {
                    eprintln!("Failed to fetch movies from {}: {}", base_url, e);
                    continue;
                }
            }
        }

        // Remove duplicates and limit results
        all_movies.sort_by(|a, b| a.id.cmp(&b.id));
        all_movies.dedup_by(|a, b| a.id == b.id);
        all_movies.truncate(30);

        if all_movies.is_empty() {
            Err("No movies could be loaded from any addon".to_string())
        } else {
            Ok(all_movies)
        }
    }

    pub async fn fetch_series(&self, catalog: &str) -> Result<Vec<Series>, String> {
        let mut all_series = Vec::new();

        for base_url in &self.base_urls {
            match self.fetch_series_from_addon(base_url, catalog).await {
                Ok(mut series) => {
                    all_series.append(&mut series);
                    if all_series.len() >= 50 {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to fetch series from {}: {}", base_url, e);
                    continue;
                }
            }
        }

        all_series.sort_by(|a, b| a.id.cmp(&b.id));
        all_series.dedup_by(|a, b| a.id == b.id);
        all_series.truncate(30);

        if all_series.is_empty() {
            Err("No series could be loaded from any addon".to_string())
        } else {
            Ok(all_series)
        }
    }

    pub async fn fetch_anime(&self, catalog: &str) -> Result<Vec<Anime>, String> {
        // For anime, we'll try specific anime addons or filter existing content
        let mut all_anime = Vec::new();

        // Try anime-specific endpoints or filter from movies/series
        for base_url in &self.base_urls {
            match self.fetch_anime_from_addon(base_url, catalog).await {
                Ok(mut anime) => {
                    all_anime.append(&mut anime);
                    if all_anime.len() >= 30 {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to fetch anime from {}: {}", base_url, e);
                    continue;
                }
            }
        }

        all_anime.sort_by(|a, b| a.id.cmp(&b.id));
        all_anime.dedup_by(|a, b| a.id == b.id);
        all_anime.truncate(20);

        if all_anime.is_empty() {
            Err("No anime could be loaded from any addon".to_string())
        } else {
            Ok(all_anime)
        }
    }

    async fn fetch_movies_from_addon(&self, base_url: &str, catalog: &str) -> Result<Vec<Movie>, String> {
        let url = format!("{}/catalog/movie/{}.json", base_url, catalog);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        let json: Value = response
            .json()
            .await
            .map_err(|e| format!("JSON parse error: {}", e))?;

        self.parse_movies_response(json)
    }

    async fn fetch_series_from_addon(&self, base_url: &str, catalog: &str) -> Result<Vec<Series>, String> {
        let url = format!("{}/catalog/series/{}.json", base_url, catalog);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        let json: Value = response
            .json()
            .await
            .map_err(|e| format!("JSON parse error: {}", e))?;

        self.parse_series_response(json)
    }

    async fn fetch_anime_from_addon(&self, base_url: &str, catalog: &str) -> Result<Vec<Anime>, String> {
        // Try anime-specific catalog first, fallback to series with anime genres
        let anime_urls = vec![
            format!("{}/catalog/anime/{}.json", base_url, catalog),
            format!("{}/catalog/series/{}.json", base_url, catalog),
        ];

        for url in anime_urls {
            match self.client.get(&url).send().await {
                Ok(response) if response.status().is_success() => {
                    match response.json::<Value>().await {
                        Ok(json) => {
                            if let Ok(anime) = self.parse_anime_response(json) {
                                if !anime.is_empty() {
                                    return Ok(anime);
                                }
                            }
                        }
                        Err(_) => continue,
                    }
                }
                _ => continue,
            }
        }

        Err("No anime content found".to_string())
    }

    fn parse_movies_response(&self, json: Value) -> Result<Vec<Movie>, String> {
        let metas = json["metas"]
            .as_array()
            .ok_or("Invalid response format: missing metas array")?;

        let mut movies = Vec::new();

        for meta in metas {
            if let Some(movie) = self.parse_movie_meta(meta) {
                movies.push(movie);
            }
        }

        Ok(movies)
    }

    fn parse_series_response(&self, json: Value) -> Result<Vec<Series>, String> {
        let metas = json["metas"]
            .as_array()
            .ok_or("Invalid response format: missing metas array")?;

        let mut series = Vec::new();

        for meta in metas {
            if let Some(show) = self.parse_series_meta(meta) {
                series.push(show);
            }
        }

        Ok(series)
    }

    fn parse_anime_response(&self, json: Value) -> Result<Vec<Anime>, String> {
        let metas = json["metas"]
            .as_array()
            .ok_or("Invalid response format: missing metas array")?;

        let mut anime = Vec::new();

        for meta in metas {
            // Filter for anime content or use genre filtering
            if let Some(show) = self.parse_anime_meta(meta) {
                anime.push(show);
            }
        }

        Ok(anime)
    }

    fn parse_movie_meta(&self, meta: &Value) -> Option<Movie> {
        let id = meta["id"].as_str()?.to_string();
        let name = meta["name"].as_str()?.to_string();

        Some(Movie {
            id,
            name,
            poster: meta["poster"].as_str().map(|s| s.to_string()),
            background: meta["background"].as_str().map(|s| s.to_string()),
            description: meta["description"].as_str().map(|s| s.to_string()),
            year: meta["year"].as_str().map(|s| s.to_string()),
            imdb_rating: meta["imdbRating"].as_str().map(|s| s.to_string()),
            genre: meta["genre"].as_array().map(|arr| {
                arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()
            }),
            director: meta["director"].as_array().and_then(|arr| {
                arr.first().and_then(|v| v.as_str().map(|s| s.to_string()))
            }),
            cast: meta["cast"].as_array().map(|arr| {
                arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()
            }),
            runtime: meta["runtime"].as_str().map(|s| s.to_string()),
        })
    }

    fn parse_series_meta(&self, meta: &Value) -> Option<Series> {
        let id = meta["id"].as_str()?.to_string();
        let name = meta["name"].as_str()?.to_string();

        Some(Series {
            id,
            name,
            poster: meta["poster"].as_str().map(|s| s.to_string()),
            background: meta["background"].as_str().map(|s| s.to_string()),
            description: meta["description"].as_str().map(|s| s.to_string()),
            year: meta["year"].as_str().map(|s| s.to_string()),
            imdb_rating: meta["imdbRating"].as_str().map(|s| s.to_string()),
            genre: meta["genre"].as_array().map(|arr| {
                arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()
            }),
            director: meta["director"].as_array().and_then(|arr| {
                arr.first().and_then(|v| v.as_str().map(|s| s.to_string()))
            }),
            cast: meta["cast"].as_array().map(|arr| {
                arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()
            }),
            seasons: meta["seasons"].as_i64().map(|n| n as i32),
            episodes: meta["episodes"].as_i64().map(|n| n as i32),
            status: meta["status"].as_str().map(|s| s.to_string()),
        })
    }

    fn parse_anime_meta(&self, meta: &Value) -> Option<Anime> {
        let id = meta["id"].as_str()?.to_string();
        let name = meta["name"].as_str()?.to_string();

        // Filter for anime-like content
        if let Some(genres) = meta["genre"].as_array() {
            let genre_strings: Vec<String> = genres
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            
            let is_anime = genre_strings.iter().any(|g| {
                g.to_lowercase().contains("anime") || 
                g.to_lowercase().contains("animation")
            }) || name.contains("Anime") || 
               id.contains("anime");

            if !is_anime {
                return None;
            }
        }

        Some(Anime {
            id,
            name,
            poster: meta["poster"].as_str().map(|s| s.to_string()),
            background: meta["background"].as_str().map(|s| s.to_string()),
            description: meta["description"].as_str().map(|s| s.to_string()),
            year: meta["year"].as_str().map(|s| s.to_string()),
            imdb_rating: meta["imdbRating"].as_str().map(|s| s.to_string()),
            genre: meta["genre"].as_array().map(|arr| {
                arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()
            }),
            studio: meta["studio"].as_str().map(|s| s.to_string()),
            episodes: meta["episodes"].as_i64().map(|n| n as i32),
            status: meta["status"].as_str().map(|s| s.to_string()),
            mal_rating: meta["malRating"].as_str().map(|s| s.to_string()),
        })
    }

    pub async fn fetch_streams(&self, imdb_id: &str) -> Result<Vec<Stream>, String> {
        let mut all_streams = Vec::new();

        for base_url in &self.base_urls {
            match self.fetch_streams_from_addon(base_url, imdb_id).await {
                Ok(mut streams) => {
                    all_streams.append(&mut streams);
                }
                Err(e) => {
                    eprintln!("Failed to fetch streams from {}: {}", base_url, e);
                    continue;
                }
            }
        }

        // Sort streams by quality and seeds
        all_streams.sort_by(|a, b| {
            let a_quality = extract_quality_score(&a.title);
            let b_quality = extract_quality_score(&b.title);
            b_quality.cmp(&a_quality)
        });

        if all_streams.is_empty() {
            Err("No streams available".to_string())
        } else {
            Ok(all_streams)
        }
    }

    async fn fetch_streams_from_addon(&self, base_url: &str, imdb_id: &str) -> Result<Vec<Stream>, String> {
        let url = format!("{}/stream/movie/{}.json", base_url, imdb_id);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        let json: Value = response
            .json()
            .await
            .map_err(|e| format!("JSON parse error: {}", e))?;

        self.parse_streams_response(json)
    }

    fn parse_streams_response(&self, json: Value) -> Result<Vec<Stream>, String> {
        let streams = json["streams"]
            .as_array()
            .ok_or("Invalid response format: missing streams array")?;

        let mut parsed_streams = Vec::new();

        for stream in streams {
            if let Some(parsed_stream) = self.parse_stream(stream) {
                parsed_streams.push(parsed_stream);
            }
        }

        Ok(parsed_streams)
    }

    fn parse_stream(&self, stream: &Value) -> Option<Stream> {
        let title = stream["title"].as_str()?.to_string();
        let url = stream["url"].as_str()?.to_string();