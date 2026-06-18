use crate::model::{AuditReport, RiskFinding};

pub fn render_evidence_json(report: &AuditReport) -> String {
    format!(
        concat!(
            "{{\n",
            "  \"schema_version\": \"rta.evidence.v1\",\n",
            "  \"repository_path\": {},\n",
            "  \"scores\": {{\n",
            "    \"overall\": {},\n",
            "    \"dependency_health\": {},\n",
            "    \"code_quality\": {},\n",
            "    \"architecture\": {},\n",
            "    \"testing\": {},\n",
            "    \"risk_posture\": {}\n",
            "  }},\n",
            "  \"evidence\": {{\n",
            "    \"overview\": {{\n",
            "      \"crate_count\": {},\n",
            "      \"package_count\": {},\n",
            "      \"workspace_members\": {},\n",
            "      \"cargo_configs\": {},\n",
            "      \"total_files\": {},\n",
            "      \"total_bytes\": {}\n",
            "    }},\n",
            "    \"dependencies\": {{\n",
            "      \"direct_dependencies\": {},\n",
            "      \"critical_dependencies\": {},\n",
            "      \"maintenance_risks\": {},\n",
            "      \"outdated_indicators\": {}\n",
            "    }},\n",
            "    \"architecture\": {{\n",
            "      \"style\": {},\n",
            "      \"separation_of_concerns\": {},\n",
            "      \"detected_layers\": {},\n",
            "      \"domain_boundaries\": {},\n",
            "      \"circular_dependency_risks\": {}\n",
            "    }},\n",
            "    \"code_quality\": {{\n",
            "      \"lines_of_rust_code\": {},\n",
            "      \"rust_modules\": {},\n",
            "      \"function_count\": {},\n",
            "      \"average_function_size\": {:.1},\n",
            "      \"complexity_indicators\": {},\n",
            "      \"large_modules\": {},\n",
            "      \"god_module_candidates\": {}\n",
            "    }},\n",
            "    \"testing\": {{\n",
            "      \"has_tests\": {},\n",
            "      \"unit_test_files\": {},\n",
            "      \"integration_test_files\": {},\n",
            "      \"test_function_count\": {},\n",
            "      \"testing_structure\": {}\n",
            "    }}\n",
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
        string_array(&report.overview.workspace_members),
        string_array(&report.overview.cargo_configs),
        report.overview.total_files,
        report.overview.total_bytes,
        report.dependencies.direct_dependencies,
        string_array(&report.dependencies.critical_dependencies),
        string_array(&report.dependencies.maintenance_risks),
        string_array(&report.dependencies.outdated_indicators),
        quoted(&report.architecture.architecture_style),
        quoted(&report.architecture.separation_of_concerns),
        string_array(&report.architecture.detected_layers),
        string_array(&report.architecture.domain_boundaries),
        string_array(&report.architecture.circular_dependency_risks),
        report.code_quality.lines_of_code,
        report.code_quality.module_count,
        report.code_quality.function_count,
        report.code_quality.average_function_size,
        string_array(&report.code_quality.complexity_indicators),
        string_array(&report.code_quality.large_modules),
        string_array(&report.code_quality.god_module_candidates),
        report.testing.has_tests,
        report.testing.unit_test_files,
        report.testing.integration_test_files,
        report.testing.test_function_count,
        quoted(&report.testing.testing_structure)
    )
}

pub fn render_risk_register_json(report: &AuditReport) -> String {
    let findings = report
        .risks
        .findings
        .iter()
        .enumerate()
        .map(|(idx, finding)| risk_register_finding_json(idx + 1, finding))
        .collect::<Vec<_>>()
        .join(",\n");

    format!(
        concat!(
            "{{\n",
            "  \"schema_version\": \"rta.risk-register.v1\",\n",
            "  \"repository_path\": {},\n",
            "  \"risk_score\": {},\n",
            "  \"finding_count\": {},\n",
            "  \"findings\": [\n",
            "{}\n",
            "  ]\n",
            "}}\n"
        ),
        quoted(&report.repository_path),
        report.risks.score,
        report.risks.findings.len(),
        findings
    )
}

pub fn render_review_questions_markdown(report: &AuditReport) -> String {
    let score_questions = score_review_questions(report);
    let finding_questions = finding_review_questions(report);

    let mut out = String::new();
    out.push_str("# Review Questions\n\n");
    out.push_str("## Score-Driven Questions\n\n");
    push_questions(&mut out, &score_questions);
    out.push('\n');
    out.push_str("## Finding-Driven Questions\n\n");
    push_questions(&mut out, &finding_questions);
    out
}

pub fn render_methodology_markdown() -> String {
    concat!(
        "# Methodology\n\n",
        "Rust Technical Audit Toolkit produces a heuristic technical due diligence snapshot from local repository evidence.\n\n",
        "## Scope\n\n",
        "- Repository structure, Cargo manifests, Rust source files, dependency declarations, test structure, and generated risk findings.\n",
        "- No AI analysis, network calls, package registry lookups, or external scanners are used by the evidence pack command.\n\n",
        "## Scoring Model\n\n",
        "| Area | Weight |\n",
        "| --- | ---: |\n",
        "| Dependency Health | 20% |\n",
        "| Code Quality | 25% |\n",
        "| Architecture | 25% |\n",
        "| Testing | 15% |\n",
        "| Risk Posture | 15% |\n\n",
        "## Interpretation\n\n",
        "Scores are triage indicators for senior engineering review, not absolute judgments. Evidence files preserve the signals used to produce the report so reviewers can validate, challenge, and deepen the assessment.\n"
    )
    .to_string()
}

fn risk_register_finding_json(index: usize, finding: &RiskFinding) -> String {
    format!(
        concat!(
            "    {{\n",
            "      \"id\": \"RISK-{:03}\",\n",
            "      \"severity\": {},\n",
            "      \"title\": {},\n",
            "      \"evidence\": {},\n",
            "      \"recommendation\": {}\n",
            "    }}"
        ),
        index,
        quoted(finding.severity.as_str()),
        quoted(&finding.title),
        quoted(&finding.evidence),
        quoted(&finding.recommendation)
    )
}

fn score_review_questions(report: &AuditReport) -> Vec<String> {
    let mut questions = Vec::new();
    if report.dependencies.score < 80 {
        questions.push(format!(
            "Dependency Health scored {}/100. Which critical dependencies lack explicit ownership, upgrade cadence, or replacement options?",
            report.dependencies.score
        ));
    }
    if report.code_quality.score < 80 {
        questions.push(format!(
            "Code Quality scored {}/100. Which large modules, branching hotspots, or God module candidates should be decomposed first?",
            report.code_quality.score
        ));
    }
    if report.architecture.score < 80 {
        questions.push(format!(
            "Architecture scored {}/100. Which boundaries are intended, documented, and enforced by crate or module ownership?",
            report.architecture.score
        ));
    }
    if report.testing.score < 80 {
        questions.push(format!(
            "Testing scored {}/100. Which critical business flows need regression or integration coverage before investment?",
            report.testing.score
        ));
    }
    if report.risks.score < 90 {
        questions.push(format!(
            "Risk Posture scored {}/100. Which risks should block, condition, or re-price the diligence decision?",
            report.risks.score
        ));
    }

    if questions.is_empty() {
        questions.push(
            "Which assumptions behind the high scores should be manually verified before relying on the audit?".into(),
        );
    }

    questions
}

fn finding_review_questions(report: &AuditReport) -> Vec<String> {
    if report.risks.findings.is_empty() {
        return vec![
            "No material findings were generated. Which risks are outside the current rule set and require manual review?".into(),
        ];
    }

    report
        .risks
        .findings
        .iter()
        .map(|finding| {
            format!(
                "{} finding `{}`: what evidence would confirm severity, ownership, and remediation cost?",
                finding.severity.as_str(),
                finding.title
            )
        })
        .collect()
}

fn push_questions(out: &mut String, questions: &[String]) {
    for question in questions {
        out.push_str(&format!("- {question}\n"));
    }
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
