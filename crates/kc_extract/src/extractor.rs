use kc_core::app_error::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct ToolchainIdentity {
    pub pdfium_identity: String,
    pub tesseract_identity: String,
}

#[derive(Debug, Clone)]
pub struct ExtractInput<'a> {
    pub bytes: &'a [u8],
    pub mime: &'a str,
    pub source_kind: &'a str,
}

#[derive(Debug, Clone)]
pub struct ExtractOutput {
    pub canonical_text: String,
}

pub trait Extractor: Send + Sync {
    fn extract(&self, _input: ExtractInput<'_>) -> AppResult<ExtractOutput>;
}

pub struct NoopExtractor;

impl Extractor for NoopExtractor {
    fn extract(&self, _input: ExtractInput<'_>) -> AppResult<ExtractOutput> {
        Err(AppError::internal("extractor not implemented yet"))
    }
}
