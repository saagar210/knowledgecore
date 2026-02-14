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
    Export {
        vault_path: String,
        export_dir: String,
        #[arg(long)]
        zip: bool,
    },
    Verify {
        bundle_path: String,
    },
    Index {
        #[command(subcommand)]
        cmd: IndexCmd,
    },
    Gc {
        #[command(subcommand)]
        cmd: GcCmd,
    },
    Deps {
        #[command(subcommand)]
        cmd: DepsCmd,
    },
    Bench {
        #[command(subcommand)]
        cmd: BenchCmd,
    },
    Fixtures {
        #[command(subcommand)]
        cmd: FixturesCmd,
    },
    Sync {
        #[command(subcommand)]
        cmd: SyncCmd,
    },
    Lineage {
        #[command(subcommand)]
        cmd: LineageCmd,
    },
    #[cfg(feature = "phase_l_preview")]
    Preview {
        #[command(subcommand)]
        cmd: PreviewCmd,
    },
}

#[derive(Subcommand)]
pub enum VaultCmd {
    Init { vault_path: String, vault_slug: String },
    Open { vault_path: String },
    Verify { vault_path: String },
    Unlock {
        vault_path: String,
        #[arg(long = "passphrase-env")]
        passphrase_env: String,
    },
    Lock { vault_path: String },
    LockStatus { vault_path: String },
    Encrypt {
        #[command(subcommand)]
        cmd: VaultEncryptCmd,
    },
    DbEncrypt {
        #[command(subcommand)]
        cmd: VaultDbEncryptCmd,
    },
}

#[derive(Subcommand)]
pub enum VaultEncryptCmd {
    Status { vault_path: String },
    Enable {
        vault_path: String,
        #[arg(long = "passphrase-env")]
        passphrase_env: String,
    },
    Migrate {
        vault_path: String,
        #[arg(long = "passphrase-env")]
        passphrase_env: String,
        #[arg(long = "now-ms")]
        now_ms: i64,
    },
}

#[derive(Subcommand)]
pub enum VaultDbEncryptCmd {
    Status { vault_path: String },
    Enable {
        vault_path: String,
        #[arg(long = "passphrase-env")]
        passphrase_env: String,
    },
    Migrate {
        vault_path: String,
        #[arg(long = "passphrase-env")]
        passphrase_env: String,
        #[arg(long = "now-ms")]
        now_ms: i64,
    },
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

#[derive(Subcommand)]
pub enum IndexCmd {
    Rebuild { vault_path: String },
}

#[derive(Subcommand)]
pub enum GcCmd {
    Run { vault_path: String },
}

#[derive(Subcommand)]
pub enum DepsCmd {
    Check,
}

#[derive(Subcommand)]
pub enum BenchCmd {
    Run {
        #[arg(long)]
        corpus: String,
    },
}

#[derive(Subcommand)]
pub enum FixturesCmd {
    Generate {
        #[arg(long)]
        corpus: String,
    },
}

#[derive(Subcommand)]
pub enum SyncCmd {
    Status {
        vault_path: String,
        target_path: String,
    },
    Push {
        vault_path: String,
        target_path: String,
        #[arg(long = "now-ms")]
        now_ms: i64,
    },
    Pull {
        vault_path: String,
        target_path: String,
        #[arg(long = "now-ms")]
        now_ms: i64,
    },
}

#[derive(Subcommand)]
pub enum LineageCmd {
    Overlay {
        #[command(subcommand)]
        cmd: LineageOverlayCmd,
    },
}

#[derive(Subcommand)]
pub enum LineageOverlayCmd {
    Add {
        vault_path: String,
        doc_id: String,
        from_node_id: String,
        to_node_id: String,
        relation: String,
        evidence: String,
        #[arg(long, default_value = "cli")]
        created_by: String,
        #[arg(long = "now-ms")]
        now_ms: i64,
    },
    Remove {
        vault_path: String,
        overlay_id: String,
    },
    List {
        vault_path: String,
        doc_id: String,
    },
}

#[cfg(feature = "phase_l_preview")]
#[derive(Subcommand)]
pub enum PreviewCmd {
    Status,
    Capability {
        #[arg(long)]
        name: String,
    },
}
