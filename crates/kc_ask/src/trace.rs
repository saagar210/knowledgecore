use kc_core::app_error::{AppError, AppResult};

pub fn write_trace() -> AppResult<()> {
    Err(AppError::internal("trace writer not implemented yet"))
}
