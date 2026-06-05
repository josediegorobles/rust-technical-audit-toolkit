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
