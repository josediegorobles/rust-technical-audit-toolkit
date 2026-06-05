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
        if !self.architecture.circular_dependency_risks.is_empty() {
            findings.push(RiskFinding {
                severity: Severity::Medium,
                title: "Circular dependency risk".into(),
                evidence: self.architecture.circular_dependency_risks.join("; "),
                recommendation:
                    "Review module directionality and enforce dependency rules at crate boundaries."
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
