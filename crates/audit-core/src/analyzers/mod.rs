use crate::{collector::RepositorySnapshot, model::AnalysisSection};

pub mod architecture;
pub mod code_quality;
pub mod dependencies;
pub mod overview;
pub mod risks;
pub mod testing;

pub trait Analyzer<T: AnalysisSection> {
    fn analyze(&self, snapshot: &RepositorySnapshot) -> T;
}
