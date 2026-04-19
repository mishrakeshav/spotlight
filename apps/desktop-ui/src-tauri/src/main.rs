#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use core_actions::execute;
use core_indexer::{build_index, IndexOptions};
use core_search::SearchEngine;
use shared_types::{SearchQuery, SearchResult};
use std::collections::HashMap;
use std::sync::RwLock;
use tauri::State;
use tracing::info;

#[derive(Default)]
struct AppState {
    engine: RwLock<SearchEngine>,
    by_id: RwLock<HashMap<String, SearchResult>>,
}

#[tauri::command]
fn search(
    state: State<'_, AppState>,
    query: String,
    limit: usize,
) -> Result<Vec<SearchResult>, String> {
    let engine = state.engine.read().map_err(|e| e.to_string())?;
    let response = engine.query(SearchQuery { text: query, limit });
    Ok(response.results)
}

#[tauri::command]
fn open_result(state: State<'_, AppState>, id: String) -> Result<String, String> {
    let map = state.by_id.read().map_err(|e| e.to_string())?;
    let result = map
        .get(&id)
        .ok_or_else(|| format!("unknown result id: {id}"))?;
    execute(result).map_err(|e| e.to_string())
}

fn load_state() -> Result<AppState, String> {
    let (seed, stats) = build_index(&IndexOptions::default()).map_err(|e| e.to_string())?;
    info!(
        total_items = stats.total_items,
        source = stats.source,
        "desktop-ui index build complete"
    );

    let by_id = seed
        .iter()
        .cloned()
        .map(|item| (item.id.clone(), item))
        .collect::<HashMap<_, _>>();

    Ok(AppState {
        engine: RwLock::new(SearchEngine::with_seed(seed)),
        by_id: RwLock::new(by_id),
    })
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .without_time()
        .init();

    let state = load_state().expect("failed to load initial index for desktop-ui");

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![search, open_result])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
