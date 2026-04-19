use anyhow::{anyhow, Result};
use shared_types::{ResultKind, SearchResult};
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

fn launch_app(result: &SearchResult) -> Result<String> {
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
