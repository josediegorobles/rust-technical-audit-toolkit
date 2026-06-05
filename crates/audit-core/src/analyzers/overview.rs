use std::collections::BTreeMap;

use crate::{
    analyzers::Analyzer,
    collector::RepositorySnapshot,
    model::{LanguageStat, Overview},
};

pub struct OverviewAnalyzer;

impl Analyzer<Overview> for OverviewAnalyzer {
    fn analyze(&self, snapshot: &RepositorySnapshot) -> Overview {
        let mut language_map: BTreeMap<String, (usize, usize)> = BTreeMap::new();
        for file in &snapshot.files {
            let language = language_for(file.extension.as_deref());
            let entry = language_map.entry(language.to_string()).or_default();
            entry.0 += 1;
            entry.1 += file.lines;
        }

        let languages = language_map
            .into_iter()
            .map(|(language, (files, lines))| LanguageStat {
                language,
                files,
                lines,
            })
            .collect::<Vec<_>>();
        let workspace_members = snapshot
            .manifests
            .iter()
            .flat_map(|manifest| manifest.workspace_members.clone())
            .collect::<Vec<_>>();
        let package_count = snapshot
            .manifests
            .iter()
            .filter(|manifest| manifest.package_name.is_some())
            .count();
        let total_bytes = snapshot.files.iter().map(|file| file.bytes).sum();
        let crate_count = package_count.max(workspace_members.len());
        let cargo_configs = snapshot
            .manifests
            .iter()
            .map(|manifest| manifest.relative_path.clone())
            .collect::<Vec<_>>();
        let summary = if workspace_members.is_empty() {
            format!("Single-package Rust repository with {package_count} Cargo manifest(s).")
        } else {
            format!(
                "Cargo workspace with {} declared member(s) and {package_count} package manifest(s).",
                workspace_members.len()
            )
        };

        Overview {
            crate_count,
            package_count,
            workspace_members,
            total_files: snapshot.files.len(),
            total_bytes,
            languages,
            cargo_configs,
            summary,
        }
    }
}

fn language_for(extension: Option<&str>) -> &'static str {
    match extension {
        Some("rs") => "Rust",
        Some("toml") => "TOML",
        Some("md") => "Markdown",
        Some("yml") | Some("yaml") => "YAML",
        Some("json") => "JSON",
        Some("sh") => "Shell",
        Some("js") | Some("ts") => "JavaScript/TypeScript",
        _ => "Other",
    }
}
