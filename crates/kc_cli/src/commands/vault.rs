use kc_core::app_error::AppResult;
use kc_core::db::open_db;
use kc_core::vault::vault_open;
use std::path::Path;

pub fn run_verify(vault_path: &str) -> AppResult<()> {
    let vault = vault_open(Path::new(vault_path))?;
    let _db = open_db(&Path::new(vault_path).join(vault.db.relative_path))?;
    println!("vault verify ok");
    Ok(())
}
