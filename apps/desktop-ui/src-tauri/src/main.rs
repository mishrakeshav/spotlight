#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use core_actions::{execute, extract_path, reveal};
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
    usage: RwLock<HashMap<String, u32>>,
    recents: RwLock<Vec<String>>,
}

#[tauri::command]
fn search(
    state: State<'_, AppState>,
    query: String,
    limit: usize,
) -> Result<Vec<SearchResult>, String> {
    let engine = state.engine.read().map_err(|e| e.to_string())?;
    let mut results = engine.query(SearchQuery { text: query, limit }).results;

    let usage = state.usage.read().map_err(|e| e.to_string())?;
    let recents = state.recents.read().map_err(|e| e.to_string())?;

    for result in &mut results {
        if let Some(count) = usage.get(&result.id) {
            result.score += ((*count).min(20) as f32) * 0.12;
        }

        if let Some(position) = recents.iter().position(|id| id == &result.id) {
            let recency = (20usize.saturating_sub(position) as f32) * 0.08;
            result.score += recency;
        }
    }

    results.sort_by(|a, b| b.score.total_cmp(&a.score));
    results.truncate(limit);
    Ok(results)
}

#[tauri::command]
fn open_result(state: State<'_, AppState>, id: String) -> Result<String, String> {
    let map = state.by_id.read().map_err(|e| e.to_string())?;
    let result = map
        .get(&id)
        .ok_or_else(|| format!("unknown result id: {id}"))?;
    let output = execute(result).map_err(|e| e.to_string())?;
    drop(map);
    update_usage(&state, &id)?;
    Ok(output)
}

#[tauri::command]
fn reveal_result(state: State<'_, AppState>, id: String) -> Result<String, String> {
    let map = state.by_id.read().map_err(|e| e.to_string())?;
    let result = map
        .get(&id)
        .ok_or_else(|| format!("unknown result id: {id}"))?;
    reveal(result).map_err(|e| e.to_string())
}

#[tauri::command]
fn path_for_result(state: State<'_, AppState>, id: String) -> Result<Option<String>, String> {
    let map = state.by_id.read().map_err(|e| e.to_string())?;
    let result = map
        .get(&id)
        .ok_or_else(|| format!("unknown result id: {id}"))?;
    Ok(extract_path(result))
}

fn update_usage(state: &State<'_, AppState>, id: &str) -> Result<(), String> {
    let mut usage = state.usage.write().map_err(|e| e.to_string())?;
    let count = usage.entry(id.to_string()).or_insert(0);
    *count += 1;
    drop(usage);

    let mut recents = state.recents.write().map_err(|e| e.to_string())?;
    recents.retain(|existing| existing != id);
    recents.insert(0, id.to_string());
    if recents.len() > 50 {
        recents.truncate(50);
    }
    Ok(())
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
        usage: RwLock::new(HashMap::new()),
        recents: RwLock::new(Vec::new()),
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
        .invoke_handler(tauri::generate_handler![
            search,
            open_result,
            reveal_result,
            path_for_result
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
