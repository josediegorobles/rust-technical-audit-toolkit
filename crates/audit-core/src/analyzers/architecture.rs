use std::collections::{BTreeMap, BTreeSet};

use petgraph::{algo::tarjan_scc, graph::DiGraph, graph::NodeIndex};
use syn::{Item, UseTree};

use crate::{analyzers::Analyzer, collector::RepositorySnapshot, model::ArchitectureAssessment};

pub struct ArchitectureAnalyzer;

impl Analyzer<ArchitectureAssessment> for ArchitectureAnalyzer {
    fn analyze(&self, snapshot: &RepositorySnapshot) -> ArchitectureAssessment {
        let module_profiles = snapshot
            .rust_files()
            .map(module_profile)
            .collect::<Vec<_>>();
        let module_count = module_profiles.len().max(1);
        let package_count = snapshot
            .manifests
            .iter()
            .filter(|manifest| manifest.package_name.is_some())
            .count();
        let top_level_modules = module_profiles
            .iter()
            .filter_map(|profile| profile.top_level_module.clone())
            .collect::<BTreeSet<_>>();
        let avg_module_depth = module_profiles
            .iter()
            .map(|profile| profile.module_depth)
            .sum::<usize>() as f32
            / module_count as f32;
        let fan_out = fan_out_by_module(&module_profiles, &top_level_modules);
        let fan_out_edges = fan_out.values().map(BTreeSet::len).sum::<usize>();
        let max_fan_out = fan_out.values().map(BTreeSet::len).max().unwrap_or(0);
        let avg_fan_out = if fan_out.is_empty() {
            0.0
        } else {
            fan_out_edges as f32 / fan_out.len() as f32
        };

        let module_centralization_risks = module_profiles
            .iter()
            .filter(|profile| profile.mod_declarations > 20)
            .map(|profile| {
                format!(
                    "{} centralizes {} module declarations",
                    profile.relative_path, profile.mod_declarations
                )
            })
            .collect::<Vec<_>>();
        let circular_dependencies = circular_dependencies(&fan_out);
        let detected_layers =
            structural_signals(package_count, avg_module_depth, fan_out_edges, max_fan_out);
        let domain_boundaries = structural_boundaries(snapshot, &top_level_modules);
        let architecture_style = classify_architecture(package_count, avg_module_depth);
        let separation_of_concerns = separation_of_concerns(
            package_count,
            avg_module_depth,
            max_fan_out,
            &module_centralization_risks,
            &circular_dependencies,
        );

        let mut score = 68_i32;
        score += match package_count {
            0 | 1 => 0,
            2..=4 => 12,
            _ => 14,
        };
        score += if (1.0..=4.5).contains(&avg_module_depth) {
            8
        } else if (0.5..1.0).contains(&avg_module_depth) {
            6
        } else if avg_module_depth > 6.0 {
            -8
        } else {
            -3
        };
        score += if fan_out_edges > 0 && max_fan_out <= 12 {
            6
        } else if max_fan_out > 20 {
            -12
        } else if avg_fan_out > 8.0 {
            -8
        } else {
            0
        };
        let centralization_density = module_centralization_risks.len() as f32 / module_count as f32;
        score -= ((centralization_density * 35.0).round() as i32).min(20);
        score -= (circular_dependencies.len() as i32 * 12).min(24);

        ArchitectureAssessment {
            detected_layers,
            domain_boundaries,
            module_centralization_risks,
            circular_dependencies,
            architecture_style,
            separation_of_concerns,
            score: score.clamp(0, 100) as u8,
        }
    }
}

#[derive(Debug)]
struct ModuleProfile {
    relative_path: String,
    top_level_module: Option<String>,
    module_depth: usize,
    mod_declarations: usize,
    crate_uses: BTreeSet<String>,
}

fn module_profile(file: &crate::collector::FileSnapshot) -> ModuleProfile {
    let content = file.content.as_deref().unwrap_or_default();
    let parsed = if file.lines <= 500 {
        parse_rust_architecture(content).ok()
    } else {
        None
    };
    ModuleProfile {
        relative_path: file.relative_path.clone(),
        top_level_module: top_level_module(&file.relative_path),
        module_depth: module_depth(&file.relative_path),
        mod_declarations: parsed
            .as_ref()
            .map(|metrics| metrics.mod_declarations)
            .unwrap_or_else(|| count_mod_declarations_textually(content)),
        crate_uses: parsed
            .map(|metrics| metrics.crate_uses)
            .unwrap_or_else(|| collect_crate_uses_textually(content)),
    }
}

#[derive(Default)]
struct ArchitectureSyntax {
    mod_declarations: usize,
    crate_uses: BTreeSet<String>,
}

fn parse_rust_architecture(content: &str) -> Result<ArchitectureSyntax, syn::Error> {
    let syntax = syn::parse_file(content)?;
    let mut metrics = ArchitectureSyntax::default();
    for item in syntax.items {
        match item {
            Item::Mod(_) => metrics.mod_declarations += 1,
            Item::Use(item_use) => collect_crate_uses_from_tree(&item_use.tree, &mut metrics),
            _ => {}
        }
    }
    Ok(metrics)
}

fn collect_crate_uses_from_tree(tree: &UseTree, metrics: &mut ArchitectureSyntax) {
    match tree {
        UseTree::Path(path) if path.ident == "crate" => {
            collect_first_segment_after_crate(&path.tree, metrics);
        }
        UseTree::Path(path) => collect_crate_uses_from_tree(&path.tree, metrics),
        UseTree::Group(group) => {
            for tree in &group.items {
                collect_crate_uses_from_tree(tree, metrics);
            }
        }
        UseTree::Name(_) | UseTree::Rename(_) | UseTree::Glob(_) => {}
    }
}

fn collect_first_segment_after_crate(tree: &UseTree, metrics: &mut ArchitectureSyntax) {
    match tree {
        UseTree::Path(path) => {
            metrics.crate_uses.insert(path.ident.to_string());
        }
        UseTree::Name(name) => {
            metrics.crate_uses.insert(name.ident.to_string());
        }
        UseTree::Rename(rename) => {
            metrics.crate_uses.insert(rename.ident.to_string());
        }
        UseTree::Group(group) => {
            for tree in &group.items {
                collect_first_segment_after_crate(tree, metrics);
            }
        }
        UseTree::Glob(_) => {}
    }
}

fn fan_out_by_module(
    profiles: &[ModuleProfile],
    top_level_modules: &BTreeSet<String>,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut fan_out = BTreeMap::new();
    for profile in profiles {
        let Some(source) = &profile.top_level_module else {
            continue;
        };
        for target in &profile.crate_uses {
            if target != source && top_level_modules.contains(target) {
                fan_out
                    .entry(source.clone())
                    .or_insert_with(BTreeSet::new)
                    .insert(target.clone());
            }
        }
    }
    fan_out
}

fn circular_dependencies(fan_out: &BTreeMap<String, BTreeSet<String>>) -> Vec<String> {
    let mut graph = DiGraph::<String, ()>::new();
    let mut nodes = BTreeMap::<String, NodeIndex>::new();
    for (source, targets) in fan_out {
        let source_idx = node_for(source, &mut graph, &mut nodes);
        for target in targets {
            let target_idx = node_for(target, &mut graph, &mut nodes);
            graph.add_edge(source_idx, target_idx, ());
        }
    }

    tarjan_scc(&graph)
        .into_iter()
        .filter(|component| component.len() > 1)
        .map(|component| {
            let mut names = component
                .into_iter()
                .map(|node| graph[node].clone())
                .collect::<Vec<_>>();
            names.sort();
            format!("cycle among top-level modules: {}", names.join(" -> "))
        })
        .collect()
}

fn node_for(
    name: &str,
    graph: &mut DiGraph<String, ()>,
    nodes: &mut BTreeMap<String, NodeIndex>,
) -> NodeIndex {
    if let Some(node) = nodes.get(name) {
        *node
    } else {
        let node = graph.add_node(name.to_string());
        nodes.insert(name.to_string(), node);
        node
    }
}

fn structural_signals(
    package_count: usize,
    avg_module_depth: f32,
    fan_out_edges: usize,
    max_fan_out: usize,
) -> Vec<String> {
    let mut signals = Vec::new();
    if package_count > 1 {
        signals.push("multi-crate workspace".to_string());
    } else {
        signals.push("single crate".to_string());
    }
    if avg_module_depth >= 1.0 {
        signals.push("nested module tree".to_string());
    }
    if fan_out_edges > 0 {
        signals.push("crate-relative module fan-out".to_string());
    }
    if max_fan_out > 20 {
        signals.push("high top-level fan-out".to_string());
    }
    signals
}

fn structural_boundaries(
    snapshot: &RepositorySnapshot,
    top_level_modules: &BTreeSet<String>,
) -> Vec<String> {
    let mut boundaries = snapshot
        .manifests
        .iter()
        .filter_map(|manifest| manifest.package_name.as_ref())
        .map(|name| format!("crate:{name}"))
        .collect::<BTreeSet<_>>();
    boundaries.extend(
        top_level_modules
            .iter()
            .map(|module| format!("module:{module}")),
    );
    boundaries.into_iter().collect()
}

fn separation_of_concerns(
    package_count: usize,
    avg_module_depth: f32,
    max_fan_out: usize,
    centralization_risks: &[String],
    circular_dependencies: &[String],
) -> String {
    if !circular_dependencies.is_empty() {
        "Top-level module cycles detected; dependency direction should be reviewed.".to_string()
    } else if max_fan_out > 20 || !centralization_risks.is_empty() {
        "Some structural boundaries exist, with centralization hotspots to review.".to_string()
    } else if package_count > 1 || avg_module_depth >= 1.0 {
        "Structural boundaries are visible through crates and module organization.".to_string()
    } else {
        "Compact structure with limited module separation detected.".to_string()
    }
}

fn classify_architecture(package_count: usize, avg_module_depth: f32) -> String {
    if package_count > 3 {
        "modular Cargo workspace".to_string()
    } else if package_count == 1 && avg_module_depth < 1.0 {
        "single-crate compact codebase".to_string()
    } else if package_count == 1 {
        "single-crate modular codebase".to_string()
    } else {
        "compact Cargo workspace".to_string()
    }
}

fn top_level_module(relative_path: &str) -> Option<String> {
    let components_after_src = components_after_src(relative_path)?;
    let first = components_after_src.first()?;
    if matches!(*first, "lib.rs" | "main.rs") {
        return None;
    }
    Some(first.trim_end_matches(".rs").to_string())
}

fn module_depth(relative_path: &str) -> usize {
    let Some(components) = components_after_src(relative_path) else {
        return 0;
    };
    if components.len() == 1 && matches!(components[0], "lib.rs" | "main.rs") {
        return 0;
    }
    components.len()
}

fn components_after_src(relative_path: &str) -> Option<Vec<&str>> {
    let components = relative_path.split('/').collect::<Vec<_>>();
    let src_index = components
        .iter()
        .position(|component| *component == "src")?;
    Some(components.into_iter().skip(src_index + 1).collect())
}

fn count_mod_declarations_textually(content: &str) -> usize {
    content
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("mod ") || line.starts_with("pub mod "))
        .count()
}

fn collect_crate_uses_textually(content: &str) -> BTreeSet<String> {
    content
        .lines()
        .map(str::trim)
        .filter_map(|line| line.strip_prefix("use crate::"))
        .filter_map(|line| {
            line.split(|ch: char| !ch.is_alphanumeric() && ch != '_')
                .find(|segment| !segment.is_empty())
        })
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, path::PathBuf};

    use crate::{
        analyzers::{architecture::parse_rust_architecture, Analyzer},
        collector::{CargoManifest, FileSnapshot, RepositorySnapshot},
    };

    use super::ArchitectureAnalyzer;

    #[test]
    fn empty_repository_scores_as_compact_unknown_structure() {
        let report = ArchitectureAnalyzer.analyze(&snapshot(Vec::new(), Vec::new()));

        assert!(report.score < 75);
        assert!(report.module_centralization_risks.is_empty());
        assert!(report.circular_dependencies.is_empty());
    }

    #[test]
    fn typical_workspace_gets_structural_signals_without_path_layer_names() {
        let report = ArchitectureAnalyzer.analyze(&snapshot(
            vec![
                ("crates/core/src/lib.rs", "pub mod engine;\n"),
                ("crates/core/src/engine.rs", "pub fn run() {}\n"),
                (
                    "crates/http/src/lib.rs",
                    "use crate::router;\npub mod router;\n",
                ),
                ("crates/http/src/router.rs", "pub fn route() {}\n"),
            ],
            vec!["core", "http"],
        ));

        assert!(report
            .detected_layers
            .contains(&"multi-crate workspace".into()));
        assert!(report.score >= 80);
    }

    #[test]
    fn extreme_entrypoint_centralization_is_reported_by_renamed_metric() {
        let mut content = String::new();
        for index in 0..25 {
            content.push_str(&format!("pub mod module_{index};\n"));
        }
        let report = ArchitectureAnalyzer.analyze(&snapshot(
            vec![("src/lib.rs", content.as_str())],
            vec!["app"],
        ));

        assert_eq!(report.module_centralization_risks.len(), 1);
        assert!(report.score < 80);
    }

    #[test]
    fn adversarial_commented_use_does_not_create_cycle() {
        let metrics = parse_rust_architecture("// use crate::fake;\npub mod real;\n")
            .expect("valid Rust should parse");

        assert!(metrics.crate_uses.is_empty());
        assert_eq!(metrics.mod_declarations, 1);
    }

    #[test]
    fn top_level_cycles_are_detected_with_petgraph() {
        let report = ArchitectureAnalyzer.analyze(&snapshot(
            vec![
                ("src/a.rs", "use crate::b;\npub fn a() {}\n"),
                ("src/b.rs", "use crate::a;\npub fn b() {}\n"),
            ],
            vec!["app"],
        ));

        assert_eq!(report.circular_dependencies.len(), 1);
    }

    fn snapshot(files: Vec<(&str, &str)>, packages: Vec<&str>) -> RepositorySnapshot {
        RepositorySnapshot {
            root: PathBuf::from("/tmp/repo"),
            files: files
                .into_iter()
                .map(|(relative_path, content)| FileSnapshot {
                    path: PathBuf::from("/tmp/repo").join(relative_path),
                    relative_path: relative_path.to_string(),
                    extension: Some("rs".into()),
                    bytes: content.len() as u64,
                    lines: content.lines().count(),
                    content: Some(content.to_string()),
                })
                .collect(),
            manifests: packages
                .into_iter()
                .map(|name| CargoManifest {
                    relative_path: format!("crates/{name}/Cargo.toml"),
                    package_name: Some(name.to_string()),
                    workspace_members: Vec::new(),
                    dependencies: BTreeMap::new(),
                    dev_dependencies: BTreeMap::new(),
                    build_dependencies: BTreeMap::new(),
                })
                .collect(),
        }
    }
}
