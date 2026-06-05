use crate::model::{AuditReport, RiskFinding};

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
    use super::quoted;

    #[test]
    fn escapes_json_strings() {
        assert_eq!(quoted("a\"b\\c"), "\"a\\\"b\\\\c\"");
    }
}
