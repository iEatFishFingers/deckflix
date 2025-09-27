use crate::models::{Movie, Series, Anime, Stream, StremioResponse, SearchResult, SearchResponse};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

pub struct AddonClient {
    client: Client,
    base_urls: Vec<String>,
}

impl AddonClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        // Use Cinemeta v3 API as the primary source
        let base_urls = vec![
            "https://v3-cinemeta.strem.io".to_string(),
            "https://torrentio.strem.fun".to_string(),
        ];

        Self { client, base_urls }
    }

    pub async fn fetch_popular_movies(&self) -> Result<Vec<Movie>, String> {
        println!("[RUST] [MOVIES_FETCH] Starting to fetch popular movies using correct Stremio v3 structure...");
        let mut all_movies = Vec::new();
        let mut last_error = String::new();

        println!("[RUST] [MOVIES_FETCH] Available addon URLs: {:?}", self.base_urls);
        println!("[RUST] [MOVIES_FETCH] Correct endpoint: https://v3-cinemeta.strem.io/catalog/movie/top.json");

        // Try Cinemeta first (prioritized)
        for (index, base_url) in self.base_urls.iter().enumerate() {
            println!("[RUST] [MOVIES_FETCH] Attempting to fetch from addon {} ({})", index + 1, base_url);

            let start_time = std::time::Instant::now();
            match self.fetch_movies_from_addon(base_url, "top").await {
                Ok(mut movies) => {
                    let duration = start_time.elapsed();
                    println!("[RUST] [MOVIES_FETCH] Successfully fetched {} movies from {} in {:?}",
                            movies.len(), base_url, duration);

                    // Log sample movie data for debugging
                    if !movies.is_empty() {
                        println!("[RUST] [MOVIES_FETCH] Sample movies:");
                        for (i, movie) in movies.iter().take(3).enumerate() {
                            println!("[RUST] [MOVIES_FETCH]   {}. {} (ID: {})", i + 1, movie.name, movie.id);
                        }
                    }

                    all_movies.append(&mut movies);

                    // If we get movies from Cinemeta (first URL), that's sufficient
                    if index == 0 && !all_movies.is_empty() {
                        println!("[RUST] [MOVIES_FETCH] Got movies from primary source (Cinemeta), stopping here");
                        break;
                    }
                }
                Err(e) => {
                    let duration = start_time.elapsed();
                    last_error = format!("Failed to fetch from {}: {}", base_url, e);
                    println!("[RUST] [MOVIES_FETCH] ERROR: {} (after {:?})", last_error, duration);
                    continue;
                }
            }
        }

        if all_movies.is_empty() {
            let error_msg = format!("No movies found from any addon. Last error: {}", last_error);
            println!("[RUST] [MOVIES_FETCH] CRITICAL ERROR: {}", error_msg);
            return Err(error_msg);
        }

        println!("[RUST] [MOVIES_FETCH] Processing {} total movies...", all_movies.len());

        // Remove duplicates and limit results for better performance
        let original_count = all_movies.len();
        all_movies.sort_by(|a, b| a.id.cmp(&b.id));
        all_movies.dedup_by(|a, b| a.id == b.id);

        let after_dedup = all_movies.len();
        all_movies.truncate(50); // Limit to 50 for Steam Deck performance
        let final_count = all_movies.len();

        println!("[RUST] [MOVIES_FETCH] Movie processing complete:");
        println!("[RUST] [MOVIES_FETCH]   Original count: {}", original_count);
        println!("[RUST] [MOVIES_FETCH]   After deduplication: {}", after_dedup);
        println!("[RUST] [MOVIES_FETCH]   Final count (after truncation): {}", final_count);

        Ok(all_movies)
    }

    pub async fn fetch_popular_series(&self) -> Result<Vec<Series>, String> {
        println!("[RUST] [SERIES_FETCH] Starting to fetch popular series using correct Stremio v3 structure...");
        let mut all_series = Vec::new();

        println!("[RUST] [SERIES_FETCH] Correct endpoint: https://v3-cinemeta.strem.io/catalog/series/top.json");

        for base_url in &self.base_urls {
            match self.fetch_series_from_addon(base_url, "top").await {
                Ok(mut series) => {
                    all_series.append(&mut series);
                }
                Err(e) => {
                    eprintln!("Failed to fetch series from {}: {}", base_url, e);
                    continue;
                }
            }
        }

        if all_series.is_empty() {
            return Err("No series found from any addon".to_string());
        }

        // Remove duplicates and limit results
        all_series.sort_by(|a, b| a.id.cmp(&b.id));
        all_series.dedup_by(|a, b| a.id == b.id);
        all_series.truncate(100);

        Ok(all_series)
    }

    pub async fn fetch_popular_anime(&self) -> Result<Vec<Anime>, String> {
        println!("[RUST] [ANIME_FETCH] Starting to fetch anime using series endpoint with filtering...");
        let mut all_anime = Vec::new();

        println!("[RUST] [ANIME_FETCH] Using series/top endpoint and filtering for anime content");

        for base_url in &self.base_urls {
            match self.fetch_anime_from_addon(base_url, "top").await {
                Ok(mut anime) => {
                    all_anime.append(&mut anime);
                }
                Err(e) => {
                    eprintln!("Failed to fetch anime from {}: {}", base_url, e);
                    continue;
                }
            }
        }

        if all_anime.is_empty() {
            return Err("No anime found from any addon".to_string());
        }

        // Remove duplicates and limit results
        all_anime.sort_by(|a, b| a.id.cmp(&b.id));
        all_anime.dedup_by(|a, b| a.id == b.id);
        all_anime.truncate(100);

        Ok(all_anime)
    }

    pub async fn search_content(&self, query: &str) -> Result<Vec<SearchResult>, String> {
        println!("[RUST] [SEARCH] Starting comprehensive search for query: '{}'", query);

        if query.len() < 2 {
            println!("[RUST] [SEARCH] Query too short, returning empty results");
            return Ok(Vec::new());
        }

        let mut all_results = Vec::new();

        // Search movies, series, and anime from all addons
        for base_url in &self.base_urls {
            println!("[RUST] [SEARCH] Searching in addon: {}", base_url);

            // Search movies
            match self.search_movies_from_addon(base_url, query).await {
                Ok(mut movie_results) => {
                    println!("[RUST] [SEARCH] Found {} movie results from {}", movie_results.len(), base_url);
                    all_results.append(&mut movie_results);
                }
                Err(e) => {
                    println!("[RUST] [SEARCH] Failed to search movies from {}: {}", base_url, e);
                }
            }

            // Search series
            match self.search_series_from_addon(base_url, query).await {
                Ok(mut series_results) => {
                    println!("[RUST] [SEARCH] Found {} series results from {}", series_results.len(), base_url);
                    all_results.append(&mut series_results);
                }
                Err(e) => {
                    println!("[RUST] [SEARCH] Failed to search series from {}: {}", base_url, e);
                }
            }
        }

        // Apply anime detection logic
        for result in &mut all_results {
            if self.is_anime_content(result) {
                result.content_type = "anime".to_string();
                println!("[RUST] [SEARCH] Detected anime content: {}", result.name);
            }
        }

        // Remove duplicates and limit results
        let original_count = all_results.len();
        all_results.sort_by(|a, b| a.id.cmp(&b.id));
        all_results.dedup_by(|a, b| a.id == b.id);
        all_results.truncate(100); // Increased limit for search results

        println!("[RUST] [SEARCH] Search complete: {} results after deduplication (from {} original)",
                all_results.len(), original_count);

        Ok(all_results)
    }

    pub async fn fetch_streams(&self, imdb_id: &str) -> Result<Vec<Stream>, String> {
        println!("[RUST] [STREAMS_FETCH] Starting to fetch streams for IMDB ID: {}", imdb_id);
        let mut all_streams = Vec::new();

        for base_url in &self.base_urls {
            let stream_url = if base_url.contains("v3-cinemeta.strem.io") {
                format!("{}/stream/movie/{}.json", base_url, imdb_id)
            } else {
                format!("{}/stream/movie/{}.json", base_url, imdb_id)
            };

            println!("[RUST] [STREAMS_FETCH] Trying stream endpoint: {}", stream_url);

            match self.fetch_streams_from_addon(base_url, imdb_id).await {
                Ok(mut streams) => {
                    println!("[RUST] [STREAMS_FETCH] Found {} streams from {}", streams.len(), base_url);
                    all_streams.append(&mut streams);
                }
                Err(e) => {
                    println!("[RUST] [STREAMS_FETCH] Failed to fetch streams from {}: {}", base_url, e);
                    continue;
                }
            }
        }

        if all_streams.is_empty() {
            return Err("No streams found for this content".to_string());
        }

        // Sort streams by quality score
        all_streams.sort_by(|a, b| self.calculate_stream_quality_score(b).partial_cmp(&self.calculate_stream_quality_score(a)).unwrap_or(std::cmp::Ordering::Equal));

        Ok(all_streams)
    }

    async fn fetch_movies_from_addon(
        &self,
        base_url: &str,
        catalog: &str,
    ) -> Result<Vec<Movie>, String> {
        let url = format!("{}/catalog/movie/{}.json", base_url, catalog);
        println!("[RUST] [HTTP] Making request to correct Stremio endpoint: {}", url);

        let start_time = std::time::Instant::now();
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| {
                let error_msg = format!("Network error: {}", e);
                println!("[RUST] [HTTP] ERROR: {}", error_msg);
                error_msg
            })?;

        let request_duration = start_time.elapsed();
        println!("[RUST] [HTTP] Request completed in {:?}, status: {}", request_duration, response.status());

        if !response.status().is_success() {
            let error_msg = format!("HTTP error: {}", response.status());
            println!("[RUST] [HTTP] ERROR: {}", error_msg);
            return Err(error_msg);
        }

        println!("[RUST] [HTTP] Parsing JSON response...");
        let json_start = std::time::Instant::now();
        let json: Value = response
            .json()
            .await
            .map_err(|e| {
                let error_msg = format!("JSON parse error: {}", e);
                println!("[RUST] [HTTP] ERROR: {}", error_msg);
                error_msg
            })?;

        let json_duration = json_start.elapsed();
        println!("[RUST] [HTTP] JSON parsing completed in {:?}", json_duration);

        // Log some details about the JSON structure
        if let Some(metas) = json.get("metas") {
            if let Some(metas_array) = metas.as_array() {
                println!("[RUST] [PARSE] Found {} metas in response", metas_array.len());
            }
        } else {
            println!("[RUST] [PARSE] WARNING: No 'metas' field found in response");
            println!("[RUST] [PARSE] Available fields: {:?}", json.as_object().map(|o| o.keys().collect::<Vec<_>>()));
        }

        println!("[RUST] [PARSE] Parsing movies from JSON...");
        let parse_start = std::time::Instant::now();
        let movies = self.parse_movies_from_json(json)?;
        let parse_duration = parse_start.elapsed();

        println!("[RUST] [PARSE] Parsed {} movies in {:?}", movies.len(), parse_duration);
        Ok(movies)
    }

    async fn fetch_streams_from_addon(
        &self,
        base_url: &str,
        imdb_id: &str,
    ) -> Result<Vec<Stream>, String> {
        let url = format!("{}/stream/movie/{}.json", base_url, imdb_id);
        
        let response = self
            .client
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

        let streams = self.parse_streams_from_json(json)?;
        Ok(streams)
    }

    fn parse_movies_from_json(&self, json: Value) -> Result<Vec<Movie>, String> {
        let metas = json
            .get("metas")
            .ok_or("Missing 'metas' field")?
            .as_array()
            .ok_or("'metas' is not an array")?;

        let mut movies = Vec::new();
        for meta in metas {
            if let Ok(movie) = self.parse_single_movie(meta) {
                movies.push(movie);
            }
        }

        Ok(movies)
    }

    async fn fetch_series_from_addon(
        &self,
        base_url: &str,
        catalog: &str,
    ) -> Result<Vec<Series>, String> {
        let url = format!("{}/catalog/series/{}.json", base_url, catalog);

        let response = self
            .client
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

        let series = self.parse_series_from_json(json)?;
        Ok(series)
    }

    async fn fetch_anime_from_addon(
        &self,
        base_url: &str,
        catalog: &str,
    ) -> Result<Vec<Anime>, String> {
        // For now, treat anime same as series - could be enhanced with anime-specific addons later
        let url = format!("{}/catalog/series/{}.json", base_url, catalog);

        let response = self
            .client
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

        let anime = self.parse_anime_from_json(json)?;
        Ok(anime)
    }

    // Search movies specifically
    async fn search_movies_from_addon(
        &self,
        base_url: &str,
        query: &str,
    ) -> Result<Vec<SearchResult>, String> {
        let encoded_query = urlencoding::encode(query);
        let url = format!("{}/catalog/movie/top/search={}.json", base_url, encoded_query);

        println!("[RUST] [SEARCH] Searching movies at correct endpoint: {}", url);

        let response = self
            .client
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

        let mut results = self.parse_search_results_from_json(json)?;

        // Set content type for movies
        for result in &mut results {
            if result.content_type.is_empty() || result.content_type == "movie" {
                result.content_type = "movie".to_string();
            }
        }

        Ok(results)
    }

    // Search series specifically
    async fn search_series_from_addon(
        &self,
        base_url: &str,
        query: &str,
    ) -> Result<Vec<SearchResult>, String> {
        let encoded_query = urlencoding::encode(query);
        let url = format!("{}/catalog/series/top/search={}.json", base_url, encoded_query);

        println!("[RUST] [SEARCH] Searching series at correct endpoint: {}", url);

        let response = self
            .client
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

        let mut results = self.parse_search_results_from_json(json)?;

        // Set content type for series
        for result in &mut results {
            if result.content_type.is_empty() || result.content_type == "series" {
                result.content_type = "series".to_string();
            }
        }

        Ok(results)
    }

    // Anime detection logic
    fn is_anime_content(&self, content: &SearchResult) -> bool {
        let name_lower = content.name.to_lowercase();
        let description_lower = content.description.as_ref().map(|d| d.to_lowercase()).unwrap_or_default();

        // Check for anime-specific keywords
        let anime_keywords = [
            "anime", "manga", "japanese", "japan", "studio ghibli", "toei", "madhouse",
            "pierrot", "bones", "wit studio", "mappa", "sunrise", "a-1 pictures",
            "production i.g", "shaft", "gainax", "trigger", "kyoto animation"
        ];

        // Check for popular anime titles
        let popular_anime = [
            "one piece", "naruto", "bleach", "dragon ball", "pokemon", "detective conan",
            "attack on titan", "demon slayer", "death note", "fullmetal alchemist",
            "spirited away", "my neighbor totoro", "princess mononoke", "howl's moving castle",
            "sword art online", "tokyo ghoul", "jujutsu kaisen", "my hero academia",
            "hunter x hunter", "fairy tail", "black clover", "violet evergarden",
            "cowboy bebop", "neon genesis evangelion", "akira", "ghost in the shell"
        ];

        // Check name and description for anime indicators
        for keyword in &anime_keywords {
            if name_lower.contains(keyword) || description_lower.contains(keyword) {
                return true;
            }
        }

        // Check for popular anime titles
        for anime_title in &popular_anime {
            if name_lower.contains(anime_title) {
                return true;
            }
        }

        // Check if it's from Japan (common anime indicator)
        if description_lower.contains("japan") && (name_lower.contains("animation") || description_lower.contains("animated")) {
            return true;
        }

        false
    }

    fn parse_series_from_json(&self, json: Value) -> Result<Vec<Series>, String> {
        let metas = json
            .get("metas")
            .ok_or("Missing 'metas' field")?
            .as_array()
            .ok_or("'metas' is not an array")?;

        let mut series = Vec::new();
        for meta in metas {
            if let Ok(parsed_series) = self.parse_single_series(meta) {
                series.push(parsed_series);
            }
        }

        Ok(series)
    }

    fn parse_anime_from_json(&self, json: Value) -> Result<Vec<Anime>, String> {
        let metas = json
            .get("metas")
            .ok_or("Missing 'metas' field")?
            .as_array()
            .ok_or("'metas' is not an array")?;

        let mut anime = Vec::new();
        for meta in metas {
            if let Ok(parsed_anime) = self.parse_single_anime(meta) {
                anime.push(parsed_anime);
            }
        }

        Ok(anime)
    }

    fn parse_search_results_from_json(&self, json: Value) -> Result<Vec<SearchResult>, String> {
        let metas = json
            .get("metas")
            .ok_or("Missing 'metas' field")?
            .as_array()
            .ok_or("'metas' is not an array")?;

        let mut results = Vec::new();
        for meta in metas {
            if let Ok(result) = self.parse_single_search_result(meta) {
                results.push(result);
            }
        }

        Ok(results)
    }

    fn parse_single_movie(&self, meta: &Value) -> Result<Movie, String> {
        let id = meta
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or("Missing movie id")?
            .to_string();

        let name = meta
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or("Missing movie name")?
            .to_string();

        let poster = meta
            .get("poster")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let background = meta
            .get("background")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let description = meta
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let year = meta
            .get("year")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let imdb_rating = meta
            .get("imdbRating")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let genre = meta
            .get("genre")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            });

        let content_type = meta
            .get("type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let director = meta
            .get("director")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            });

        let cast = meta
            .get("cast")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            });

        let runtime = meta
            .get("runtime")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let country = meta
            .get("country")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let language = meta
            .get("language")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(Movie {
            id,
            name,
            poster,
            background,
            description,
            year,
            imdb_rating,
            genre,
            content_type,
            director,
            cast,
            runtime,
            country,
            language,
        })
    }

    fn parse_streams_from_json(&self, json: Value) -> Result<Vec<Stream>, String> {
        let streams = json
            .get("streams")
            .ok_or("Missing 'streams' field")?
            .as_array()
            .ok_or("'streams' is not an array")?;

        let mut parsed_streams = Vec::new();
        for stream in streams {
            if let Ok(parsed) = self.parse_single_stream(stream) {
                parsed_streams.push(parsed);
            }
        }

        Ok(parsed_streams)
    }

    fn parse_single_stream(&self, stream: &Value) -> Result<Stream, String> {
        let url = stream
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or("Missing stream url")?
            .to_string();

        let title = stream
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown Stream")
            .to_string();

        let name = stream
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let quality = self.extract_quality_from_title(&title);
        let size = stream
            .get("size")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let seeders = stream
            .get("seeders")
            .and_then(|v| v.as_u64())
            .map(|n| n as u32);

        let leechers = stream
            .get("leechers")
            .and_then(|v| v.as_u64())
            .map(|n| n as u32);

        let source = stream
            .get("source")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let language = stream
            .get("language")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let subtitles = stream
            .get("subtitles")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            });

        Ok(Stream {
            name,
            title,
            url,
            behavior_hints: None,
            quality,
            size,
            seeders,
            leechers,
            source,
            language,
            subtitles,
        })
    }

    fn parse_single_series(&self, meta: &Value) -> Result<Series, String> {
        // Extract basic info (same as movies)
        let id = meta
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or("Missing series id")?
            .to_string();

        let name = meta
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or("Missing series name")?
            .to_string();

        let poster = meta
            .get("poster")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let background = meta
            .get("background")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let description = meta
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let year = meta
            .get("year")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let imdb_rating = meta
            .get("imdbRating")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let genre = meta
            .get("genre")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            });

        let content_type = meta
            .get("type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let director = meta
            .get("director")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            });

        let cast = meta
            .get("cast")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            });

        let runtime = meta
            .get("runtime")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let country = meta
            .get("country")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let language = meta
            .get("language")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Series-specific fields
        let seasons = meta
            .get("seasons")
            .and_then(|v| v.as_u64())
            .map(|n| n as u32);

        let episodes = meta
            .get("episodes")
            .and_then(|v| v.as_u64())
            .map(|n| n as u32);

        let status = meta
            .get("status")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let network = meta
            .get("network")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(Series {
            id,
            name,
            poster,
            background,
            description,
            year,
            imdb_rating,
            genre,
            content_type,
            director,
            cast,
            runtime,
            country,
            language,
            seasons,
            episodes,
            status,
            network,
        })
    }

    fn parse_single_anime(&self, meta: &Value) -> Result<Anime, String> {
        // Extract basic info (same as series)
        let id = meta
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or("Missing anime id")?
            .to_string();

        let name = meta
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or("Missing anime name")?
            .to_string();

        let poster = meta
            .get("poster")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let background = meta
            .get("background")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let description = meta
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let year = meta
            .get("year")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let imdb_rating = meta
            .get("imdbRating")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let genre = meta
            .get("genre")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            });

        let content_type = meta
            .get("type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let director = meta
            .get("director")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            });

        let cast = meta
            .get("cast")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            });

        let runtime = meta
            .get("runtime")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let country = meta
            .get("country")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let language = meta
            .get("language")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Anime-specific fields
        let seasons = meta
            .get("seasons")
            .and_then(|v| v.as_u64())
            .map(|n| n as u32);

        let episodes = meta
            .get("episodes")
            .and_then(|v| v.as_u64())
            .map(|n| n as u32);

        let status = meta
            .get("status")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let studio = meta
            .get("studio")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let mal_rating = meta
            .get("malRating")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let anime_type = meta
            .get("animeType")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(Anime {
            id,
            name,
            poster,
            background,
            description,
            year,
            imdb_rating,
            genre,
            content_type,
            director,
            cast,
            runtime,
            country,
            language,
            seasons,
            episodes,
            status,
            studio,
            mal_rating,
            anime_type,
        })
    }

    fn parse_single_search_result(&self, meta: &Value) -> Result<SearchResult, String> {
        let id = meta
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or("Missing search result id")?
            .to_string();

        let name = meta
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or("Missing search result name")?
            .to_string();

        let poster = meta
            .get("poster")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let year = meta
            .get("year")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let imdb_rating = meta
            .get("imdbRating")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let content_type = meta
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("movie")
            .to_string();

        let description = meta
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(SearchResult {
            id,
            name,
            poster,
            year,
            imdb_rating,
            content_type,
            description,
        })
    }

    fn extract_quality_from_title(&self, title: &str) -> Option<String> {
        let quality_patterns = [
            ("4K", 5),
            ("2160p", 5),
            ("1080p", 4),
            ("720p", 3),
            ("480p", 2),
            ("HDRip", 3),
            ("BluRay", 4),
            ("WEBRip", 3),
            ("CAM", 1),
            ("TS", 1),
        ];

        for (pattern, _score) in &quality_patterns {
            if title.to_uppercase().contains(pattern) {
                return Some(pattern.to_string());
            }
        }

        None
    }

    fn calculate_stream_quality_score(&self, stream: &Stream) -> f64 {
        let mut score = 0.0;

        // Quality score based on resolution
        if let Some(quality) = &stream.quality {
            score += match quality.as_str() {
                "4K" | "2160p" => 50.0,
                "1080p" => 40.0,
                "720p" => 30.0,
                "480p" => 20.0,
                "HDRip" | "BluRay" | "WEBRip" => 25.0,
                "CAM" | "TS" => 5.0,
                _ => 15.0,
            };
        }

        // Seeders/leechers ratio (for torrents)
        if let (Some(seeders), Some(leechers)) = (stream.seeders, stream.leechers) {
            if leechers > 0 {
                let ratio = seeders as f64 / leechers as f64;
                score += ratio.min(10.0); // Cap ratio bonus at 10
            } else if seeders > 0 {
                score += 10.0; // No leechers but has seeders
            }
        }

        // Size preference (reasonable file sizes get bonus)
        if let Some(size_str) = &stream.size {
            if let Ok(size_gb) = self.parse_size_to_gb(size_str) {
                // Prefer sizes between 1-8GB for movies, penalize very small or very large
                if size_gb >= 1.0 && size_gb <= 8.0 {
                    score += 5.0;
                } else if size_gb > 8.0 && size_gb <= 15.0 {
                    score += 2.0;
                } else if size_gb < 0.5 {
                    score -= 5.0; // Penalize very small files
                }
            }
        }

        score
    }

    fn parse_size_to_gb(&self, size_str: &str) -> Result<f64, std::num::ParseFloatError> {
        let size_upper = size_str.to_uppercase();

        if size_upper.contains("GB") {
            size_upper.replace("GB", "").trim().parse()
        } else if size_upper.contains("MB") {
            let mb: f64 = size_upper.replace("MB", "").trim().parse()?;
            Ok(mb / 1024.0)
        } else if size_upper.contains("TB") {
            let tb: f64 = size_upper.replace("TB", "").trim().parse()?;
            Ok(tb * 1024.0)
        } else {
            // Assume bytes
            let bytes: f64 = size_str.trim().parse()?;
            Ok(bytes / (1024.0 * 1024.0 * 1024.0))
        }
    }
    }
