use shared_types::{SearchQuery, SearchResponse, SearchResult};
use std::time::Instant;

#[derive(Default)]
pub struct SearchEngine {
    seed: Vec<SearchResult>,
}

impl SearchEngine {
    pub fn with_seed(seed: Vec<SearchResult>) -> Self {
        Self { seed }
    }

    pub fn set_seed(&mut self, seed: Vec<SearchResult>) {
        self.seed = seed;
    }

    pub fn query(&self, query: SearchQuery) -> SearchResponse {
        let start = Instant::now();
        let q = query.text.trim().to_lowercase();

        let mut scored = self
            .seed
            .iter()
            .map(|item| {
                let score = rank_item(item, &q);
                (score, item)
            })
            .filter(|(score, _)| *score > 0.0)
            .collect::<Vec<_>>();

        scored.sort_by(|(a, _), (b, _)| b.total_cmp(a));

        let results = scored
            .into_iter()
            .take(query.limit)
            .map(|(score, item)| {
                let mut out = item.clone();
                out.score = score;
                out
            })
            .collect();

        SearchResponse {
            query: query.text,
            took_ms: start.elapsed().as_millis(),
            results,
        }
    }
}

fn rank_item(item: &SearchResult, query: &str) -> f32 {
    if query.is_empty() {
        return item.score;
    }

    let title = item.title.to_lowercase();
    let subtitle = item.subtitle.clone().unwrap_or_default().to_lowercase();

    let mut score = item.score;

    if title == query {
        score += 2.0;
    }
    if title.starts_with(query) {
        score += 1.2;
    }
    if title.contains(query) {
        score += 0.7;
    }
    if subtitle.contains(query) {
        score += 0.3;
    }

    let query_tokens = query.split_whitespace().collect::<Vec<_>>();
    if !query_tokens.is_empty() {
        let mut token_hits = 0;
        for token in &query_tokens {
            if title.contains(token) || subtitle.contains(token) {
                token_hits += 1;
            }
        }
        if token_hits > 0 {
            score += (token_hits as f32) * 0.4;
        }
    }

    if score <= item.score {
        0.0
    } else {
        score
    }
}

#[cfg(test)]
mod tests {
    use super::{rank_item, SearchEngine};
    use shared_types::{ResultKind, SearchQuery, SearchResult};

    fn sample(title: &str, subtitle: Option<&str>, base: f32) -> SearchResult {
        SearchResult {
            id: title.to_string(),
            title: title.to_string(),
            subtitle: subtitle.map(|s| s.to_string()),
            kind: ResultKind::App,
            score: base,
        }
    }

    #[test]
    fn prefix_scores_higher_than_contains() {
        let prefix = sample("Terminal", None, 0.5);
        let contains = sample("My Terminal Profile", None, 0.5);

        assert!(rank_item(&prefix, "term") > rank_item(&contains, "term"));
    }

    #[test]
    fn search_sorts_descending() {
        let engine = SearchEngine::with_seed(vec![
            sample("Files", Some("File Manager"), 0.2),
            sample("Firefox", Some("Web Browser"), 0.2),
        ]);

        let r = engine.query(SearchQuery {
            text: "fi".to_string(),
            limit: 5,
        });

        assert_eq!(r.results.first().expect("result").title, "Files");
    }
}
