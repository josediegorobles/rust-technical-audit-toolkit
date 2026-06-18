use crate::{analyzers::Analyzer, collector::RepositorySnapshot, model::TestingMaturity};

pub struct TestingAnalyzer;

impl Analyzer<TestingMaturity> for TestingAnalyzer {
    fn analyze(&self, snapshot: &RepositorySnapshot) -> TestingMaturity {
        let mut unit_test_files = 0;
        let mut integration_test_files = 0;
        let mut test_function_count = 0;

        for file in snapshot.rust_files() {
            let content = file.content.as_deref().unwrap_or_default();
            let is_integration_test =
                file.relative_path.starts_with("tests/") || file.relative_path.contains("/tests/");
            let has_unit_tests = content.contains("#[cfg(test)]") || has_test_attribute(content);

            if is_integration_test {
                integration_test_files += 1;
                test_function_count += count_test_functions(content);
            } else if has_unit_tests {
                unit_test_files += 1;
                test_function_count += count_test_functions(content);
            }
        }

        let has_tests = unit_test_files > 0 || integration_test_files > 0;
        let testing_structure = if !has_tests {
            "No Rust test structure detected.".to_string()
        } else if integration_test_files > 0 && unit_test_files > 0 {
            "Unit and integration testing structures detected.".to_string()
        } else if integration_test_files > 0 {
            "Integration tests detected; unit test coverage should be reviewed.".to_string()
        } else {
            "Unit tests detected; integration test coverage should be reviewed.".to_string()
        };

        let mut score = if has_tests { 55_i32 } else { 20_i32 };
        score += (unit_test_files as i32 * 6).min(20);
        score += (integration_test_files as i32 * 10).min(20);
        score += (test_function_count as i32).min(15);

        TestingMaturity {
            has_tests,
            unit_test_files,
            integration_test_files,
            test_function_count,
            testing_structure,
            score: score.clamp(0, 100) as u8,
        }
    }
}

fn count_test_functions(content: &str) -> usize {
    content
        .lines()
        .map(str::trim)
        .filter(|line| is_test_attribute(line))
        .count()
}

fn has_test_attribute(content: &str) -> bool {
    content.lines().map(str::trim).any(is_test_attribute)
}

fn is_test_attribute(line: &str) -> bool {
    line == "#[test]" || line.starts_with("#[tokio::test")
}
