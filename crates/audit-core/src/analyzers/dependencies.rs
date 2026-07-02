use std::collections::BTreeSet;

use crate::{analyzers::Analyzer, collector::RepositorySnapshot, model::DependencyHealth};

pub struct DependencyAnalyzer;

impl Analyzer<DependencyHealth> for DependencyAnalyzer {
    fn analyze(&self, snapshot: &RepositorySnapshot) -> DependencyHealth {
        let mut direct = BTreeSet::new();
        let mut critical_dependencies = Vec::new();
        let mut outdated_indicators = Vec::new();
        let mut maintenance_risks = Vec::new();

        for manifest in &snapshot.manifests {
            for (name, version) in manifest
                .dependencies
                .iter()
                .chain(manifest.dev_dependencies.iter())
                .chain(manifest.build_dependencies.iter())
            {
                direct.insert(name.clone());
                if is_critical(name) {
                    critical_dependencies.push(format!("{name} in {}", manifest.relative_path));
                }
                if version.contains('*') || version.contains("path") || version.contains("git") {
                    maintenance_risks.push(format!(
                        "{name} uses a non-registry or broad version declaration in {}",
                        manifest.relative_path
                    ));
                }
                if version.starts_with("0.") || version.contains("\"0.") {
                    outdated_indicators.push(format!(
                        "{name} is pinned to pre-1.0 API surface in {}",
                        manifest.relative_path
                    ));
                }
            }
        }

        let total_dependencies = direct.len();
        if total_dependencies > 80 {
            maintenance_risks
                .push("Very high direct dependency count for due diligence review.".into());
        }

        let mut score = 100_i32;
        score -= (maintenance_risks.len() as i32 * 8).min(32);
        score -= (outdated_indicators.len() as i32 * 4).min(24);
        score -= if total_dependencies > 60 { 15 } else { 0 };
        score -= if critical_dependencies.len() > 12 {
            8
        } else {
            0
        };

        DependencyHealth {
            total_dependencies,
            direct_dependencies: total_dependencies,
            critical_dependencies,
            outdated_indicators,
            maintenance_risks,
            score: score.clamp(0, 100) as u8,
        }
    }
}

fn is_critical(name: &str) -> bool {
    matches!(
        name,
        "tokio"
            | "serde"
            | "sqlx"
            | "diesel"
            | "axum"
            | "actix-web"
            | "hyper"
            | "reqwest"
            | "openssl"
            | "ring"
            | "rustls"
            | "tonic"
            | "prost"
    )
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, path::PathBuf};

    use crate::{
        analyzers::Analyzer,
        collector::{CargoManifest, RepositorySnapshot},
    };

    use super::DependencyAnalyzer;

    #[test]
    fn empty_repository_has_no_dependency_risk() {
        let report = DependencyAnalyzer.analyze(&snapshot(Vec::new()));

        assert_eq!(report.direct_dependencies, 0);
        assert_eq!(report.score, 100);
    }

    #[test]
    fn typical_manifest_detects_direct_and_critical_dependencies() {
        let report = DependencyAnalyzer.analyze(&snapshot(vec![manifest(
            "Cargo.toml",
            "app",
            [("serde", "1"), ("tokio", "1")],
            [],
            [],
        )]));

        assert_eq!(report.direct_dependencies, 2);
        assert_eq!(report.critical_dependencies.len(), 2);
        assert_eq!(report.score, 100);
    }

    #[test]
    fn extreme_dependency_count_is_penalized() {
        let dependencies = (0..81)
            .map(|index| (format!("dep_{index}"), "1".to_string()))
            .collect::<BTreeMap<_, _>>();
        let report = DependencyAnalyzer.analyze(&snapshot(vec![CargoManifest {
            relative_path: "Cargo.toml".into(),
            package_name: Some("app".into()),
            workspace_members: Vec::new(),
            dependencies,
            dev_dependencies: BTreeMap::new(),
            build_dependencies: BTreeMap::new(),
        }]));

        assert_eq!(report.direct_dependencies, 81);
        assert!(report.score < 90);
        assert!(report
            .maintenance_risks
            .iter()
            .any(|risk| risk.contains("Very high direct dependency count")));
    }

    #[test]
    fn adversarial_dependency_names_do_not_match_critical_substrings() {
        let report = DependencyAnalyzer.analyze(&snapshot(vec![manifest(
            "Cargo.toml",
            "app",
            [("serde_fake", "1"), ("tokioish", "1")],
            [],
            [],
        )]));

        assert!(report.critical_dependencies.is_empty());
    }

    fn snapshot(manifests: Vec<CargoManifest>) -> RepositorySnapshot {
        RepositorySnapshot {
            root: PathBuf::from("/tmp/repo"),
            files: Vec::new(),
            manifests,
        }
    }

    fn manifest<const D: usize, const DEV: usize, const B: usize>(
        relative_path: &str,
        package_name: &str,
        dependencies: [(&str, &str); D],
        dev_dependencies: [(&str, &str); DEV],
        build_dependencies: [(&str, &str); B],
    ) -> CargoManifest {
        CargoManifest {
            relative_path: relative_path.into(),
            package_name: Some(package_name.into()),
            workspace_members: Vec::new(),
            dependencies: dependency_map(dependencies),
            dev_dependencies: dependency_map(dev_dependencies),
            build_dependencies: dependency_map(build_dependencies),
        }
    }

    fn dependency_map<const N: usize>(dependencies: [(&str, &str); N]) -> BTreeMap<String, String> {
        dependencies
            .into_iter()
            .map(|(name, version)| (name.to_string(), version.to_string()))
            .collect()
    }
}
