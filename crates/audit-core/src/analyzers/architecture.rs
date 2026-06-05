use std::collections::BTreeSet;

use crate::{analyzers::Analyzer, collector::RepositorySnapshot, model::ArchitectureAssessment};

pub struct ArchitectureAnalyzer;

impl Analyzer<ArchitectureAssessment> for ArchitectureAnalyzer {
    fn analyze(&self, snapshot: &RepositorySnapshot) -> ArchitectureAssessment {
        let mut detected_layers = BTreeSet::new();
        let mut domain_boundaries = BTreeSet::new();
        let mut circular_dependency_risks = Vec::new();

        for file in snapshot.rust_files() {
            let path = file.relative_path.to_ascii_lowercase();
            for layer in [
                "api",
                "application",
                "domain",
                "infrastructure",
                "adapter",
                "repository",
                "service",
                "transport",
                "persistence",
            ] {
                if path.contains(layer) {
                    detected_layers.insert(layer.to_string());
                }
            }
            if let Some(boundary) = path.split('/').nth(1) {
                if (path.starts_with("src/") || path.starts_with("crates/"))
                    && !boundary.ends_with(".rs")
                {
                    domain_boundaries.insert(boundary.to_string());
                }
            }
            if path.ends_with("lib.rs") || path.ends_with("main.rs") {
                let content = file.content.as_deref().unwrap_or_default();
                let mod_count =
                    content.matches("mod ").count() + content.matches("pub mod ").count();
                if mod_count > 20 {
                    circular_dependency_risks.push(format!(
                        "{} centralizes {mod_count} module declarations",
                        file.relative_path
                    ));
                }
            }
        }

        let detected_layers = detected_layers.into_iter().collect::<Vec<_>>();
        let domain_boundaries = domain_boundaries.into_iter().collect::<Vec<_>>();
        let architecture_style = classify_architecture(snapshot, &detected_layers);
        let separation_of_concerns = match detected_layers.len() {
            0..=1 => "Limited explicit layer separation detected.".to_string(),
            2..=3 => "Some separation of concerns is visible, but boundaries should be reviewed."
                .to_string(),
            _ => "Clear layer vocabulary detected across the repository.".to_string(),
        };

        let mut score = 70_i32 + (detected_layers.len() as i32 * 5).min(20);
        score -= (circular_dependency_risks.len() as i32 * 10).min(30);
        if architecture_style.contains("monolith") {
            score -= 8;
        }

        ArchitectureAssessment {
            detected_layers,
            domain_boundaries,
            circular_dependency_risks,
            architecture_style,
            separation_of_concerns,
            score: score.clamp(0, 100) as u8,
        }
    }
}

fn classify_architecture(snapshot: &RepositorySnapshot, layers: &[String]) -> String {
    let package_count = snapshot
        .manifests
        .iter()
        .filter(|manifest| manifest.package_name.is_some())
        .count();
    if package_count > 3 {
        "modular Cargo workspace".to_string()
    } else if package_count == 1 && layers.len() <= 1 {
        "single-crate monolith".to_string()
    } else {
        "layered single service or compact workspace".to_string()
    }
}
