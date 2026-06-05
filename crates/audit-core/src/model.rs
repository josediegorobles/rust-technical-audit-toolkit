#[derive(Debug, Clone)]
pub struct AuditReport {
    pub repository_path: String,
    pub overview: Overview,
    pub dependencies: DependencyHealth,
    pub code_quality: CodeQuality,
    pub architecture: ArchitectureAssessment,
    pub testing: TestingMaturity,
    pub risks: RiskReport,
    pub overall_score: u8,
}

pub trait AnalysisSection {}

#[derive(Debug, Clone)]
pub struct Overview {
    pub crate_count: usize,
    pub package_count: usize,
    pub workspace_members: Vec<String>,
    pub total_files: usize,
    pub total_bytes: u64,
    pub languages: Vec<LanguageStat>,
    pub cargo_configs: Vec<String>,
    pub summary: String,
}

impl AnalysisSection for Overview {}

#[derive(Debug, Clone)]
pub struct LanguageStat {
    pub language: String,
    pub files: usize,
    pub lines: usize,
}

#[derive(Debug, Clone)]
pub struct DependencyHealth {
    pub total_dependencies: usize,
    pub direct_dependencies: usize,
    pub critical_dependencies: Vec<String>,
    pub outdated_indicators: Vec<String>,
    pub maintenance_risks: Vec<String>,
    pub score: u8,
}

impl AnalysisSection for DependencyHealth {}

#[derive(Debug, Clone)]
pub struct CodeQuality {
    pub lines_of_code: usize,
    pub module_count: usize,
    pub function_count: usize,
    pub average_function_size: f32,
    pub complexity_indicators: Vec<String>,
    pub large_modules: Vec<String>,
    pub god_module_candidates: Vec<String>,
    pub score: u8,
}

impl AnalysisSection for CodeQuality {}

#[derive(Debug, Clone)]
pub struct ArchitectureAssessment {
    pub detected_layers: Vec<String>,
    pub domain_boundaries: Vec<String>,
    pub circular_dependency_risks: Vec<String>,
    pub architecture_style: String,
    pub separation_of_concerns: String,
    pub score: u8,
}

impl AnalysisSection for ArchitectureAssessment {}

#[derive(Debug, Clone)]
pub struct TestingMaturity {
    pub has_tests: bool,
    pub unit_test_files: usize,
    pub integration_test_files: usize,
    pub test_function_count: usize,
    pub testing_structure: String,
    pub score: u8,
}

impl AnalysisSection for TestingMaturity {}

#[derive(Debug, Clone)]
pub struct RiskReport {
    pub findings: Vec<RiskFinding>,
    pub score: u8,
}

impl AnalysisSection for RiskReport {}

#[derive(Debug, Clone)]
pub struct RiskFinding {
    pub severity: Severity,
    pub title: String,
    pub evidence: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, Copy)]
pub enum Severity {
    Low,
    Medium,
    High,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}
