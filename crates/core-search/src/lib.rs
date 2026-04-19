use shared_types::{ResultKind, SearchQuery, SearchResponse, SearchResult};
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
    let title_tokens = tokenize(&title);
    let query_tokens = tokenize(query);
    let compact_query = compact(query);
    let compact_title = compact(&title);

    let mut score = item.score;
    let mut matched = false;

    if title == query {
        score += 3.0;
        matched = true;
    }
    if title.starts_with(query) {
        score += 1.5;
        matched = true;
    }
    if title.contains(query) {
        score += 1.0;
        matched = true;
    }
    if subtitle.contains(query) {
        score += 0.45;
        matched = true;
    }

    if !query_tokens.is_empty() {
        let mut token_hits = 0;
        for token in &query_tokens {
            if title.contains(token) || subtitle.contains(token) {
                token_hits += 1;
            }
        }
        if token_hits > 0 {
            score += (token_hits as f32) * 0.55;
            matched = true;
        }
        if token_hits == query_tokens.len() {
            score += 0.6;
        }
    }

    if title_tokens.iter().any(|token| token.starts_with(query)) {
        score += 0.9;
        matched = true;
    }

    if !compact_query.is_empty() && compact_title.contains(&compact_query) {
        score += 0.5;
        matched = true;
    }

    if is_subsequence(&title, query) {
        score += 0.25;
        matched = true;
    }

    if matched {
        match item.kind {
            ResultKind::App => {
                score += 1.1;
            }
            ResultKind::File => {
                score -= 0.25;
            }
            ResultKind::Folder => {
                score -= 0.05;
            }
            ResultKind::Command | ResultKind::Calculation => {}
        }
    }

    if score <= item.score {
        0.0
    } else {
        score
    }
}

fn tokenize(value: &str) -> Vec<String> {
    value
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

fn compact(value: &str) -> String {
    value.chars().filter(|c| c.is_alphanumeric()).collect()
}

fn is_subsequence(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }

    let mut chars = needle.chars();
    let mut current = chars.next();
    for c in haystack.chars() {
        if current == Some(c) {
            current = chars.next();
            if current.is_none() {
                return true;
            }
        }
    }
    false
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

    #[test]
    fn app_match_beats_file_match_for_chrome() {
        let app = SearchResult {
            id: "app:chrome".to_string(),
            title: "Google Chrome".to_string(),
            subtitle: Some("google-chrome-stable".to_string()),
            kind: ResultKind::App,
            score: 0.9,
        };

        let file = SearchResult {
            id: "file:chrome_notes.txt".to_string(),
            title: "chrome_notes.txt".to_string(),
            subtitle: Some("/home/u/notes/chrome_notes.txt".to_string()),
            kind: ResultKind::File,
            score: 0.7,
        };

        assert!(rank_item(&app, "chrome") > rank_item(&file, "chrome"));
    }
}
