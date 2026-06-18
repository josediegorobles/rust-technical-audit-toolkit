use std::path::PathBuf;

use rta_core::{
    audit_repository,
    json::render_scorecard_json,
    model::{
        ArchitectureAssessment, AuditReport, CodeQuality, DependencyHealth, Overview, RiskFinding,
        RiskReport, Severity, TestingMaturity,
    },
    report::render_markdown,
    scoring::overall_score,
};
use serde_json::Value;

#[test]
fn markdown_report_renders_key_due_diligence_sections() {
    let markdown = render_markdown(&fixture_report());

    for section in [
        "# Rust Technical Audit Report",
        "## Executive Summary",
        "### Weighted Scoring Model",
        "## Architecture",
        "## Dependency Health",
        "## Code Quality",
        "## Testing",
        "## Risks",
        "## Recommendations",
    ] {
        assert!(
            markdown.contains(section),
            "markdown report should contain {section}"
        );
    }

    assert!(markdown.contains("| Dependency Health | 20% | 90 |"));
    assert!(markdown.contains("### Dependency policy gap (medium)"));
    assert!(markdown
        .contains("Repository `/tmp/repo` received an overall technical due diligence score"));
}

#[test]
fn scorecard_json_schema_is_stable() {
    let scorecard: Value = serde_json::from_str(&render_scorecard_json(&fixture_report()))
        .expect("scorecard should be valid JSON");

    assert_eq!(scorecard["schema_version"], "rta.scorecard.v1");
    assert_eq!(scorecard["repository_path"], "/tmp/repo");
    assert_eq!(scorecard["overall_score"], 82);

    assert_eq!(scorecard["scores"]["dependency_health"], 90);
    assert_eq!(scorecard["scores"]["code_quality"], 80);
    assert_eq!(scorecard["scores"]["architecture"], 75);
    assert_eq!(scorecard["scores"]["testing"], 70);
    assert_eq!(scorecard["scores"]["risk_posture"], 88);

    assert_eq!(scorecard["metrics"]["crate_count"], 1);
    assert_eq!(scorecard["metrics"]["package_count"], 1);
    assert_eq!(scorecard["metrics"]["direct_dependencies"], 3);
    assert_eq!(scorecard["metrics"]["lines_of_rust_code"], 120);
    assert_eq!(scorecard["metrics"]["rust_modules"], 4);
    assert_eq!(scorecard["metrics"]["function_count"], 12);
    assert_eq!(scorecard["metrics"]["average_function_size"], 10.0);
    assert_eq!(scorecard["metrics"]["unit_test_files"], 1);
    assert_eq!(scorecard["metrics"]["integration_test_files"], 1);
    assert_eq!(scorecard["metrics"]["test_function_count"], 3);

    assert_eq!(scorecard["risk_findings"]["total"], 2);
    assert_eq!(scorecard["risk_findings"]["high"], 1);
    assert_eq!(scorecard["risk_findings"]["medium"], 1);
    assert_eq!(scorecard["risk_findings"]["low"], 0);
}

#[test]
fn scoring_uses_documented_weights() {
    let dependencies = DependencyHealth {
        score: 90,
        ..empty_dependencies()
    };
    let code_quality = CodeQuality {
        score: 80,
        ..empty_code_quality()
    };
    let architecture = ArchitectureAssessment {
        score: 70,
        ..empty_architecture()
    };
    let testing = TestingMaturity {
        score: 60,
        ..empty_testing()
    };
    let risks = RiskReport {
        score: 50,
        findings: Vec::new(),
    };

    assert_eq!(
        overall_score(
            &dependencies,
            &code_quality,
            &architecture,
            &testing,
            &risks
        ),
        72
    );
}

#[test]
fn scoring_preserves_zero_and_hundred_extremes() {
    let zero_dependencies = DependencyHealth {
        score: 0,
        ..empty_dependencies()
    };
    let zero_code_quality = CodeQuality {
        score: 0,
        ..empty_code_quality()
    };
    let zero_architecture = ArchitectureAssessment {
        score: 0,
        ..empty_architecture()
    };
    let zero_testing = TestingMaturity {
        score: 0,
        ..empty_testing()
    };
    let zero_risks = RiskReport {
        score: 0,
        findings: Vec::new(),
    };
    assert_eq!(
        overall_score(
            &zero_dependencies,
            &zero_code_quality,
            &zero_architecture,
            &zero_testing,
            &zero_risks
        ),
        0
    );

    let max_dependencies = DependencyHealth {
        score: 100,
        ..empty_dependencies()
    };
    let max_code_quality = CodeQuality {
        score: 100,
        ..empty_code_quality()
    };
    let max_architecture = ArchitectureAssessment {
        score: 100,
        ..empty_architecture()
    };
    let max_testing = TestingMaturity {
        score: 100,
        ..empty_testing()
    };
    let max_risks = RiskReport {
        score: 100,
        findings: Vec::new(),
    };
    assert_eq!(
        overall_score(
            &max_dependencies,
            &max_code_quality,
            &max_architecture,
            &max_testing,
            &max_risks
        ),
        100
    );
}

#[test]
fn sample_rust_service_detects_architecture_testing_and_dependencies() {
    let report = audit_repository(&sample_service_path()).expect("sample service should audit");

    assert_eq!(report.overview.package_count, 1);
    assert_eq!(report.dependencies.direct_dependencies, 4);
    assert!(report
        .dependencies
        .critical_dependencies
        .iter()
        .any(|dependency| dependency.starts_with("axum in ")));
    assert!(report
        .dependencies
        .outdated_indicators
        .iter()
        .any(|indicator| indicator.starts_with("axum is pinned to pre-1.0")));

    assert_eq!(
        report.architecture.detected_layers,
        ["api", "domain", "repository", "service"]
    );
    assert_eq!(
        report.architecture.architecture_style,
        "layered single service or compact workspace"
    );

    assert!(report.testing.has_tests);
    assert_eq!(report.testing.unit_test_files, 1);
    assert_eq!(report.testing.integration_test_files, 1);
    assert_eq!(report.testing.test_function_count, 2);
}

fn sample_service_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/sample-rust-service")
}

fn fixture_report() -> AuditReport {
    AuditReport {
        repository_path: "/tmp/repo".into(),
        overview: Overview {
            crate_count: 1,
            package_count: 1,
            workspace_members: Vec::new(),
            total_files: 8,
            total_bytes: 4096,
            languages: Vec::new(),
            cargo_configs: vec!["Cargo.toml".into()],
            summary: "Single-package Rust repository with 1 Cargo manifest(s).".into(),
        },
        dependencies: DependencyHealth {
            total_dependencies: 3,
            direct_dependencies: 3,
            critical_dependencies: vec!["tokio in Cargo.toml".into()],
            outdated_indicators: Vec::new(),
            maintenance_risks: vec!["path dependency requires ownership review".into()],
            score: 90,
        },
        code_quality: CodeQuality {
            lines_of_code: 120,
            module_count: 4,
            function_count: 12,
            average_function_size: 10.0,
            complexity_indicators: Vec::new(),
            large_modules: Vec::new(),
            god_module_candidates: Vec::new(),
            score: 80,
        },
        architecture: ArchitectureAssessment {
            detected_layers: vec!["api".into(), "domain".into(), "repository".into()],
            domain_boundaries: vec!["users".into()],
            circular_dependency_risks: Vec::new(),
            architecture_style: "layered single service or compact workspace".into(),
            separation_of_concerns:
                "Some separation of concerns is visible, but boundaries should be reviewed.".into(),
            score: 75,
        },
        testing: TestingMaturity {
            has_tests: true,
            unit_test_files: 1,
            integration_test_files: 1,
            test_function_count: 3,
            testing_structure: "Unit and integration testing structures detected.".into(),
            score: 70,
        },
        risks: RiskReport {
            findings: vec![
                RiskFinding {
                    severity: Severity::Medium,
                    title: "Dependency policy gap".into(),
                    evidence: "One dependency uses a broad declaration.".into(),
                    recommendation: "Assign explicit ownership and upgrade cadence.".into(),
                },
                RiskFinding {
                    severity: Severity::High,
                    title: "Critical path lacks coverage".into(),
                    evidence: "Core workflow has no regression test.".into(),
                    recommendation: "Add a regression test around the critical workflow.".into(),
                },
            ],
            score: 88,
        },
        overall_score: 82,
    }
}

fn empty_dependencies() -> DependencyHealth {
    DependencyHealth {
        total_dependencies: 0,
        direct_dependencies: 0,
        critical_dependencies: Vec::new(),
        outdated_indicators: Vec::new(),
        maintenance_risks: Vec::new(),
        score: 0,
    }
}

fn empty_code_quality() -> CodeQuality {
    CodeQuality {
        lines_of_code: 0,
        module_count: 0,
        function_count: 0,
        average_function_size: 0.0,
        complexity_indicators: Vec::new(),
        large_modules: Vec::new(),
        god_module_candidates: Vec::new(),
        score: 0,
    }
}

fn empty_architecture() -> ArchitectureAssessment {
    ArchitectureAssessment {
        detected_layers: Vec::new(),
        domain_boundaries: Vec::new(),
        circular_dependency_risks: Vec::new(),
        architecture_style: String::new(),
        separation_of_concerns: String::new(),
        score: 0,
    }
}

fn empty_testing() -> TestingMaturity {
    TestingMaturity {
        has_tests: false,
        unit_test_files: 0,
        integration_test_files: 0,
        test_function_count: 0,
        testing_structure: String::new(),
        score: 0,
    }
}
