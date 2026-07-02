use crate::{
    analyzers::Analyzer,
    collector::RepositorySnapshot,
    model::{
        ArchitectureAssessment, CodeQuality, DependencyHealth, Overview, RiskFinding, RiskReport,
        Severity, TestingMaturity,
    },
};

pub struct RiskAnalyzer<'a> {
    overview: &'a Overview,
    dependencies: &'a DependencyHealth,
    code_quality: &'a CodeQuality,
    architecture: &'a ArchitectureAssessment,
    testing: &'a TestingMaturity,
}

impl<'a> RiskAnalyzer<'a> {
    pub fn new(
        overview: &'a Overview,
        dependencies: &'a DependencyHealth,
        code_quality: &'a CodeQuality,
        architecture: &'a ArchitectureAssessment,
        testing: &'a TestingMaturity,
    ) -> Self {
        Self {
            overview,
            dependencies,
            code_quality,
            architecture,
            testing,
        }
    }
}

impl Analyzer<RiskReport> for RiskAnalyzer<'_> {
    fn analyze(&self, _snapshot: &RepositorySnapshot) -> RiskReport {
        let mut findings = Vec::new();

        if self.overview.package_count <= 1 && self.code_quality.lines_of_code > 8_000 {
            findings.push(RiskFinding {
                severity: Severity::Medium,
                title: "Bus factor and ownership concentration".into(),
                evidence: "Large codebase concentrated in a single package.".into(),
                recommendation:
                    "Review ownership boundaries and split high-change domains into explicit crates."
                        .into(),
            });
        }
        if !self.code_quality.god_module_candidates.is_empty() {
            findings.push(RiskFinding {
                severity: Severity::High,
                title: "Potential God modules".into(),
                evidence: self.code_quality.god_module_candidates.join(", "),
                recommendation:
                    "Extract cohesive submodules and isolate orchestration from domain behavior."
                        .into(),
            });
        }
        if !self.dependencies.maintenance_risks.is_empty() {
            findings.push(RiskFinding {
                severity: Severity::Medium,
                title: "Dependency maintenance risk".into(),
                evidence: self.dependencies.maintenance_risks.join("; "),
                recommendation:
                    "Review dependency sourcing, version policy, and upgrade ownership.".into(),
            });
        }
        if !self.testing.has_tests {
            findings.push(RiskFinding {
                severity: Severity::High,
                title: "Lack of automated tests".into(),
                evidence: "No unit or integration tests were detected.".into(),
                recommendation:
                    "Introduce smoke, integration, and critical-path unit tests before major investment."
                        .into(),
            });
        }
        if self.dependencies.total_dependencies > 60 {
            findings.push(RiskFinding {
                severity: Severity::Medium,
                title: "Excessive dependency concentration".into(),
                evidence: format!(
                    "{} direct dependencies were detected.",
                    self.dependencies.total_dependencies
                ),
                recommendation:
                    "Identify strategic dependencies and remove low-value transitive surface area."
                        .into(),
            });
        }
        if !self.architecture.module_centralization_risks.is_empty() {
            findings.push(RiskFinding {
                severity: Severity::Medium,
                title: "Module centralization risk".into(),
                evidence: self.architecture.module_centralization_risks.join("; "),
                recommendation:
                    "Review module directionality and enforce dependency rules at crate boundaries."
                        .into(),
            });
        }
        if !self.architecture.circular_dependencies.is_empty() {
            findings.push(RiskFinding {
                severity: Severity::Medium,
                title: "Circular module dependency".into(),
                evidence: self.architecture.circular_dependencies.join("; "),
                recommendation:
                    "Break top-level module cycles or move shared contracts behind an explicit boundary."
                        .into(),
            });
        }

        let mut score = 100_i32;
        for finding in &findings {
            score -= match finding.severity {
                Severity::Low => 4,
                Severity::Medium => 10,
                Severity::High => 18,
            };
        }

        RiskReport {
            findings,
            score: score.clamp(0, 100) as u8,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        analyzers::{risks::RiskAnalyzer, Analyzer},
        collector::{CargoManifest, RepositorySnapshot},
        model::{ArchitectureAssessment, CodeQuality, DependencyHealth, Overview, TestingMaturity},
    };

    #[test]
    fn empty_low_risk_inputs_generate_no_findings() {
        let overview = overview(2, 1);
        let dependencies = dependencies(4, Vec::new());
        let code_quality = code_quality(1_000, Vec::new());
        let architecture = architecture(Vec::new(), Vec::new());
        let testing = testing(true);

        let report = RiskAnalyzer::new(
            &overview,
            &dependencies,
            &code_quality,
            &architecture,
            &testing,
        )
        .analyze(&snapshot());

        assert!(report.findings.is_empty());
        assert_eq!(report.score, 100);
    }

    #[test]
    fn typical_dependency_and_testing_gaps_generate_findings() {
        let overview = overview(1, 1);
        let dependencies = dependencies(4, vec!["path dependency".into()]);
        let code_quality = code_quality(1_000, Vec::new());
        let architecture = architecture(Vec::new(), Vec::new());
        let testing = testing(false);

        let report = RiskAnalyzer::new(
            &overview,
            &dependencies,
            &code_quality,
            &architecture,
            &testing,
        )
        .analyze(&snapshot());

        assert_eq!(report.findings.len(), 2);
        assert!(report.score < 80);
    }

    #[test]
    fn extreme_single_package_and_god_modules_generate_high_findings() {
        let overview = overview(1, 1);
        let dependencies = dependencies(70, Vec::new());
        let code_quality = code_quality(9_000, vec!["src/large.rs".into()]);
        let architecture = architecture(Vec::new(), Vec::new());
        let testing = testing(true);

        let report = RiskAnalyzer::new(
            &overview,
            &dependencies,
            &code_quality,
            &architecture,
            &testing,
        )
        .analyze(&snapshot());

        assert!(report
            .findings
            .iter()
            .any(|finding| finding.title == "Potential God modules"));
        assert!(report
            .findings
            .iter()
            .any(|finding| finding.title == "Excessive dependency concentration"));
    }

    #[test]
    fn adversarial_architecture_names_are_not_conflated() {
        let overview = overview(2, 1);
        let dependencies = dependencies(4, Vec::new());
        let code_quality = code_quality(1_000, Vec::new());
        let architecture = architecture(
            vec!["src/lib.rs centralizes 25 module declarations".into()],
            vec!["cycle among top-level modules: a -> b".into()],
        );
        let testing = testing(true);

        let report = RiskAnalyzer::new(
            &overview,
            &dependencies,
            &code_quality,
            &architecture,
            &testing,
        )
        .analyze(&snapshot());

        assert!(report
            .findings
            .iter()
            .any(|finding| finding.title == "Module centralization risk"));
        assert!(report
            .findings
            .iter()
            .any(|finding| finding.title == "Circular module dependency"));
    }

    fn overview(package_count: usize, crate_count: usize) -> Overview {
        Overview {
            crate_count,
            package_count,
            workspace_members: Vec::new(),
            total_files: 0,
            total_bytes: 0,
            languages: Vec::new(),
            cargo_configs: Vec::new(),
            summary: String::new(),
        }
    }

    fn dependencies(total_dependencies: usize, maintenance_risks: Vec<String>) -> DependencyHealth {
        DependencyHealth {
            total_dependencies,
            direct_dependencies: total_dependencies,
            critical_dependencies: Vec::new(),
            outdated_indicators: Vec::new(),
            maintenance_risks,
            score: 100,
        }
    }

    fn code_quality(lines_of_code: usize, god_module_candidates: Vec<String>) -> CodeQuality {
        CodeQuality {
            lines_of_code,
            module_count: 1,
            function_count: 0,
            average_function_size: 0.0,
            complexity_indicators: Vec::new(),
            large_modules: Vec::new(),
            god_module_candidates,
            score: 100,
        }
    }

    fn architecture(
        module_centralization_risks: Vec<String>,
        circular_dependencies: Vec<String>,
    ) -> ArchitectureAssessment {
        ArchitectureAssessment {
            detected_layers: Vec::new(),
            domain_boundaries: Vec::new(),
            module_centralization_risks,
            circular_dependencies,
            architecture_style: String::new(),
            separation_of_concerns: String::new(),
            score: 100,
        }
    }

    fn testing(has_tests: bool) -> TestingMaturity {
        TestingMaturity {
            has_tests,
            unit_test_files: usize::from(has_tests),
            integration_test_files: 0,
            test_function_count: usize::from(has_tests),
            testing_structure: String::new(),
            score: 100,
        }
    }

    fn snapshot() -> RepositorySnapshot {
        RepositorySnapshot {
            root: PathBuf::from("/tmp/repo"),
            files: Vec::new(),
            manifests: Vec::<CargoManifest>::new(),
        }
    }
}
