use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "kc_cli")]
#[command(about = "KnowledgeCore CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Vault {
        #[command(subcommand)]
        cmd: VaultCmd,
    },
    Ingest {
        #[command(subcommand)]
        cmd: IngestCmd,
    },
}

#[derive(Subcommand)]
pub enum VaultCmd {
    Init { vault_path: String, vault_slug: String },
    Open { vault_path: String },
}

#[derive(Subcommand)]
pub enum IngestCmd {
    ScanFolder {
        vault_path: String,
        scan_root: String,
        source_kind: String,
    },
    InboxOnce {
        vault_path: String,
        file_path: String,
        source_kind: String,
    },
}
