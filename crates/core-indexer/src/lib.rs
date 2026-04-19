use anyhow::Result;
use shared_types::{ResultKind, SearchResult};
use std::collections::HashSet;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct IndexStats {
    pub total_items: usize,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct IndexOptions {
    pub app_paths: Vec<PathBuf>,
    pub file_paths: Vec<PathBuf>,
    pub max_files: usize,
    pub include_hidden: bool,
}

impl Default for IndexOptions {
    fn default() -> Self {
        let home = env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
        Self {
            app_paths: collect_app_paths(&home),
            file_paths: vec![PathBuf::from(home)],
            max_files: 5000,
            include_hidden: false,
        }
    }
}

fn collect_app_paths(home: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut seen = HashSet::new();

    let mut push = |path: PathBuf| {
        let key = path.to_string_lossy().to_string();
        if seen.insert(key) {
            paths.push(path);
        }
    };

    push(PathBuf::from(format!("{home}/.local/share/applications")));
    push(PathBuf::from(format!(
        "{home}/.local/share/flatpak/exports/share/applications"
    )));
    push(PathBuf::from("/usr/local/share/applications"));
    push(PathBuf::from("/usr/share/applications"));
    push(PathBuf::from("/var/lib/flatpak/exports/share/applications"));
    push(PathBuf::from("/var/lib/snapd/desktop/applications"));

    if let Ok(xdg_data_dirs) = env::var("XDG_DATA_DIRS") {
        for base in xdg_data_dirs
            .split(':')
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
        {
            push(PathBuf::from(base).join("applications"));
        }
    }

    paths
}

pub fn build_index(options: &IndexOptions) -> Result<(Vec<SearchResult>, IndexStats)> {
    let mut seen = HashSet::new();
    let mut items = Vec::new();

    items.extend(index_desktop_apps(&options.app_paths, &mut seen)?);
    items.extend(index_filesystem(
        &options.file_paths,
        options.max_files,
        options.include_hidden,
        &mut seen,
    ));

    let stats = IndexStats {
        total_items: items.len(),
        source: "linux-desktop+filesystem".to_string(),
    };

    Ok((items, stats))
}

fn index_desktop_apps(paths: &[PathBuf], seen: &mut HashSet<String>) -> Result<Vec<SearchResult>> {
    let mut apps = Vec::new();

    for base in paths {
        if !base.exists() {
            continue;
        }

        for entry in WalkDir::new(base).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if !entry.file_type().is_file() || path.extension() != Some(OsStr::new("desktop")) {
                continue;
            }

            if let Ok(raw) = fs::read_to_string(path) {
                if let Some((name, exec)) = parse_desktop_entry(&raw) {
                    let id = format!("app:{}", path.display());
                    if !seen.insert(id.clone()) {
                        continue;
                    }

                    apps.push(SearchResult {
                        id,
                        title: name,
                        subtitle: Some(exec.unwrap_or_else(|| path.display().to_string())),
                        kind: ResultKind::App,
                        score: 0.9,
                    });
                }
            }
        }
    }

    Ok(apps)
}

fn parse_desktop_entry(contents: &str) -> Option<(String, Option<String>)> {
    let mut name = None;
    let mut exec = None;
    let mut no_display = false;

    for line in contents.lines() {
        if line.starts_with("Name=") && name.is_none() {
            name = Some(line.trim_start_matches("Name=").trim().to_string());
        }
        if line.starts_with("Exec=") && exec.is_none() {
            let cmd = line.trim_start_matches("Exec=").trim();
            exec = Some(clean_exec(cmd));
        }
        if line.trim() == "NoDisplay=true" {
            no_display = true;
        }
    }

    if no_display {
        return None;
    }

    name.map(|n| (n, exec))
}

fn clean_exec(exec: &str) -> String {
    exec.split_whitespace()
        .filter(|part| !part.starts_with('%'))
        .collect::<Vec<_>>()
        .join(" ")
}

fn index_filesystem(
    roots: &[PathBuf],
    max_files: usize,
    include_hidden: bool,
    seen: &mut HashSet<String>,
) -> Vec<SearchResult> {
    let mut files = Vec::new();

    for root in roots {
        if !root.exists() {
            continue;
        }

        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            if files.len() >= max_files {
                return files;
            }

            let path = entry.path();
            if entry.file_type().is_dir() {
                continue;
            }

            if !include_hidden && is_hidden(path) {
                continue;
            }

            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };

            let id = format!("file:{}", path.display());
            if !seen.insert(id.clone()) {
                continue;
            }

            files.push(SearchResult {
                id,
                title: name.to_string(),
                subtitle: Some(path.display().to_string()),
                kind: ResultKind::File,
                score: 0.7,
            });
        }
    }

    files
}

fn is_hidden(path: &Path) -> bool {
    path.components()
        .any(|c| c.as_os_str().to_string_lossy().starts_with('.'))
}

#[cfg(test)]
mod tests {
    use super::{collect_app_paths, parse_desktop_entry};

    #[test]
    fn parses_desktop_entry() {
        let desktop = "[Desktop Entry]\nName=Terminal\nExec=gnome-terminal --wait %U\n";
        let parsed = parse_desktop_entry(desktop).expect("desktop should parse");
        assert_eq!(parsed.0, "Terminal");
        assert_eq!(parsed.1.as_deref(), Some("gnome-terminal --wait"));
    }

    #[test]
    fn collects_standard_app_paths() {
        let paths = collect_app_paths("/home/tester");
        let as_strings = paths
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>();

        assert!(as_strings
            .iter()
            .any(|p| p == "/home/tester/.local/share/applications"));
        assert!(as_strings.iter().any(|p| p == "/usr/share/applications"));
        assert!(as_strings
            .iter()
            .any(|p| p == "/var/lib/flatpak/exports/share/applications"));
    }
}
