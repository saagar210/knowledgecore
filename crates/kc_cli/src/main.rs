mod cli;
mod commands {
    pub mod bench;
    pub mod deps;
    pub mod export;
    pub mod fixtures;
    pub mod gc;
    pub mod index;
    pub mod ingest;
    pub mod lineage;
    pub mod sync;
    pub mod vault;
    pub mod verify;
}
mod verifier;

use clap::Parser;
use cli::{
    BenchCmd, Cli, Command, DepsCmd, FixturesCmd, GcCmd, IndexCmd, IngestCmd, LineageCmd,
    LineageOverlayCmd, SyncCmd, VaultCmd, VaultDbEncryptCmd, VaultEncryptCmd, VaultRecoveryCmd,
};
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
            VaultCmd::Init {
                vault_path,
                vault_slug,
            } => {
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
            VaultCmd::Verify { vault_path } => commands::vault::run_verify(&vault_path),
            VaultCmd::Unlock {
                vault_path,
                passphrase_env,
            } => commands::vault::run_unlock(&vault_path, &passphrase_env),
            VaultCmd::Lock { vault_path } => commands::vault::run_lock(&vault_path),
            VaultCmd::LockStatus { vault_path } => commands::vault::run_lock_status(&vault_path),
            VaultCmd::Encrypt { cmd } => match cmd {
                VaultEncryptCmd::Status { vault_path } => {
                    commands::vault::run_encrypt_status(&vault_path)
                }
                VaultEncryptCmd::Enable {
                    vault_path,
                    passphrase_env,
                } => commands::vault::run_encrypt_enable(&vault_path, &passphrase_env),
                VaultEncryptCmd::Migrate {
                    vault_path,
                    passphrase_env,
                    now_ms,
                } => commands::vault::run_encrypt_migrate(&vault_path, &passphrase_env, now_ms),
            },
            VaultCmd::DbEncrypt { cmd } => match cmd {
                VaultDbEncryptCmd::Status { vault_path } => {
                    commands::vault::run_db_encrypt_status(&vault_path)
                }
                VaultDbEncryptCmd::Enable {
                    vault_path,
                    passphrase_env,
                } => commands::vault::run_db_encrypt_enable(&vault_path, &passphrase_env),
                VaultDbEncryptCmd::Migrate {
                    vault_path,
                    passphrase_env,
                    now_ms,
                } => commands::vault::run_db_encrypt_migrate(&vault_path, &passphrase_env, now_ms),
            },
            VaultCmd::Recovery { cmd } => match cmd {
                VaultRecoveryCmd::Status { vault_path } => {
                    commands::vault::run_recovery_status(&vault_path)
                }
                VaultRecoveryCmd::Generate {
                    vault_path,
                    output,
                    passphrase_env,
                    now_ms,
                } => commands::vault::run_recovery_generate(
                    &vault_path,
                    &output,
                    &passphrase_env,
                    now_ms,
                ),
                VaultRecoveryCmd::Verify {
                    vault_path,
                    bundle,
                    phrase_env,
                } => commands::vault::run_recovery_verify(&vault_path, &bundle, &phrase_env),
            },
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
            zip,
        } => commands::export::run_export(&vault_path, &export_dir, zip, now_ms()).map(|bundle| {
            println!("exported bundle: {}", bundle.display());
        }),
        Command::Verify { bundle_path } => {
            commands::verify::run_verify(&bundle_path).map(|(code, report)| {
                println!(
                    "{}",
                    serde_json::to_string(&report).unwrap_or_else(|_| "{}".to_string())
                );
                if code != 0 {
                    std::process::exit(code as i32);
                }
            })
        }
        Command::Index { cmd } => match cmd {
            IndexCmd::Rebuild { vault_path } => commands::index::run_rebuild(&vault_path),
        },
        Command::Gc { cmd } => match cmd {
            GcCmd::Run { vault_path } => commands::gc::run_gc(&vault_path),
        },
        Command::Deps { cmd } => match cmd {
            DepsCmd::Check => commands::deps::run_check(),
        },
        Command::Bench { cmd } => match cmd {
            BenchCmd::Run { corpus } => commands::bench::run_bench(&corpus),
        },
        Command::Fixtures { cmd } => match cmd {
            FixturesCmd::Generate { corpus } => {
                commands::fixtures::generate_corpus(&corpus).map(|path| {
                    println!("generated fixtures at {}", path.display());
                })
            }
        },
        Command::Sync { cmd } => match cmd {
            SyncCmd::Status {
                vault_path,
                target_path,
            } => commands::sync::run_status(&vault_path, &target_path),
            SyncCmd::Push {
                vault_path,
                target_path,
                now_ms,
            } => commands::sync::run_push(&vault_path, &target_path, now_ms),
            SyncCmd::Pull {
                vault_path,
                target_path,
                auto_merge,
                now_ms,
            } => commands::sync::run_pull(&vault_path, &target_path, now_ms, auto_merge.as_deref()),
            SyncCmd::MergePreview {
                vault_path,
                target_path,
                now_ms,
            } => commands::sync::run_merge_preview(&vault_path, &target_path, now_ms),
        },
        Command::Lineage { cmd } => match cmd {
            LineageCmd::Overlay { cmd } => match cmd {
                LineageOverlayCmd::Add {
                    vault_path,
                    doc_id,
                    from_node_id,
                    to_node_id,
                    relation,
                    evidence,
                    created_by,
                    now_ms,
                } => commands::lineage::run_overlay_add(
                    &vault_path,
                    &doc_id,
                    &from_node_id,
                    &to_node_id,
                    &relation,
                    &evidence,
                    &created_by,
                    now_ms,
                ),
                LineageOverlayCmd::Remove {
                    vault_path,
                    overlay_id,
                } => commands::lineage::run_overlay_remove(&vault_path, &overlay_id),
                LineageOverlayCmd::List { vault_path, doc_id } => {
                    commands::lineage::run_overlay_list(&vault_path, &doc_id)
                }
            },
        },
    };

    if let Err(err) = result {
        eprintln!("{}: {}", err.code, err.message);
        std::process::exit(1);
    }
}
