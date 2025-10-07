use crate::models::{Movie, Series, Anime, Stream, SearchResult};
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

        // Use only the two most reliable streaming addon sources
        // This reduces fake/mislabeled torrents and improves stream quality
        let base_urls = vec![
            "https://v3-cinemeta.strem.io".to_string(),                    // Metadata only (movies/series info)
            "https://torrentio.strem.fun".to_string(),                     // Primary torrent source (most reliable)
            "https://thepiratebay-plus.strem.fun".to_string(),             // TPB torrents (backup source)
        ];

        Self { client, base_urls }
    }

    pub async fn fetch_popular_movies(&self) -> Result<Vec<Movie>, String> {
        println!("[RUST] [MOVIES_FETCH] Starting to fetch popular movies from real streaming sources...");
        let mut all_movies = Vec::new();
        let mut last_error = String::new();

        println!("[RUST] [MOVIES_FETCH] Available streaming addon URLs: {:?}", self.base_urls);
        println!("[RUST] [MOVIES_FETCH] Primary metadata source: https://v3-cinemeta.strem.io/catalog/movie/top.json");
        println!("[RUST] [MOVIES_FETCH] Torrent sources: Torrentio (primary), ThePirateBay+ (backup)");

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
        println!("[RUST] [ANIME_FETCH] Starting to fetch anime from both movies and series endpoints...");
        let mut all_anime = Vec::new();

        for base_url in &self.base_urls {
            // Fetch from series catalog
            println!("[RUST] [ANIME_FETCH] Fetching anime from series catalog: {}", base_url);
            match self.fetch_anime_from_addon(base_url, "top").await {
                Ok(mut anime) => {
                    println!("[RUST] [ANIME_FETCH] Found {} anime series from {}", anime.len(), base_url);
                    all_anime.append(&mut anime);
                }
                Err(e) => {
                    eprintln!("Failed to fetch anime series from {}: {}", base_url, e);
                }
            }

            // Also fetch from movie catalog and filter for anime
            println!("[RUST] [ANIME_FETCH] Fetching anime from movie catalog: {}", base_url);
            match self.fetch_anime_movies_from_addon(base_url, "top").await {
                Ok(mut anime_movies) => {
                    println!("[RUST] [ANIME_FETCH] Found {} anime movies from {}", anime_movies.len(), base_url);
                    all_anime.append(&mut anime_movies);
                }
                Err(e) => {
                    eprintln!("Failed to fetch anime movies from {}: {}", base_url, e);
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

        println!("[RUST] [ANIME_FETCH] Total anime after deduplication: {}", all_anime.len());

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
        println!("[RUST] [STREAMS_FETCH] Starting to fetch streams for IMDB ID: {} from {} sources", imdb_id, self.base_urls.len());
        let mut all_streams = Vec::new();
        let mut successful_sources = 0;
        let mut failed_sources = Vec::new();

        for base_url in &self.base_urls {
            // Skip metadata-only sources for stream fetching
            if base_url.contains("v3-cinemeta.strem.io") {
                println!("[RUST] [STREAMS_FETCH] Skipping metadata-only source: {}", base_url);
                continue;
            }

            let stream_url = format!("{}/stream/movie/{}.json", base_url, imdb_id);
            println!("[RUST] [STREAMS_FETCH] Trying torrent source: {}", stream_url);

            match self.fetch_streams_from_addon(base_url, imdb_id).await {
                Ok(mut streams) => {
                    println!("[RUST] [STREAMS_FETCH] Found {} streams from torrent source {}", streams.len(), base_url);
                    if !streams.is_empty() {
                        successful_sources += 1;
                        all_streams.append(&mut streams);
                    }
                }
                Err(e) => {
                    println!("[RUST] [STREAMS_FETCH] Failed to fetch streams from {}: {}", base_url, e);
                    failed_sources.push(base_url.clone());
                    continue;
                }
            }
        }

        println!("[RUST] [STREAMS_FETCH] Stream fetching summary: {} successful sources, {} failed sources",
                successful_sources, failed_sources.len());

        if all_streams.is_empty() {
            let error_msg = if failed_sources.len() == self.base_urls.len() - 1 { // -1 for cinemeta
                format!("No streaming sources available. All torrent addons failed: {:?}", failed_sources)
            } else {
                "No streams found for this content from any torrent source".to_string()
            };
            return Err(error_msg);
        }

        // Sort streams by quality score (best first)
        all_streams.sort_by(|a, b| {
            self.calculate_stream_quality_score(b).partial_cmp(&self.calculate_stream_quality_score(a))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        println!("[RUST] [STREAMS_FETCH] Returning {} total streams", all_streams.len());
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
        // Fetch anime from series catalog
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

    async fn fetch_anime_movies_from_addon(
        &self,
        base_url: &str,
        catalog: &str,
    ) -> Result<Vec<Anime>, String> {
        // Fetch from movie catalog and filter for anime
        let url = format!("{}/catalog/movie/{}.json", base_url, catalog);

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

        // Parse as anime and filter for anime content only
        let mut all_content = self.parse_anime_from_json(json)?;

        // Filter to keep only actual anime movies based on keywords
        all_content.retain(|anime| {
            let name_lower = anime.name.to_lowercase();
            let description_lower = anime.description.as_ref().map(|d| d.to_lowercase()).unwrap_or_default();

            // Use same anime detection logic
            let anime_keywords = [
                "anime", "manga", "japanese animation", "studio ghibli", "miyazaki",
                "pokemon", "naruto", "dragon ball", "one piece", "spirited away"
            ];

            anime_keywords.iter().any(|keyword| {
                name_lower.contains(keyword) || description_lower.contains(keyword)
            })
        });

        Ok(all_content)
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

        // First pass: collect all streams and group by infoHash to detect multi-file torrents
        let mut info_hash_counts: std::collections::HashMap<String, Vec<u64>> = std::collections::HashMap::new();

        for stream in streams {
            if let Some(info_hash) = stream.get("infoHash").and_then(|v| v.as_str()) {
                if let Some(file_idx) = stream.get("fileIdx").and_then(|v| v.as_u64()) {
                    info_hash_counts.entry(info_hash.to_string())
                        .or_insert_with(Vec::new)
                        .push(file_idx);
                }
            }
        }

        // Second pass: parse streams, but skip FileIdx 0 if multiple files exist for same infoHash
        for stream in streams {
            let should_skip = if let Some(info_hash) = stream.get("infoHash").and_then(|v| v.as_str()) {
                if let Some(file_idx) = stream.get("fileIdx").and_then(|v| v.as_u64()) {
                    // If this infoHash has multiple files AND this is FileIdx 0, skip it
                    if let Some(file_indices) = info_hash_counts.get(info_hash) {
                        if file_indices.len() > 1 && file_idx == 0 {
                            println!("[RUST] [STREAM_PARSE] ⚠️  Skipping FileIdx 0 from multi-file torrent (InfoHash: {}, total files: {})",
                                     info_hash, file_indices.len());
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            };

            if !should_skip {
                if let Ok(parsed) = self.parse_single_stream(stream) {
                    parsed_streams.push(parsed);
                }
            }
        }

        Ok(parsed_streams)
    }

    fn parse_single_stream(&self, stream: &Value) -> Result<Stream, String> {
        // Handle both direct URLs and torrent infoHash
        let url = if let Some(direct_url) = stream.get("url").and_then(|v| v.as_str()) {
            // Direct streaming URL
            direct_url.to_string()
        } else if let Some(info_hash) = stream.get("infoHash").and_then(|v| v.as_str()) {
            // Torrent magnet link
            let file_idx = stream.get("fileIdx").and_then(|v| v.as_u64()).unwrap_or(0);

            // Log the stream details to debug multi-file torrents
            let title_preview = stream.get("title").and_then(|v| v.as_str()).unwrap_or("No title");
            println!("[RUST] [STREAM_PARSE] InfoHash: {} | FileIdx: {} | Title: {}",
                     info_hash, file_idx, title_preview);

            if file_idx > 0 {
                format!("magnet:?xt=urn:btih:{}&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337%2Fannounce&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce&so={}", info_hash, file_idx - 1)
            } else {
                format!("magnet:?xt=urn:btih:{}&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337%2Fannounce&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce", info_hash)
            }
        } else {
            return Err("Missing stream url or infoHash".to_string());
        };

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

        // Extract size from torrent title or size field
        let size = if let Some(size_field) = stream.get("size").and_then(|v| v.as_str()) {
            Some(size_field.to_string())
        } else {
            // Try to extract size from title (e.g., "💾 5.09 GB")
            self.extract_size_from_title(&title)
        };

        // Extract seeders from field or title (e.g., "👤 12")
        let seeders = if let Some(seeders_field) = stream.get("seeders").and_then(|v| v.as_u64()) {
            Some(seeders_field as u32)
        } else {
            self.extract_seeders_from_title(&title)
        };

        let leechers = stream
            .get("leechers")
            .and_then(|v| v.as_u64())
            .map(|n| n as u32);

        // Set source based on whether it's a torrent or direct stream
        let source = if stream.get("infoHash").is_some() {
            Some("torrent".to_string())
        } else {
            stream
                .get("source")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| Some("direct".to_string()))
        };

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
        let title_upper = title.to_uppercase();

        // Enhanced quality patterns for torrent sources (ordered by preference)
        let quality_patterns = [
            // 4K variants
            ("2160P", 6),
            ("4K", 6),
            ("UHD", 6),
            // High quality 1080p variants
            ("1080P BLURAY", 5),
            ("1080P REMUX", 5),
            ("1080P", 4),
            // 720p variants
            ("720P BLURAY", 4),
            ("720P", 3),
            // Other formats
            ("BLURAY", 4),
            ("WEBDL", 3),
            ("WEBRIP", 3),
            ("HDRIP", 3),
            ("BRRIP", 3),
            ("480P", 2),
            ("DVDRIP", 2),
            // Low quality
            ("CAM", 1),
            ("TS", 1),
            ("HDTS", 1),
            ("SCREENER", 1),
        ];

        // Find the best matching quality indicator
        for (pattern, _score) in &quality_patterns {
            if title_upper.contains(pattern) {
                return Some(pattern.to_string());
            }
        }

        // Check for resolution patterns like "x264", "x265", "H264", "H265"
        if title_upper.contains("X265") || title_upper.contains("H265") || title_upper.contains("HEVC") {
            return Some("H265".to_string());
        }
        if title_upper.contains("X264") || title_upper.contains("H264") {
            return Some("H264".to_string());
        }

        None
    }

    fn calculate_stream_quality_score(&self, stream: &Stream) -> f64 {
        let mut score = 0.0;

        // Enhanced quality score based on torrent stream quality indicators
        if let Some(quality) = &stream.quality {
            score += match quality.as_str() {
                // 4K and UHD content
                "4K" | "2160P" | "UHD" => 60.0,
                // High quality 1080p
                "1080P BLURAY" | "1080P REMUX" => 55.0,
                "1080P" => 45.0,
                // 720p variants
                "720P BLURAY" => 40.0,
                "720P" => 35.0,
                // Good quality sources
                "BLURAY" => 42.0,
                "WEBDL" => 30.0,
                "WEBRIP" | "HDRIP" | "BRRIP" => 25.0,
                // Standard definition
                "480P" | "DVDRIP" => 15.0,
                // Encoding quality bonus
                "H265" | "X265" | "HEVC" => 35.0,
                "H264" | "X264" => 30.0,
                // Low quality
                "CAM" | "TS" | "HDTS" | "SCREENER" => 5.0,
                _ => 20.0,
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

    fn extract_size_from_title(&self, title: &str) -> Option<String> {
        use regex::Regex;

        // Pattern to match size in torrent titles (e.g., "💾 5.09 GB", "1.78 GB", "15.27 GB")
        if let Ok(re) = Regex::new(r"💾\s*([0-9.]+\s*[KMGT]?B)|([0-9.]+\s*[KMGT]?B)") {
            if let Some(captures) = re.find(title) {
                let size_text = captures.as_str().replace("💾", "").trim().to_string();
                return Some(size_text);
            }
        }
        None
    }

    fn extract_seeders_from_title(&self, title: &str) -> Option<u32> {
        use regex::Regex;

        // Pattern to match seeders in torrent titles (e.g., "👤 12", "👤12")
        if let Ok(re) = Regex::new(r"👤\s*([0-9]+)") {
            if let Some(captures) = re.captures(title) {
                if let Some(seeders_str) = captures.get(1) {
                    if let Ok(seeders) = seeders_str.as_str().parse::<u32>() {
                        return Some(seeders);
                    }
                }
            }
        }
        None
    }
}
