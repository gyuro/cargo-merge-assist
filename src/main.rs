mod merge;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand};

use crate::merge::merge_manifest_texts;

#[derive(Parser, Debug)]
#[command(name = "cargo-merge-assist")]
#[command(version)]
#[command(about = "Semantic merge assistant for Cargo.toml and Cargo.lock")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 3-way semantic merge for Cargo.toml
    MergeManifest(MergeManifestArgs),
    /// Regenerate Cargo.lock from Cargo.toml
    ResolveLock(ResolveLockArgs),
    /// Merge manifest + regenerate lockfile + optional cargo check
    MergeAll(MergeAllArgs),
    /// Install local Git merge drivers and .gitattributes entries
    InstallGitDriver(InstallGitDriverArgs),
}

#[derive(Args, Debug)]
struct MergeManifestArgs {
    /// Base (ancestor) Cargo.toml path (%O in Git merge driver)
    #[arg(long)]
    base: PathBuf,
    /// Ours/current Cargo.toml path (%A in Git merge driver)
    #[arg(long)]
    ours: PathBuf,
    /// Theirs/incoming Cargo.toml path (%B in Git merge driver)
    #[arg(long)]
    theirs: PathBuf,
    /// Output path (usually same as --ours)
    #[arg(long)]
    out: PathBuf,
}

#[derive(Args, Debug)]
struct ResolveLockArgs {
    /// Repository root containing Cargo.toml
    #[arg(long, default_value = ".")]
    repo: PathBuf,
    /// Also run `cargo check -q` after lockfile regeneration
    #[arg(long)]
    verify: bool,
    /// Run cargo commands with --offline
    #[arg(long)]
    offline: bool,
}

#[derive(Args, Debug)]
struct MergeAllArgs {
    #[arg(long)]
    base: PathBuf,
    #[arg(long)]
    ours: PathBuf,
    #[arg(long)]
    theirs: PathBuf,
    #[arg(long)]
    out: PathBuf,
    /// Repository root containing Cargo.toml
    #[arg(long, default_value = ".")]
    repo: PathBuf,
    /// Skip cargo check verification
    #[arg(long)]
    skip_verify: bool,
    /// Run cargo commands with --offline
    #[arg(long)]
    offline: bool,
}

#[derive(Args, Debug)]
struct InstallGitDriverArgs {
    /// Repository root where merge driver config should be installed
    #[arg(long, default_value = ".")]
    repo: PathBuf,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::MergeManifest(args) => merge_manifest_cmd(args),
        Commands::ResolveLock(args) => resolve_lock_cmd(args),
        Commands::MergeAll(args) => merge_all_cmd(args),
        Commands::InstallGitDriver(args) => install_git_driver_cmd(args),
    }
}

fn merge_manifest_cmd(args: MergeManifestArgs) -> Result<()> {
    let base_text = read_utf8(&args.base)?;
    let ours_text = read_utf8(&args.ours)?;
    let theirs_text = read_utf8(&args.theirs)?;

    let merged = merge_manifest_texts(&base_text, &ours_text, &theirs_text)
        .map_err(|err| anyhow::anyhow!(err.to_string()))?;

    fs::write(&args.out, merged)
        .with_context(|| format!("failed writing merged manifest: {}", args.out.display()))?;

    Ok(())
}

fn resolve_lock_cmd(args: ResolveLockArgs) -> Result<()> {
    ensure_manifest_exists(&args.repo)?;

    run_cargo(&args.repo, &["generate-lockfile"], args.offline)?;
    if args.verify {
        run_cargo(&args.repo, &["check", "-q"], args.offline)?;
    }

    Ok(())
}

fn merge_all_cmd(args: MergeAllArgs) -> Result<()> {
    merge_manifest_cmd(MergeManifestArgs {
        base: args.base,
        ours: args.ours,
        theirs: args.theirs,
        out: args.out,
    })?;

    resolve_lock_cmd(ResolveLockArgs {
        repo: args.repo,
        verify: !args.skip_verify,
        offline: args.offline,
    })?;

    Ok(())
}

fn install_git_driver_cmd(args: InstallGitDriverArgs) -> Result<()> {
    ensure_manifest_exists(&args.repo)?;

    let gitattributes_path = args.repo.join(".gitattributes");
    append_unique_line(
        &gitattributes_path,
        "Cargo.toml merge=cargo-merge-assist-manifest",
    )?;
    append_unique_line(
        &gitattributes_path,
        "Cargo.lock merge=cargo-merge-assist-lock",
    )?;

    git_config(
        &args.repo,
        "merge.cargo-merge-assist-manifest.name",
        "cargo-merge-assist semantic merge for Cargo.toml",
    )?;
    git_config(
        &args.repo,
        "merge.cargo-merge-assist-manifest.driver",
        "cargo-merge-assist merge-manifest --base %O --ours %A --theirs %B --out %A",
    )?;
    git_config(
        &args.repo,
        "merge.cargo-merge-assist-lock.name",
        "cargo-merge-assist lockfile regeneration driver",
    )?;
    git_config(
        &args.repo,
        "merge.cargo-merge-assist-lock.driver",
        "cargo-merge-assist resolve-lock --repo .",
    )?;

    println!("Installed merge driver into {}", args.repo.display());
    println!("Added/updated {}", gitattributes_path.display());

    Ok(())
}

fn read_utf8(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("failed reading {}", path.display()))
}

fn ensure_manifest_exists(repo: &Path) -> Result<()> {
    let manifest = repo.join("Cargo.toml");
    if !manifest.exists() {
        bail!(
            "{} does not contain Cargo.toml; pass --repo with a Rust project root",
            repo.display()
        );
    }
    Ok(())
}

fn run_cargo(repo: &Path, args: &[&str], offline: bool) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(repo);
    cmd.args(args);
    if offline {
        cmd.arg("--offline");
    }

    let status = cmd
        .status()
        .with_context(|| format!("failed to execute cargo in {}", repo.display()))?;

    if !status.success() {
        bail!(
            "cargo command failed in {}: cargo {}",
            repo.display(),
            args.join(" ")
        );
    }

    Ok(())
}

fn git_config(repo: &Path, key: &str, value: &str) -> Result<()> {
    let status = Command::new("git")
        .arg("-C")
        .arg(repo)
        .arg("config")
        .arg("--local")
        .arg(key)
        .arg(value)
        .status()
        .with_context(|| format!("failed to run git config in {}", repo.display()))?;

    if !status.success() {
        bail!("git config failed for key `{key}`");
    }

    Ok(())
}

fn append_unique_line(path: &Path, line: &str) -> Result<()> {
    let mut existing = if path.exists() {
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?
    } else {
        String::new()
    };

    if existing.lines().any(|l| l.trim() == line) {
        return Ok(());
    }

    if !existing.is_empty() && !existing.ends_with('\n') {
        existing.push('\n');
    }
    existing.push_str(line);
    existing.push('\n');

    fs::write(path, existing).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}
