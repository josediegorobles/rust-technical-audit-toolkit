use crate::{
    analyzers::Analyzer,
    collector::{FileSnapshot, RepositorySnapshot},
    model::CodeQuality,
};
use syn::{
    spanned::Spanned,
    visit::{self, Visit},
    BinOp, ExprBinary, ExprForLoop, ExprIf, ExprMatch, ExprWhile, ImplItemFn, ItemFn, TraitItemFn,
};

pub struct CodeQualityAnalyzer;

impl Analyzer<CodeQuality> for CodeQualityAnalyzer {
    fn analyze(&self, snapshot: &RepositorySnapshot) -> CodeQuality {
        let rust_files = snapshot.rust_files().collect::<Vec<_>>();
        let module_metrics = rust_files
            .iter()
            .map(|file| module_metrics(file))
            .collect::<Vec<_>>();
        let lines_of_code = module_metrics
            .iter()
            .map(|metrics| metrics.code_lines)
            .sum::<usize>();
        let module_count = rust_files.len();
        let function_count = module_metrics
            .iter()
            .map(|metrics| metrics.function_count)
            .sum::<usize>();
        let function_lines = module_metrics
            .iter()
            .flat_map(|metrics| metrics.function_lines.iter().copied())
            .collect::<Vec<_>>();
        let average_function_size = if function_lines.is_empty() {
            0.0
        } else {
            function_lines.iter().sum::<usize>() as f32 / function_lines.len() as f32
        };

        let module_lines = module_metrics
            .iter()
            .map(|metrics| metrics.lines)
            .collect::<Vec<_>>();
        let p90_module_lines = percentile(&module_lines, 90);
        let large_module_floor = 300;
        let large_module_threshold = p90_module_lines.max(large_module_floor);
        let god_module_line_threshold = ((p90_module_lines as f32 * 1.5).ceil() as usize).max(500);
        let module_function_counts = module_metrics
            .iter()
            .map(|metrics| metrics.function_count)
            .collect::<Vec<_>>();
        let god_module_function_threshold = percentile(&module_function_counts, 90).max(30);

        let large_modules = rust_files
            .iter()
            .zip(module_metrics.iter())
            .filter(|(_, metrics)| metrics.lines >= large_module_threshold)
            .map(|(file, _)| format!("{} ({} lines)", file.relative_path, file.lines))
            .collect::<Vec<_>>();
        let god_module_candidates = rust_files
            .iter()
            .zip(module_metrics.iter())
            .filter(|(_, metrics)| {
                metrics.lines >= god_module_line_threshold
                    || metrics.function_count >= god_module_function_threshold
            })
            .map(|(file, _)| file.relative_path.clone())
            .collect::<Vec<_>>();

        let mut complexity_indicators = Vec::new();
        for (file, metrics) in rust_files.iter().zip(module_metrics.iter()) {
            let branch_density = if metrics.code_lines == 0 {
                0.0
            } else {
                metrics.branch_count as f32 / metrics.code_lines as f32
            };
            if metrics.branch_count > 80 && branch_density > 0.08 {
                complexity_indicators.push(format!(
                    "{} has high branching density ({} indicators)",
                    file.relative_path, metrics.branch_count
                ));
            }
        }

        let mut score = 100_i32;
        let module_denominator = module_count.max(1) as f32;
        let large_module_density = large_modules.len() as f32 / module_denominator;
        let god_module_density = god_module_candidates.len() as f32 / module_denominator;
        let complexity_density = complexity_indicators.len() as f32 / module_denominator;
        score -= ((large_module_density * 30.0).round() as i32).min(30);
        score -= ((god_module_density * 45.0).round() as i32).min(45);
        score -= ((complexity_density * 25.0).round() as i32).min(25);
        score -= if average_function_size > 60.0 { 12 } else { 0 };

        CodeQuality {
            lines_of_code,
            module_count,
            function_count,
            average_function_size,
            complexity_indicators,
            large_modules,
            god_module_candidates,
            score: score.clamp(0, 100) as u8,
        }
    }
}

#[derive(Debug)]
struct ModuleMetrics {
    lines: usize,
    code_lines: usize,
    function_count: usize,
    function_lines: Vec<usize>,
    branch_count: usize,
}

fn count_code_lines(content: &str) -> usize {
    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("//"))
        .count()
}

fn module_metrics(file: &FileSnapshot) -> ModuleMetrics {
    let content = file.content.as_deref().unwrap_or_default();
    let parsed = if file.lines <= 500 {
        parse_syntax_metrics(content).ok()
    } else {
        None
    };
    let function_lines = parsed
        .as_ref()
        .map(|metrics| metrics.function_lines.clone())
        .filter(|lines| !lines.is_empty())
        .unwrap_or_else(|| estimate_function_sizes(content));
    let function_count = parsed
        .as_ref()
        .map(|metrics| metrics.function_count)
        .unwrap_or_else(|| count_function_declarations_textually(content));
    let branch_count = parsed
        .as_ref()
        .map(|metrics| metrics.branch_count)
        .unwrap_or_else(|| count_branch_indicators_textually(content));

    ModuleMetrics {
        lines: file.lines,
        code_lines: count_code_lines(content),
        function_count,
        function_lines,
        branch_count,
    }
}

#[derive(Default)]
struct SyntaxMetrics {
    function_count: usize,
    function_lines: Vec<usize>,
    branch_count: usize,
}

impl<'ast> Visit<'ast> for SyntaxMetrics {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        self.push_function(node.span());
        visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast ImplItemFn) {
        self.push_function(node.span());
        visit::visit_impl_item_fn(self, node);
    }

    fn visit_trait_item_fn(&mut self, node: &'ast TraitItemFn) {
        self.push_function(node.span());
        visit::visit_trait_item_fn(self, node);
    }

    fn visit_expr_if(&mut self, node: &'ast ExprIf) {
        self.branch_count += 1;
        visit::visit_expr_if(self, node);
    }

    fn visit_expr_match(&mut self, node: &'ast ExprMatch) {
        self.branch_count += 1;
        visit::visit_expr_match(self, node);
    }

    fn visit_expr_while(&mut self, node: &'ast ExprWhile) {
        self.branch_count += 1;
        visit::visit_expr_while(self, node);
    }

    fn visit_expr_for_loop(&mut self, node: &'ast ExprForLoop) {
        self.branch_count += 1;
        visit::visit_expr_for_loop(self, node);
    }

    fn visit_expr_binary(&mut self, node: &'ast ExprBinary) {
        if matches!(node.op, BinOp::And(_) | BinOp::Or(_)) {
            self.branch_count += 1;
        }
        visit::visit_expr_binary(self, node);
    }
}

impl SyntaxMetrics {
    fn push_function(&mut self, span: proc_macro2::Span) {
        let start = span.start().line;
        let end = span.end().line;
        self.function_count += 1;
        self.function_lines
            .push(end.saturating_sub(start).max(0) + 1);
    }
}

fn parse_syntax_metrics(content: &str) -> Result<SyntaxMetrics, syn::Error> {
    let syntax = syn::parse_file(content)?;
    let mut metrics = SyntaxMetrics::default();
    metrics.visit_file(&syntax);
    Ok(metrics)
}

fn count_function_declarations_textually(content: &str) -> usize {
    estimate_function_sizes(content).len()
}

fn count_branch_indicators_textually(content: &str) -> usize {
    [" if ", " match ", " while ", " for ", "&&", "||"]
        .iter()
        .map(|needle| content.matches(needle).count())
        .sum::<usize>()
}

fn percentile(values: &[usize], percentile: usize) -> usize {
    if values.is_empty() {
        return 0;
    }
    let mut values = values.to_vec();
    values.sort_unstable();
    let rank = ((values.len() as f32 - 1.0) * percentile as f32 / 100.0).ceil() as usize;
    values[rank.min(values.len() - 1)]
}

fn estimate_function_sizes(content: &str) -> Vec<usize> {
    let lines = content.lines().collect::<Vec<_>>();
    let mut sizes = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        if line.trim_start().starts_with("fn ")
            || line.trim_start().starts_with("pub fn ")
            || line.trim_start().starts_with("async fn ")
            || line.trim_start().starts_with("pub async fn ")
        {
            let mut depth = 0_i32;
            let mut seen_body = false;
            for (end_idx, candidate) in lines.iter().enumerate().skip(idx) {
                for ch in candidate.chars() {
                    if ch == '{' {
                        depth += 1;
                        seen_body = true;
                    } else if ch == '}' {
                        depth -= 1;
                    }
                }
                if seen_body && depth <= 0 {
                    sizes.push(end_idx - idx + 1);
                    break;
                }
            }
        }
    }
    sizes
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        analyzers::{code_quality::parse_syntax_metrics, Analyzer},
        collector::{CargoManifest, FileSnapshot, RepositorySnapshot},
    };

    use super::CodeQualityAnalyzer;

    #[test]
    fn empty_repository_scores_without_panicking() {
        let report = CodeQualityAnalyzer.analyze(&snapshot(Vec::new()));

        assert_eq!(report.module_count, 0);
        assert_eq!(report.function_count, 0);
        assert_eq!(report.score, 100);
    }

    #[test]
    fn typical_repository_uses_parser_backed_function_counts() {
        let report = CodeQualityAnalyzer.analyze(&snapshot(vec![(
            "src/lib.rs",
            r#"
pub fn one() {}

impl Thing {
    pub async fn two(&self) {
        if true {}
    }
}
"#,
        )]));

        assert_eq!(report.function_count, 2);
        assert!(report.average_function_size >= 1.0);
        assert!(report.score >= 90);
    }

    #[test]
    fn extreme_repository_penalizes_density_not_absolute_count() {
        let mut files = Vec::new();
        for index in 0..10 {
            let content = if index < 2 {
                repeated_functions(80)
            } else {
                "pub fn small() {}\n".to_string()
            };
            files.push((format!("src/module_{index}.rs"), content));
        }
        let files = files
            .iter()
            .map(|(path, content)| (path.as_str(), content.as_str()))
            .collect::<Vec<_>>();

        let report = CodeQualityAnalyzer.analyze(&snapshot(files));

        assert_eq!(report.god_module_candidates.len(), 2);
        assert!(report.score > 40);
        assert!(report.score < 100);
    }

    #[test]
    fn adversarial_comments_do_not_count_as_functions_when_syn_parses() {
        let metrics = parse_syntax_metrics(
            r#"
// fn fake() {}
pub fn real() {}
"#,
        )
        .expect("valid Rust should parse");

        assert_eq!(metrics.function_count, 1);
    }

    fn snapshot(files: Vec<(&str, &str)>) -> RepositorySnapshot {
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
            manifests: Vec::<CargoManifest>::new(),
        }
    }

    fn repeated_functions(count: usize) -> String {
        let mut content = String::new();
        for index in 0..count {
            content.push_str(&format!("pub fn function_{index}() {{}}\n"));
        }
        content
    }
}
