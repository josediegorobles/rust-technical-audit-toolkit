use crate::{
    analyzers::Analyzer,
    collector::{FileSnapshot, RepositorySnapshot},
    model::CodeQuality,
};

pub struct CodeQualityAnalyzer;

impl Analyzer<CodeQuality> for CodeQualityAnalyzer {
    fn analyze(&self, snapshot: &RepositorySnapshot) -> CodeQuality {
        let rust_files = snapshot.rust_files().collect::<Vec<_>>();
        let lines_of_code = rust_files
            .iter()
            .map(|file| count_code_lines(file.content.as_deref().unwrap_or_default()))
            .sum::<usize>();
        let module_count = rust_files.len();
        let function_count = rust_files
            .iter()
            .map(|file| count_matches(file, "fn "))
            .sum::<usize>();
        let function_lines = rust_files
            .iter()
            .flat_map(|file| estimate_function_sizes(file.content.as_deref().unwrap_or_default()))
            .collect::<Vec<_>>();
        let average_function_size = if function_lines.is_empty() {
            0.0
        } else {
            function_lines.iter().sum::<usize>() as f32 / function_lines.len() as f32
        };

        let large_modules = rust_files
            .iter()
            .filter(|file| file.lines > 300)
            .map(|file| format!("{} ({} lines)", file.relative_path, file.lines))
            .collect::<Vec<_>>();
        let god_module_candidates = rust_files
            .iter()
            .filter(|file| {
                let content = file.content.as_deref().unwrap_or_default();
                file.lines > 500 || count_matches_in(content, "fn ") > 30
            })
            .map(|file| file.relative_path.clone())
            .collect::<Vec<_>>();

        let mut complexity_indicators = Vec::new();
        for file in &rust_files {
            let content = file.content.as_deref().unwrap_or_default();
            let branch_count = [" if ", " match ", " while ", " for ", "&&", "||"]
                .iter()
                .map(|needle| content.matches(needle).count())
                .sum::<usize>();
            if branch_count > 80 {
                complexity_indicators.push(format!(
                    "{} has high branching density ({branch_count} indicators)",
                    file.relative_path
                ));
            }
        }

        let mut score = 100_i32;
        score -= (large_modules.len() as i32 * 8).min(32);
        score -= (god_module_candidates.len() as i32 * 12).min(36);
        score -= (complexity_indicators.len() as i32 * 8).min(24);
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

fn count_code_lines(content: &str) -> usize {
    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("//"))
        .count()
}

fn count_matches(file: &FileSnapshot, needle: &str) -> usize {
    count_matches_in(file.content.as_deref().unwrap_or_default(), needle)
}

fn count_matches_in(content: &str, needle: &str) -> usize {
    content.matches(needle).count()
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
