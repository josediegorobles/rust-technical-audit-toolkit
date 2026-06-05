use crate::model::{
    ArchitectureAssessment, CodeQuality, DependencyHealth, RiskReport, TestingMaturity,
};

pub fn overall_score(
    dependencies: &DependencyHealth,
    code_quality: &CodeQuality,
    architecture: &ArchitectureAssessment,
    testing: &TestingMaturity,
    risks: &RiskReport,
) -> u8 {
    let weighted = dependencies.score as f32 * 0.20
        + code_quality.score as f32 * 0.25
        + architecture.score as f32 * 0.25
        + testing.score as f32 * 0.15
        + risks.score as f32 * 0.15;
    weighted.round().clamp(0.0, 100.0) as u8
}
