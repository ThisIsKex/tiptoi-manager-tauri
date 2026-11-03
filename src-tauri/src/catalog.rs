//! Katalog-Client für das öffentliche Ravensburger Serviceportal
//! (service.ravensburger.de, MindTouch/NICE CXone Expert).
//!
//! Nutzt ausschließlich öffentlich erreichbare, unauthentifizierte
//! REST-Endpunkte (`@api/deki/pages/...`), die die Website selbst im
//! Browser verwendet, um die Seite "tiptoi® Audiodateien" aufzubauen. Es
//! werden keine Zugangsdaten, Tokens oder privaten/undokumentierten APIs
//! verwendet – die `.gme`-Downloadlinks sind exakt dieselben, die ein
//! Nutzer auch manuell über die Serviceportal-Seite anklicken könnte:
//! <https://service.ravensburger.de/tiptoi®/tiptoi®_Audiodateien>
//!
//! Ablauf (zweistufig, wie auf der Webseite selbst):
//! 1. Zunächst wird nur die Liste der Kategorien geladen (ein einzelner
//!    Request auf die Wurzelseite "tiptoi® Audiodateien") – analog zur
//!    Kategorie-Navigation auf der Serviceportal-Seite.
//! 2. Erst wenn eine Kategorie ausgewählt wird, werden deren Unterseiten
//!    rekursiv abgefragt (`.../subpages?format=json&limit=all`), bis nur
//!    noch Produkt-Endseiten (`subpages="false"`) übrig sind, und für jede
//!    Produktseite deren Inhalt (`.../contents?format=json`), um daraus den
//!    `.gme`-Downloadlink und das Vorschaubild per Pattern-Match zu
//!    extrahieren.
//!
//! So muss nicht bei jedem Start der komplette Katalog (alle Kategorien,
//! mehrere hundert Produktseiten) geladen werden, sondern nur die gerade
//! angeschaute Kategorie – spürbar schneller und schonender gegenüber dem
//! Serviceportal. Jede Kategorie wird separat auf Platte gecacht.

use futures_util::stream::{self, StreamExt};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};

const API_BASE: &str = "https://service.ravensburger.de/@api/deki";
/// Wurzelseite "tiptoi® Audiodateien" im Serviceportal.
const ROOT_PAGE_ID: &str = "1779";
const CACHE_TTL_SECS: u64 = 24 * 60 * 60;
/// Begrenzt gleichzeitige Requests, um das Serviceportal nicht zu überlasten.
const CONCURRENCY: usize = 8;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Product {
    pub id: String,
    pub name: String,
    pub thumbnail_url: Option<String>,
    pub gme_url: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Catalog {
    pub products: Vec<Product>,
    pub fetched_at: u64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct CategoryCache {
    categories: Vec<Category>,
    fetched_at: u64,
}

#[derive(Serialize, Clone)]
pub struct DownloadProgress {
    pub url: String,
    pub downloaded: u64,
    pub total: u64,
}

struct SubpageEntry {
    id: String,
    title: String,
    has_subpages: bool,
}

fn cache_dir() -> PathBuf {
    let base = dirs::cache_dir().unwrap_or_else(std::env::temp_dir);
    base.join("tiptoi-manager")
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn html_unescape(s: &str) -> String {
    let mut result = s.to_string();
    // Manche Felder sind doppelt HTML-entity-escaped (HTML-Attribut-Encoding
    // plus die XML/JSON-Hülle der API), daher mehrfach dekodieren.
    for _ in 0..3 {
        let next = result
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&nbsp;", " ");
        if next == result {
            break;
        }
        result = next;
    }
    result
}

fn subpage_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"<page\.subpage id="(\d+)"[^>]*subpages="(true|false)"[^>]*><uri\.ui>[^<]*</uri\.ui><title>([^<]*)</title>"#,
        )
        .expect("subpage regex is valid")
    })
}

fn gme_link_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"href="([^"]+\.gme)""#).expect("gme regex is valid"))
}

fn thumbnail_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"<img[^>]*\ssrc="([^"]+)""#).expect("img regex is valid"))
}

/// Fragt die direkten Unterseiten einer Serviceportal-Seite ab.
async fn fetch_subpages(client: &reqwest::Client, page_id: &str) -> Result<Vec<SubpageEntry>, String> {
    let url = format!("{API_BASE}/pages/{page_id}/subpages?format=json&limit=all");
    let text = client
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    Ok(subpage_regex()
        .captures_iter(&text)
        .map(|c| SubpageEntry {
            id: c[1].to_string(),
            has_subpages: &c[2] == "true",
            title: html_unescape(&c[3]),
        })
        .collect())
}

/// Läuft rekursiv/iterativ (Breitensuche) ab einer gegebenen Startseite über
/// den Seitenbaum, bis nur noch Produkt-Endseiten (ohne eigene Unterseiten)
/// übrig sind. Fehlerhafte Teiläste werden übersprungen, statt den gesamten
/// Crawl scheitern zu lassen (das Serviceportal ist kein stabiles
/// API-Vertragsverhältnis).
async fn crawl_product_pages(client: &reqwest::Client, start_id: &str) -> Vec<(String, String)> {
    let mut queue = vec![start_id.to_string()];
    let mut leaves = Vec::new();

    while !queue.is_empty() {
        let results: Vec<Result<Vec<SubpageEntry>, String>> = stream::iter(queue.drain(..).map(|id| {
            let client = client.clone();
            async move { fetch_subpages(&client, &id).await }
        }))
        .buffer_unordered(CONCURRENCY)
        .collect()
        .await;

        let mut next_queue = Vec::new();
        for result in results {
            let entries = match result {
                Ok(entries) => entries,
                Err(e) => {
                    log::warn!("tiptoi: Teilbaum des Katalogs konnte nicht geladen werden: {e}");
                    continue;
                }
            };
            for entry in entries {
                if entry.has_subpages {
                    next_queue.push(entry.id);
                } else {
                    leaves.push((entry.id, entry.title));
                }
            }
        }
        queue = next_queue;
    }

    leaves
}

/// Lädt Inhalt einer Produkt-Endseite und extrahiert `.gme`-Link + Vorschaubild.
async fn fetch_product(client: &reqwest::Client, id: String, title: String) -> Option<Product> {
    let url = format!("{API_BASE}/pages/{id}/contents?format=json");
    let raw_text = client
        .get(&url)
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .text()
        .await
        .ok()?;
    // Tags wie <img>/<a> sind HTML-entity-escaped (&lt;img ...&gt;); erst
    // nach dem Dekodieren lassen sie sich mit den Tag-Regexen erkennen.
    let text = html_unescape(&raw_text);

    let gme_url = gme_link_regex().captures(&text).map(|c| c[1].to_string());
    let thumbnail_url = thumbnail_regex().captures(&text).map(|c| c[1].to_string());

    // Produktseiten ohne erkennbaren .gme-Link sind z. B. reine
    // Übersichts-/Hinweisseiten – für den Katalog nicht relevant.
    if gme_url.is_none() {
        return None;
    }

    Some(Product {
        id,
        name: title,
        thumbnail_url,
        gme_url,
    })
}

fn http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent("tiptoi-manager-tauri/0.1 (privates Hobby-Projekt)")
        .build()
        .map_err(|e| e.to_string())
}

/// Baut den vollständigen Katalog *einer Kategorie* frisch auf.
async fn crawl_category_catalog(category_id: &str) -> Result<Catalog, String> {
    let client = http_client()?;
    let product_pages = crawl_product_pages(&client, category_id).await;

    let products: Vec<Product> = stream::iter(product_pages.into_iter().map(|(id, title)| {
        let client = client.clone();
        async move { fetch_product(&client, id, title).await }
    }))
    .buffer_unordered(CONCURRENCY)
    .filter_map(|p| async move { p })
    .collect()
    .await;

    Ok(Catalog {
        products,
        fetched_at: now_secs(),
    })
}

fn category_cache_file(dir: &std::path::Path, category_id: &str) -> PathBuf {
    // Kategorie-IDs sind numerisch (aus der Serviceportal-API), daher
    // unbedenklich direkt als Dateiname nutzbar.
    dir.join(format!("category-{category_id}.json"))
}

/// Lädt die Liste der Top-Level-Kategorien (z. B. "Bücher",
/// "Adventskalender", ...) von der Wurzelseite. Ein einzelner, günstiger
/// Request – die eigentlichen Produkte werden erst bei Bedarf pro Kategorie
/// nachgeladen (siehe [`get_category_catalog`]).
pub async fn get_categories(force_refresh: bool) -> Result<Vec<Category>, String> {
    let dir = cache_dir();
    let _ = tokio::fs::create_dir_all(&dir).await;
    let cache_file = dir.join("categories.json");

    if !force_refresh {
        if let Some(cached) = read_categories_cache(&cache_file).await {
            if now_secs().saturating_sub(cached.fetched_at) < CACHE_TTL_SECS {
                return Ok(cached.categories);
            }
        }
    }

    let client = http_client()?;
    let result = fetch_subpages(&client, ROOT_PAGE_ID).await.map(|entries| {
        entries
            .into_iter()
            .map(|e| Category { id: e.id, name: e.title })
            .collect::<Vec<_>>()
    });

    match result {
        Ok(categories) => {
            let cache = CategoryCache {
                categories: categories.clone(),
                fetched_at: now_secs(),
            };
            if let Ok(json) = serde_json::to_string(&cache) {
                let _ = tokio::fs::write(&cache_file, json).await;
            }
            Ok(categories)
        }
        Err(e) => match read_categories_cache(&cache_file).await {
            Some(cached) => Ok(cached.categories),
            None => Err(e),
        },
    }
}

async fn read_categories_cache(path: &PathBuf) -> Option<CategoryCache> {
    let contents = tokio::fs::read_to_string(path).await.ok()?;
    serde_json::from_str(&contents).ok()
}

/// Lädt den Produktkatalog einer einzelnen Kategorie, mit 24h-Disk-Cache und
/// Offline-Fallback auf einen abgelaufenen Cache, falls der Crawl
/// fehlschlägt (z. B. kein Internet).
pub async fn get_category_catalog(category_id: &str, force_refresh: bool) -> Result<Catalog, String> {
    let dir = cache_dir();
    let _ = tokio::fs::create_dir_all(&dir).await;
    let cache_file = category_cache_file(&dir, category_id);

    if !force_refresh {
        if let Some(cached) = read_cache(&cache_file).await {
            if now_secs().saturating_sub(cached.fetched_at) < CACHE_TTL_SECS {
                return Ok(cached);
            }
        }
    }

    match crawl_category_catalog(category_id).await {
        Ok(catalog) => {
            if let Ok(json) = serde_json::to_string(&catalog) {
                let _ = tokio::fs::write(&cache_file, json).await;
            }
            Ok(catalog)
        }
        Err(e) => match read_cache(&cache_file).await {
            Some(cached) => Ok(cached),
            None => Err(e),
        },
    }
}

async fn read_cache(path: &PathBuf) -> Option<Catalog> {
    let contents = tokio::fs::read_to_string(path).await.ok()?;
    serde_json::from_str(&contents).ok()
}

/// Lädt eine `.gme`-Datei (öffentliche, unauthentifizierte URL) mit
/// Fortschritts-Events auf den gewünschten Zielpfad herunter.
pub async fn download_file(app: &AppHandle, url: &str, dest_path: &str) -> Result<(), String> {
    use futures_util::StreamExt as _;

    let dest = PathBuf::from(dest_path);
    let temp_dest = PathBuf::from(format!("{dest_path}.download"));

    let client = http_client()?;
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;

    let total = resp.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut stream = resp.bytes_stream();

    let mut file = tokio::fs::File::create(&temp_dest)
        .await
        .map_err(|e| e.to_string())?;

    use tokio::io::AsyncWriteExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        file.write_all(&chunk).await.map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;

        let _ = app.emit(
            "download-progress",
            DownloadProgress {
                url: url.to_string(),
                downloaded,
                total,
            },
        );
    }

    file.flush().await.map_err(|e| e.to_string())?;
    drop(file);

    tokio::fs::rename(&temp_dest, &dest)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn smoke_test_small_crawl() {
        let client = http_client().unwrap();
        let subpages = fetch_subpages(&client, "1780").await.unwrap();
        assert!(!subpages.is_empty());
        let first = &subpages[0];
        let product = fetch_product(&client, first.id.clone(), first.title.clone()).await;
        println!("{:?}", product.as_ref().map(|p| (&p.name, &p.gme_url, &p.thumbnail_url)));
        assert!(product.is_some());
    }

    #[tokio::test]
    #[ignore]
    async fn smoke_test_categories() {
        let categories = get_categories(true).await.unwrap();
        assert!(!categories.is_empty());
        println!("{} Kategorien gefunden", categories.len());
        for c in &categories {
            println!("  {} -> {}", c.id, c.name);
        }
    }
}
