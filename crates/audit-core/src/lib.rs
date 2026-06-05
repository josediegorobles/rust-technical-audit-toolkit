pub mod analyzers;
pub mod collector;
pub mod json;
pub mod model;
pub mod report;
pub mod scoring;

use std::path::Path;

use analyzers::{
    architecture::ArchitectureAnalyzer, code_quality::CodeQualityAnalyzer,
    dependencies::DependencyAnalyzer, overview::OverviewAnalyzer, risks::RiskAnalyzer,
    testing::TestingAnalyzer, Analyzer,
};
use collector::RepositorySnapshot;
use model::AuditReport;

pub fn audit_repository(path: &Path) -> Result<AuditReport, String> {
    let snapshot = RepositorySnapshot::collect(path)?;
    let overview = OverviewAnalyzer.analyze(&snapshot);
    let dependencies = DependencyAnalyzer.analyze(&snapshot);
    let code_quality = CodeQualityAnalyzer.analyze(&snapshot);
    let architecture = ArchitectureAnalyzer.analyze(&snapshot);
    let testing = TestingAnalyzer.analyze(&snapshot);
    let risks = RiskAnalyzer::new(
        &overview,
        &dependencies,
        &code_quality,
        &architecture,
        &testing,
    )
    .analyze(&snapshot);
    let overall_score = scoring::overall_score(
        &dependencies,
        &code_quality,
        &architecture,
        &testing,
        &risks,
    );

    Ok(AuditReport {
        repository_path: snapshot.root.display().to_string(),
        overview,
        dependencies,
        code_quality,
        architecture,
        testing,
        risks,
        overall_score,
    })
}
