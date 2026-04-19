use anyhow::Result;
use core_actions::execute;
use core_indexer::{build_index, IndexOptions};
use core_search::SearchEngine;
use shared_types::SearchQuery;
use std::env;
use tracing::info;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .without_time()
        .init();

    let query_text = env::args().nth(1).unwrap_or_else(|| "term".to_string());

    let options = IndexOptions::default();
    let (seed, stats) = build_index(&options)?;
    info!(
        total_items = stats.total_items,
        source = stats.source,
        "index build complete"
    );

    let engine = SearchEngine::with_seed(seed);
    let response = engine.query(SearchQuery {
        text: query_text,
        limit: 10,
    });

    info!(
        query = response.query,
        took_ms = response.took_ms,
        results = response.results.len(),
        "query complete"
    );

    for (idx, result) in response.results.iter().take(5).enumerate() {
        info!(
            rank = idx + 1,
            title = result.title,
            score = result.score,
            "result"
        );
    }

    if let Some(top) = response.results.first() {
        let action = execute(top)?;
        info!(result_id = top.id, action, "top action preview");
    }

    Ok(())
}
