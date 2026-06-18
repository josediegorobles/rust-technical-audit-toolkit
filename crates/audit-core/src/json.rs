use crate::model::{AuditReport, RiskFinding, Severity};

pub fn render_json(report: &AuditReport) -> String {
    format!(
        concat!(
            "{{\n",
            "  \"repository_path\": {},\n",
            "  \"overall_score\": {},\n",
            "  \"overview\": {},\n",
            "  \"dependencies\": {},\n",
            "  \"code_quality\": {},\n",
            "  \"architecture\": {},\n",
            "  \"testing\": {},\n",
            "  \"risks\": {}\n",
            "}}\n"
        ),
        quoted(&report.repository_path),
        report.overall_score,
        overview_json(report),
        dependencies_json(report),
        code_quality_json(report),
        architecture_json(report),
        testing_json(report),
        risks_json(report)
    )
}

pub fn render_scorecard_json(report: &AuditReport) -> String {
    let risk_counts = RiskCounts::from_report(report);
    format!(
        concat!(
            "{{\n",
            "  \"schema_version\": \"rta.scorecard.v1\",\n",
            "  \"repository_path\": {},\n",
            "  \"overall_score\": {},\n",
            "  \"scores\": {{\n",
            "    \"dependency_health\": {},\n",
            "    \"code_quality\": {},\n",
            "    \"architecture\": {},\n",
            "    \"testing\": {},\n",
            "    \"risk_posture\": {}\n",
            "  }},\n",
            "  \"metrics\": {{\n",
            "    \"crate_count\": {},\n",
            "    \"package_count\": {},\n",
            "    \"direct_dependencies\": {},\n",
            "    \"lines_of_rust_code\": {},\n",
            "    \"rust_modules\": {},\n",
            "    \"function_count\": {},\n",
            "    \"average_function_size\": {:.1},\n",
            "    \"unit_test_files\": {},\n",
            "    \"integration_test_files\": {},\n",
            "    \"test_function_count\": {}\n",
            "  }},\n",
            "  \"risk_findings\": {{\n",
            "    \"total\": {},\n",
            "    \"high\": {},\n",
            "    \"medium\": {},\n",
            "    \"low\": {}\n",
            "  }}\n",
            "}}\n"
        ),
        quoted(&report.repository_path),
        report.overall_score,
        report.dependencies.score,
        report.code_quality.score,
        report.architecture.score,
        report.testing.score,
        report.risks.score,
        report.overview.crate_count,
        report.overview.package_count,
        report.dependencies.direct_dependencies,
        report.code_quality.lines_of_code,
        report.code_quality.module_count,
        report.code_quality.function_count,
        report.code_quality.average_function_size,
        report.testing.unit_test_files,
        report.testing.integration_test_files,
        report.testing.test_function_count,
        report.risks.findings.len(),
        risk_counts.high,
        risk_counts.medium,
        risk_counts.low
    )
}

#[derive(Debug, Default)]
struct RiskCounts {
    high: usize,
    medium: usize,
    low: usize,
}

impl RiskCounts {
    fn from_report(report: &AuditReport) -> Self {
        let mut counts = Self::default();
        for finding in &report.risks.findings {
            match finding.severity {
                Severity::High => counts.high += 1,
                Severity::Medium => counts.medium += 1,
                Severity::Low => counts.low += 1,
            }
        }
        counts
    }
}

fn overview_json(report: &AuditReport) -> String {
    let languages = report
        .overview
        .languages
        .iter()
        .map(|language| {
            format!(
                "{{\"language\":{},\"files\":{},\"lines\":{}}}",
                quoted(&language.language),
                language.files,
                language.lines
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"crate_count\":{},\"package_count\":{},\"workspace_members\":{},\"total_files\":{},\"total_bytes\":{},\"languages\":[{}],\"cargo_configs\":{},\"summary\":{}}}",
        report.overview.crate_count,
        report.overview.package_count,
        string_array(&report.overview.workspace_members),
        report.overview.total_files,
        report.overview.total_bytes,
        languages,
        string_array(&report.overview.cargo_configs),
        quoted(&report.overview.summary)
    )
}

fn dependencies_json(report: &AuditReport) -> String {
    format!(
        "{{\"total_dependencies\":{},\"direct_dependencies\":{},\"critical_dependencies\":{},\"outdated_indicators\":{},\"maintenance_risks\":{},\"score\":{}}}",
        report.dependencies.total_dependencies,
        report.dependencies.direct_dependencies,
        string_array(&report.dependencies.critical_dependencies),
        string_array(&report.dependencies.outdated_indicators),
        string_array(&report.dependencies.maintenance_risks),
        report.dependencies.score
    )
}

fn code_quality_json(report: &AuditReport) -> String {
    format!(
        "{{\"lines_of_code\":{},\"module_count\":{},\"function_count\":{},\"average_function_size\":{:.1},\"complexity_indicators\":{},\"large_modules\":{},\"god_module_candidates\":{},\"score\":{}}}",
        report.code_quality.lines_of_code,
        report.code_quality.module_count,
        report.code_quality.function_count,
        report.code_quality.average_function_size,
        string_array(&report.code_quality.complexity_indicators),
        string_array(&report.code_quality.large_modules),
        string_array(&report.code_quality.god_module_candidates),
        report.code_quality.score
    )
}

fn architecture_json(report: &AuditReport) -> String {
    format!(
        "{{\"detected_layers\":{},\"domain_boundaries\":{},\"circular_dependency_risks\":{},\"architecture_style\":{},\"separation_of_concerns\":{},\"score\":{}}}",
        string_array(&report.architecture.detected_layers),
        string_array(&report.architecture.domain_boundaries),
        string_array(&report.architecture.circular_dependency_risks),
        quoted(&report.architecture.architecture_style),
        quoted(&report.architecture.separation_of_concerns),
        report.architecture.score
    )
}

fn testing_json(report: &AuditReport) -> String {
    format!(
        "{{\"has_tests\":{},\"unit_test_files\":{},\"integration_test_files\":{},\"test_function_count\":{},\"testing_structure\":{},\"score\":{}}}",
        report.testing.has_tests,
        report.testing.unit_test_files,
        report.testing.integration_test_files,
        report.testing.test_function_count,
        quoted(&report.testing.testing_structure),
        report.testing.score
    )
}

fn risks_json(report: &AuditReport) -> String {
    let findings = report
        .risks
        .findings
        .iter()
        .map(risk_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"score\":{},\"findings\":[{}]}}",
        report.risks.score, findings
    )
}

fn risk_json(finding: &RiskFinding) -> String {
    format!(
        "{{\"severity\":{},\"title\":{},\"evidence\":{},\"recommendation\":{}}}",
        quoted(finding.severity.as_str()),
        quoted(&finding.title),
        quoted(&finding.evidence),
        quoted(&finding.recommendation)
    )
}

fn string_array(values: &[String]) -> String {
    let values = values
        .iter()
        .map(|value| quoted(value))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{values}]")
}

fn quoted(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push(' '),
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use crate::model::{
        ArchitectureAssessment, AuditReport, CodeQuality, DependencyHealth, Overview, RiskReport,
        TestingMaturity,
    };

    use super::{quoted, render_scorecard_json};

    #[test]
    fn escapes_json_strings() {
        assert_eq!(quoted("a\"b\\c"), "\"a\\\"b\\\\c\"");
    }

    #[test]
    fn renders_scorecard_schema_version() {
        let report = AuditReport {
            repository_path: "/tmp/repo".into(),
            overview: Overview {
                crate_count: 1,
                package_count: 1,
                workspace_members: Vec::new(),
                total_files: 1,
                total_bytes: 1,
                languages: Vec::new(),
                cargo_configs: Vec::new(),
                summary: "single package".into(),
            },
            dependencies: DependencyHealth {
                total_dependencies: 2,
                direct_dependencies: 2,
                critical_dependencies: Vec::new(),
                outdated_indicators: Vec::new(),
                maintenance_risks: Vec::new(),
                score: 90,
            },
            code_quality: CodeQuality {
                lines_of_code: 100,
                module_count: 2,
                function_count: 5,
                average_function_size: 20.0,
                complexity_indicators: Vec::new(),
                large_modules: Vec::new(),
                god_module_candidates: Vec::new(),
                score: 80,
            },
            architecture: ArchitectureAssessment {
                detected_layers: Vec::new(),
                domain_boundaries: Vec::new(),
                circular_dependency_risks: Vec::new(),
                architecture_style: "compact".into(),
                separation_of_concerns: "clear".into(),
                score: 75,
            },
            testing: TestingMaturity {
                has_tests: true,
                unit_test_files: 1,
                integration_test_files: 0,
                test_function_count: 2,
                testing_structure: "Unit tests detected.".into(),
                score: 70,
            },
            risks: RiskReport {
                findings: Vec::new(),
                score: 100,
            },
            overall_score: 82,
        };

        let rendered = render_scorecard_json(&report);

        assert!(rendered.contains("\"schema_version\": \"rta.scorecard.v1\""));
        assert!(rendered.contains("\"overall_score\": 82"));
        assert!(rendered.contains("\"risk_findings\""));
    }
}
