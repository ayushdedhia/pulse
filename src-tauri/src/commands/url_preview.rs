use crate::db::Database;
use crate::models::UrlPreview;
use regex::Regex;
use scraper::{Html, Selector};
use std::time::Duration;
use tauri::State;

const FETCH_TIMEOUT_SECS: u64 = 3;
const CACHE_TTL_SECS: i64 = 3600; // 1 hour

/// Extract the first HTTPS URL from text content
pub fn extract_first_url(content: &str) -> Option<String> {
    let url_regex = Regex::new(
        r"https://[a-zA-Z0-9\-._~:/?#\[\]@!$&'()*+,;=%]+"
    ).ok()?;

    url_regex
        .find(content)
        .map(|m| m.as_str().to_string())
}

/// Get cached URL preview from database
pub fn get_cached_preview(conn: &rusqlite::Connection, url: &str) -> Option<UrlPreview> {
    let now = chrono::Utc::now().timestamp();

    conn.query_row(
        "SELECT url, title, description, image_url, site_name, fetched_at
         FROM url_previews
         WHERE url = ?1 AND fetched_at > ?2",
        [url, &(now - CACHE_TTL_SECS).to_string()],
        |row| {
            Ok(UrlPreview {
                url: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                image_url: row.get(3)?,
                site_name: row.get(4)?,
                fetched_at: row.get(5)?,
            })
        },
    )
    .ok()
}

/// Cache URL preview in database
pub fn cache_preview(conn: &rusqlite::Connection, preview: &UrlPreview) -> Result<(), String> {
    conn.execute(
        "INSERT OR REPLACE INTO url_previews (url, title, description, image_url, site_name, fetched_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            &preview.url,
            &preview.title,
            &preview.description,
            &preview.image_url,
            &preview.site_name,
            preview.fetched_at,
        ),
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// Fetch URL preview metadata (Open Graph tags)
pub async fn fetch_url_preview(url: &str) -> Result<UrlPreview, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(FETCH_TIMEOUT_SECS))
        .redirect(reqwest::redirect::Policy::limited(3))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (compatible; PulseBot/1.0)")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let html = response.text().await.map_err(|e| e.to_string())?;
    let document = Html::parse_document(&html);

    // Selectors for Open Graph and fallback meta tags
    let og_title_selector = Selector::parse("meta[property='og:title']").ok();
    let og_desc_selector = Selector::parse("meta[property='og:description']").ok();
    let og_image_selector = Selector::parse("meta[property='og:image']").ok();
    let og_site_selector = Selector::parse("meta[property='og:site_name']").ok();
    let title_selector = Selector::parse("title").ok();
    let desc_selector = Selector::parse("meta[name='description']").ok();

    let get_meta_content = |selector: Option<Selector>| -> Option<String> {
        selector.and_then(|s| {
            document
                .select(&s)
                .next()
                .and_then(|el| el.value().attr("content"))
                .map(|s| s.to_string())
        })
    };

    let title = get_meta_content(og_title_selector).or_else(|| {
        title_selector.and_then(|s| {
            document.select(&s).next().map(|el| el.text().collect())
        })
    });

    let description = get_meta_content(og_desc_selector)
        .or_else(|| get_meta_content(desc_selector));

    let image_url = get_meta_content(og_image_selector);
    let site_name = get_meta_content(og_site_selector);

    Ok(UrlPreview {
        url: url.to_string(),
        title,
        description,
        image_url,
        site_name,
        fetched_at: chrono::Utc::now().timestamp(),
    })
}

/// Get or fetch URL preview (with caching)
#[allow(dead_code)]
pub async fn get_or_fetch_preview(
    conn: &rusqlite::Connection,
    url: &str,
) -> Option<UrlPreview> {
    // Check cache first
    if let Some(cached) = get_cached_preview(conn, url) {
        return Some(cached);
    }

    // Fetch and cache
    match fetch_url_preview(url).await {
        Ok(preview) => {
            let _ = cache_preview(conn, &preview);
            Some(preview)
        }
        Err(e) => {
            tracing::warn!("Failed to fetch URL preview for {}: {}", url, e);
            None
        }
    }
}

/// Tauri command to manually fetch URL preview
#[tauri::command]
pub async fn fetch_preview(db: State<'_, Database>, url: String) -> Result<Option<UrlPreview>, String> {
    // Validate URL
    if !url.starts_with("https://") {
        return Err("Only HTTPS URLs are supported".to_string());
    }

    // Check cache first (with lock)
    let cached = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        get_cached_preview(&conn, &url)
    }; // Lock released here

    if let Some(preview) = cached {
        return Ok(Some(preview));
    }

    // Fetch (async, no lock)
    let preview = fetch_url_preview(&url).await.ok();

    // Cache the result (with lock)
    if let Some(ref p) = preview {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let _ = cache_preview(&conn, p);
    }

    Ok(preview)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_first_url_basic() {
        let content = "Check out https://example.com for more info";
        let url = extract_first_url(content);
        assert_eq!(url, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_extract_first_url_with_path() {
        let content = "Visit https://example.com/path/to/page.html";
        let url = extract_first_url(content);
        assert_eq!(url, Some("https://example.com/path/to/page.html".to_string()));
    }

    #[test]
    fn test_extract_first_url_with_query() {
        let content = "Link: https://example.com/search?q=test&page=1";
        let url = extract_first_url(content);
        assert_eq!(url, Some("https://example.com/search?q=test&page=1".to_string()));
    }

    #[test]
    fn test_extract_first_url_with_fragment() {
        let content = "See https://example.com/page#section-1";
        let url = extract_first_url(content);
        assert_eq!(url, Some("https://example.com/page#section-1".to_string()));
    }

    #[test]
    fn test_extract_first_url_multiple_urls() {
        let content = "First https://first.com and second https://second.com";
        let url = extract_first_url(content);
        assert_eq!(url, Some("https://first.com".to_string()));
    }

    #[test]
    fn test_extract_first_url_no_url() {
        let content = "This message has no URL";
        let url = extract_first_url(content);
        assert_eq!(url, None);
    }

    #[test]
    fn test_extract_first_url_http_ignored() {
        let content = "Not secure: http://example.com";
        let url = extract_first_url(content);
        assert_eq!(url, None);
    }

    #[test]
    fn test_extract_first_url_subdomain() {
        let content = "Visit https://sub.domain.example.com/page";
        let url = extract_first_url(content);
        assert_eq!(url, Some("https://sub.domain.example.com/page".to_string()));
    }

    #[test]
    fn test_extract_first_url_with_port() {
        let content = "Local: https://localhost:8080/api";
        let url = extract_first_url(content);
        assert_eq!(url, Some("https://localhost:8080/api".to_string()));
    }

    #[test]
    fn test_extract_first_url_encoded_chars() {
        let content = "Encoded: https://example.com/path%20with%20spaces";
        let url = extract_first_url(content);
        assert_eq!(url, Some("https://example.com/path%20with%20spaces".to_string()));
    }

    #[test]
    fn test_extract_first_url_github() {
        let content = "Check the repo: https://github.com/user/repo/blob/main/README.md";
        let url = extract_first_url(content);
        assert_eq!(url, Some("https://github.com/user/repo/blob/main/README.md".to_string()));
    }

    #[test]
    fn test_extract_first_url_youtube() {
        let content = "Watch this: https://www.youtube.com/watch?v=dQw4w9WgXcQ";
        let url = extract_first_url(content);
        assert_eq!(url, Some("https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string()));
    }

    #[test]
    fn test_extract_first_url_at_start() {
        let content = "https://example.com is a great site";
        let url = extract_first_url(content);
        assert_eq!(url, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_extract_first_url_at_end() {
        let content = "Check out https://example.com";
        let url = extract_first_url(content);
        assert_eq!(url, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_extract_first_url_multiline() {
        let content = "First line\nhttps://example.com\nLast line";
        let url = extract_first_url(content);
        assert_eq!(url, Some("https://example.com".to_string()));
    }
}
