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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        analyzers::Analyzer,
        collector::{CargoManifest, FileSnapshot, RepositorySnapshot},
    };

    use super::TestingAnalyzer;

    #[test]
    fn empty_repository_has_low_testing_maturity() {
        let report = TestingAnalyzer.analyze(&snapshot(Vec::new()));

        assert!(!report.has_tests);
        assert_eq!(report.score, 20);
    }

    #[test]
    fn typical_repository_detects_unit_and_integration_tests() {
        let report = TestingAnalyzer.analyze(&snapshot(vec![
            file(
                "src/lib.rs",
                "#[cfg(test)]\nmod tests {\n#[test]\nfn it_works() {}\n}\n",
            ),
            file("tests/smoke.rs", "#[test]\nfn smoke() {}\n"),
        ]));

        assert!(report.has_tests);
        assert_eq!(report.unit_test_files, 1);
        assert_eq!(report.integration_test_files, 1);
        assert_eq!(report.test_function_count, 2);
    }

    #[test]
    fn extreme_test_count_caps_score_at_hundred() {
        let mut content = String::new();
        for index in 0..80 {
            content.push_str(&format!("#[test]\nfn test_{index}() {{}}\n"));
        }
        let report = TestingAnalyzer.analyze(&snapshot(vec![
            file(
                "src/lib.rs",
                "#[cfg(test)]\nmod tests {\n#[test]\nfn unit() {}\n}\n",
            ),
            file(
                "src/core.rs",
                "#[cfg(test)]\nmod tests {\n#[test]\nfn unit() {}\n}\n",
            ),
            file(
                "src/api.rs",
                "#[cfg(test)]\nmod tests {\n#[test]\nfn unit() {}\n}\n",
            ),
            file(
                "src/db.rs",
                "#[cfg(test)]\nmod tests {\n#[test]\nfn unit() {}\n}\n",
            ),
            file("tests/many.rs", &content),
            file("tests/smoke.rs", "#[test]\nfn smoke() {}\n"),
        ]));

        assert_eq!(report.test_function_count, 85);
        assert_eq!(report.score, 100);
    }

    #[test]
    fn adversarial_commented_test_attribute_is_ignored() {
        let report = TestingAnalyzer.analyze(&snapshot(vec![file(
            "src/lib.rs",
            "// #[test]\n// fn fake() {}\npub fn real() {}\n",
        )]));

        assert!(!report.has_tests);
        assert_eq!(report.test_function_count, 0);
    }

    fn snapshot(files: Vec<FileSnapshot>) -> RepositorySnapshot {
        RepositorySnapshot {
            root: PathBuf::from("/tmp/repo"),
            files,
            manifests: Vec::<CargoManifest>::new(),
        }
    }

    fn file(relative_path: &str, content: &str) -> FileSnapshot {
        FileSnapshot {
            path: PathBuf::from("/tmp/repo").join(relative_path),
            relative_path: relative_path.into(),
            extension: Some("rs".into()),
            bytes: content.len() as u64,
            lines: content.lines().count(),
            content: Some(content.into()),
        }
    }
}
