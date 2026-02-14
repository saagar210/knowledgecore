use kc_core::app_error::AppResult;

#[derive(Debug, Clone)]
pub struct LexicalCandidates;

#[derive(Debug, Clone)]
pub struct VectorCandidates;

pub trait IndexService {
    fn rebuild(&self) -> AppResult<()>;
}
