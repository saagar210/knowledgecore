mod cli;
mod commands {
    pub mod export;
    pub mod ingest;
    pub mod verify;
}
mod verifier;

use clap::Parser;
use cli::{Cli, Command, IngestCmd, VaultCmd};
use kc_core::vault::{vault_init, vault_open};

fn now_ms() -> i64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time before unix epoch");
    now.as_millis() as i64
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.cmd {
        Command::Vault { cmd } => match cmd {
            VaultCmd::Init { vault_path, vault_slug } => {
                let created = vault_init(std::path::Path::new(&vault_path), &vault_slug, now_ms());
                if let Ok(v) = &created {
                    println!("vault initialized: {} ({})", v.vault_slug, v.vault_id);
                }
                created.map(|_| ())
            }
            VaultCmd::Open { vault_path } => {
                let opened = vault_open(std::path::Path::new(&vault_path));
                if let Ok(v) = &opened {
                    println!("vault opened: {} ({})", v.vault_slug, v.vault_id);
                }
                opened.map(|_| ())
            }
        },
        Command::Ingest { cmd } => match cmd {
            IngestCmd::ScanFolder {
                vault_path,
                scan_root,
                source_kind,
            } => commands::ingest::ingest_scan_folder(&vault_path, &scan_root, &source_kind),
            IngestCmd::InboxOnce {
                vault_path,
                file_path,
                source_kind,
            } => commands::ingest::ingest_inbox_once(&vault_path, &file_path, &source_kind),
        },
        Command::Export {
            vault_path,
            export_dir,
        } => commands::export::run_export(&vault_path, &export_dir, now_ms()).map(|bundle| {
            println!("exported bundle: {}", bundle.display());
        }),
        Command::Verify { bundle_path } => commands::verify::run_verify(&bundle_path).map(|(code, report)| {
            println!("{}", serde_json::to_string(&report).unwrap_or_else(|_| "{}".to_string()));
            if code != 0 {
                std::process::exit(code as i32);
            }
        }),
    };

    if let Err(err) = result {
        eprintln!("{}: {}", err.code, err.message);
        std::process::exit(1);
    }
}
