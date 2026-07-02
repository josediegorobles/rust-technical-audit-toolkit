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

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, path::PathBuf};

    use crate::{
        analyzers::Analyzer,
        collector::{CargoManifest, FileSnapshot, RepositorySnapshot},
    };

    use super::OverviewAnalyzer;

    #[test]
    fn empty_repository_reports_empty_overview() {
        let overview = OverviewAnalyzer.analyze(&snapshot(Vec::new(), Vec::new()));

        assert_eq!(overview.package_count, 0);
        assert_eq!(overview.total_files, 0);
        assert!(overview.summary.contains("Single-package"));
    }

    #[test]
    fn typical_workspace_reports_members_and_languages() {
        let overview = OverviewAnalyzer.analyze(&snapshot(
            vec![
                file("src/lib.rs", "rs", "pub fn main() {}\n"),
                file("README.md", "md", "# App\n"),
            ],
            vec![manifest(
                "Cargo.toml",
                None,
                vec!["crates/core", "crates/cli"],
            )],
        ));

        assert_eq!(overview.crate_count, 2);
        assert_eq!(overview.workspace_members.len(), 2);
        assert!(overview
            .languages
            .iter()
            .any(|language| language.language == "Rust"));
    }

    #[test]
    fn extreme_file_sizes_are_summed_without_scoring_side_effects() {
        let overview = OverviewAnalyzer.analyze(&snapshot(
            vec![FileSnapshot {
                path: PathBuf::from("/tmp/repo/big.bin"),
                relative_path: "big.bin".into(),
                extension: None,
                bytes: 10_000_000,
                lines: 0,
                content: None,
            }],
            Vec::new(),
        ));

        assert_eq!(overview.total_bytes, 10_000_000);
        assert_eq!(overview.total_files, 1);
    }

    #[test]
    fn adversarial_unknown_extension_is_bucketed_as_other() {
        let overview = OverviewAnalyzer.analyze(&snapshot(
            vec![file("src/fake.rs.txt", "txt", "fn fake() {}\n")],
            Vec::new(),
        ));

        assert!(overview
            .languages
            .iter()
            .any(|language| language.language == "Other" && language.files == 1));
    }

    fn snapshot(files: Vec<FileSnapshot>, manifests: Vec<CargoManifest>) -> RepositorySnapshot {
        RepositorySnapshot {
            root: PathBuf::from("/tmp/repo"),
            files,
            manifests,
        }
    }

    fn file(relative_path: &str, extension: &str, content: &str) -> FileSnapshot {
        FileSnapshot {
            path: PathBuf::from("/tmp/repo").join(relative_path),
            relative_path: relative_path.into(),
            extension: Some(extension.into()),
            bytes: content.len() as u64,
            lines: content.lines().count(),
            content: Some(content.into()),
        }
    }

    fn manifest(
        relative_path: &str,
        package_name: Option<&str>,
        workspace_members: Vec<&str>,
    ) -> CargoManifest {
        CargoManifest {
            relative_path: relative_path.into(),
            package_name: package_name.map(str::to_string),
            workspace_members: workspace_members.into_iter().map(str::to_string).collect(),
            dependencies: BTreeMap::new(),
            dev_dependencies: BTreeMap::new(),
            build_dependencies: BTreeMap::new(),
        }
    }
}
