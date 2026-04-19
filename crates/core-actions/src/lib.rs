use anyhow::{anyhow, Result};
use shared_types::{ResultKind, SearchResult};
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn execute(result: &SearchResult) -> Result<String> {
    match result.kind {
        ResultKind::App => launch_app(result),
        ResultKind::File => open_path_from_id(&result.id, "open-file"),
        ResultKind::Folder => open_path_from_id(&result.id, "open-folder"),
        ResultKind::Command => Ok(format!("run-command:{}", result.id)),
        ResultKind::Calculation => Err(anyhow!("calculation results are not executable")),
    }
}

pub fn reveal(result: &SearchResult) -> Result<String> {
    match result.kind {
        ResultKind::File => reveal_parent_from_id(&result.id),
        ResultKind::Folder => open_path_from_id(&result.id, "reveal-folder"),
        ResultKind::App | ResultKind::Command | ResultKind::Calculation => {
            Err(anyhow!("reveal is not supported for this result kind"))
        }
    }
}

pub fn extract_path(result: &SearchResult) -> Option<String> {
    match result.kind {
        ResultKind::File | ResultKind::Folder => {
            result.id.split_once(':').map(|(_, path)| path.to_string())
        }
        _ => None,
    }
}

fn launch_app(result: &SearchResult) -> Result<String> {
    if try_focus_running_app(result).unwrap_or(false) {
        return Ok(format!("focus-running-app:{}", result.id));
    }

    if let Some(exec) = app_desktop_path(&result.id)
        .as_deref()
        .and_then(read_exec_from_desktop_file)
    {
        let status = Command::new("sh").arg("-lc").arg(exec).spawn()?;
        return Ok(format!("launch-app:{} (pid {:?})", result.id, status.id()));
    }

    if let Some(exec) = result.subtitle.as_ref().filter(|s| !s.trim().is_empty()) {
        let status = Command::new("sh").arg("-lc").arg(exec).spawn()?;
        return Ok(format!("launch-app:{} (pid {:?})", result.id, status.id()));
    }
    Ok(format!("launch-app:{}", result.id))
}

fn open_path_from_id(id: &str, label: &str) -> Result<String> {
    let path = id
        .split_once(':')
        .map(|(_, value)| value)
        .ok_or_else(|| anyhow!("invalid result id: {id}"))?;

    let status = Command::new("xdg-open").arg(path).spawn()?;
    Ok(format!("{label}:{path} (pid {:?})", status.id()))
}

fn reveal_parent_from_id(id: &str) -> Result<String> {
    let path = id
        .split_once(':')
        .map(|(_, value)| value)
        .ok_or_else(|| anyhow!("invalid result id: {id}"))?;

    let parent = std::path::Path::new(path)
        .parent()
        .ok_or_else(|| anyhow!("unable to resolve parent for path: {path}"))?;

    let status = Command::new("xdg-open").arg(parent).spawn()?;
    Ok(format!("reveal-file:{path} (pid {:?})", status.id()))
}

fn try_focus_running_app(result: &SearchResult) -> Result<bool> {
    if !command_exists("wmctrl") {
        return Ok(false);
    }

    let mut candidates = Vec::new();
    candidates.push(result.title.to_lowercase());

    if let Some(exec) = result.subtitle.as_ref() {
        if let Some(token) = first_exec_token(exec) {
            candidates.push(token.to_lowercase());
        }
    }

    if let Some(desktop_path) = app_desktop_path(&result.id) {
        candidates.extend(parse_desktop_candidates(&desktop_path));
    }

    candidates = unique_candidates(candidates);
    if candidates.is_empty() {
        return Ok(false);
    }

    let windows = Command::new("wmctrl").arg("-lx").output()?;
    if !windows.status.success() {
        return Ok(false);
    }

    let listing = String::from_utf8_lossy(&windows.stdout);
    for line in listing.lines() {
        let mut parts = line.split_whitespace();
        let Some(window_id) = parts.next() else {
            continue;
        };
        let _desktop_num = parts.next();
        let _host = parts.next();
        let class = parts.next().unwrap_or_default().to_lowercase();
        let title = parts.collect::<Vec<_>>().join(" ").to_lowercase();

        let matched = candidates
            .iter()
            .any(|candidate| class.contains(candidate) || title.contains(candidate));
        if matched {
            let status = Command::new("wmctrl").arg("-ia").arg(window_id).status()?;
            if status.success() {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

fn command_exists(cmd: &str) -> bool {
    Command::new("sh")
        .arg("-lc")
        .arg(format!("command -v {cmd} >/dev/null 2>&1"))
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn app_desktop_path(id: &str) -> Option<String> {
    id.split_once(':').and_then(|(prefix, path)| {
        if prefix == "app" {
            Some(path.to_string())
        } else {
            None
        }
    })
}

fn parse_desktop_candidates(path: &str) -> Vec<String> {
    let mut out = Vec::new();
    let Ok(raw) = fs::read_to_string(path) else {
        return out;
    };

    for line in raw.lines() {
        if line.starts_with("StartupWMClass=") {
            let value = line.trim_start_matches("StartupWMClass=").trim();
            if !value.is_empty() {
                out.push(value.to_lowercase());
            }
        } else if line.starts_with("Name=") {
            let value = line.trim_start_matches("Name=").trim();
            if !value.is_empty() {
                out.push(value.to_lowercase());
            }
        } else if line.starts_with("Exec=") {
            let value = line.trim_start_matches("Exec=").trim();
            if let Some(token) = first_exec_token(value) {
                out.push(token.to_lowercase());
            }
        }
    }

    if let Some(file_name) = Path::new(path).file_name().and_then(OsStr::to_str) {
        let desktop_id = file_name.trim_end_matches(".desktop").to_lowercase();
        if !desktop_id.is_empty() {
            out.push(desktop_id);
        }
    }

    out
}

fn read_exec_from_desktop_file(path: &str) -> Option<String> {
    let raw = fs::read_to_string(path).ok()?;
    let exec_line = raw
        .lines()
        .find(|line| line.starts_with("Exec="))
        .map(|line| line.trim_start_matches("Exec=").trim())?;
    Some(
        exec_line
            .split_whitespace()
            .filter(|part| !part.starts_with('%'))
            .collect::<Vec<_>>()
            .join(" "),
    )
}

fn first_exec_token(exec: &str) -> Option<String> {
    let raw = exec
        .split_whitespace()
        .find(|part| !part.starts_with('%') && !part.is_empty())?;
    let base = Path::new(raw).file_name().and_then(OsStr::to_str)?;
    Some(base.trim_end_matches("-stable").to_string())
}

fn unique_candidates(candidates: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    for candidate in candidates {
        let normalized = candidate
            .trim()
            .trim_matches('"')
            .replace('_', "-")
            .to_lowercase();
        if normalized.len() < 2 {
            continue;
        }
        if !out.contains(&normalized) {
            out.push(normalized);
        }
    }
    out
}
