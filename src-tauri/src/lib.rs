mod catalog;
mod pen;

use catalog::{Catalog, Category};
use pen::PenInfo;
use tauri::AppHandle;

/// Sucht nach einem angeschlossenen tiptoi-Stift (optional an einem expliziten Pfad).
#[tauri::command]
fn detect_pen(explicit_path: Option<String>) -> Option<PenInfo> {
    pen::find_connected_pen(explicit_path.as_deref())
}

/// Hängt den Stift sicher aus.
#[tauri::command]
fn eject_pen(mount_path: String) -> bool {
    pen::eject_pen(&mount_path)
}

/// Lädt die Liste der Katalog-Kategorien (z. B. "Bücher", "Adventskalender").
#[tauri::command]
async fn get_categories(force_refresh: bool) -> Result<Vec<Category>, String> {
    catalog::get_categories(force_refresh).await
}

/// Lädt den Produktkatalog einer einzelnen Kategorie vom öffentlichen
/// Ravensburger Serviceportal (mit 24h-Disk-Cache).
#[tauri::command]
async fn get_category_catalog(category_id: String, force_refresh: bool) -> Result<Catalog, String> {
    catalog::get_category_catalog(&category_id, force_refresh).await
}

/// Lädt eine `.gme`-Datei herunter und emittiert `download-progress`-Events.
#[tauri::command]
async fn download_book(app: AppHandle, url: String, dest_path: String) -> Result<(), String> {
    catalog::download_file(&app, &url, &dest_path).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            detect_pen,
            eject_pen,
            get_categories,
            get_category_catalog,
            download_book,
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while building tauri application");
}
