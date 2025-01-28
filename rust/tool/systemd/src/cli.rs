use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use crate::install;
use lanzaboote_tool::{architecture::Architecture, signature::local::LocalKeyPair};
use lanzaboote_tool::generation::Generation;

/// The default log level.
///
/// 2 corresponds to the level INFO.
const DEFAULT_LOG_LEVEL: usize = 2;

#[derive(Parser)]
pub struct Cli {
    /// Silence all output
    #[arg(short, long)]
    quiet: bool,
    /// Verbose mode (-v, -vv, etc.)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    #[clap(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Build(BuildCommand),
    Install(InstallCommand),
}

#[derive(Parser)]
struct BuildCommand {
    /// System for lanzaboote binaries, e.g. defines the EFI fallback path
    #[arg(long)]
    system: String,

    /// Systemd path
    #[arg(long)]
    systemd: Option<PathBuf>,

    /// Systemd-boot loader config
    #[arg(long)]
    systemd_boot_loader_config: Option<PathBuf>,

    /// sbsign Public Key
    #[arg(long)]
    public_key: Option<PathBuf>,

    /// sbsign Private Key
    #[arg(long)]
    private_key: Option<PathBuf>,

    // /// Override initrd
    // #[arg(long)]
    // initrd: PathBuf,

    /// EFI system partition mountpoint (e.g. efiSysMountPoint)
    esp: PathBuf,

    /// Generation
    generation: PathBuf,
}

#[derive(Parser)]
struct InstallCommand {
    /// System for lanzaboote binaries, e.g. defines the EFI fallback path
    #[arg(long)]
    system: String,

    /// Systemd path
    #[arg(long)]
    systemd: PathBuf,

    /// Systemd-boot loader config
    #[arg(long)]
    systemd_boot_loader_config: PathBuf,

    /// Systemd-pcrlock directory
    #[arg(long)]
    systemd_pcrlock: PathBuf,

    /// sbsign Public Key
    #[arg(long)]
    public_key: Option<PathBuf>,

    /// sbsign Private Key
    #[arg(long)]
    private_key: Option<PathBuf>,

    /// Configuration limit
    #[arg(long, default_value_t = 1)]
    configuration_limit: usize,

    /// EFI system partition mountpoint (e.g. efiSysMountPoint)
    esp: PathBuf,

    /// List of generation links (e.g. /nix/var/nix/profiles/system-*-link)
    generations: Vec<PathBuf>,
}

impl Cli {
    pub fn call(self, module: &str) {
        stderrlog::new()
            .module(module)
            .show_level(false)
            .quiet(self.quiet)
            .verbosity(DEFAULT_LOG_LEVEL + usize::from(self.verbose))
            .init()
            .expect("Failed to setup logger.");

        if let Err(e) = self.commands.call() {
            log::error!("{e:#}");
            std::process::exit(1);
        };
    }
}

impl Commands {
    pub fn call(self) -> Result<()> {
        match self {
            Commands::Build(args) => build(args),
            Commands::Install(args) => install(args),
        }
    }
}

fn build(args: BuildCommand) -> Result<()> {
    let lanzaboote_stub =
        std::env::var("LANZABOOTE_STUB").context("Failed to read LANZABOOTE_STUB env variable")?;

    let local_signer = LocalKeyPair::new(
        &args.public_key.expect("Failed to obtain public key"),
        &args.private_key.expect("Failed to obtain private key"),
    );

    let generation = Generation::from_toplevel(&args.generation, 1)
        .with_context(|| format!("Failed to build generation from link: {0:?}", args.generation))?;

    let mut installer = install::Installer::new(
        PathBuf::from(lanzaboote_stub),
        Architecture::from_nixos_system(&args.system)?,
        args.systemd.clone().unwrap_or(PathBuf::from("")),
        args.systemd_boot_loader_config.unwrap_or(PathBuf::from("")),
        PathBuf::from(""), // args.systemd_pcrlock,
        local_signer,
        1, // args.configuration_limit,
        args.esp,
        vec![],
    );

    installer.build_generation(&generation)?;

    if args.systemd.is_some() {
        // TODO: only if systemd is set.
        installer.install_systemd_boot()?;
    }

    Ok(())
}

fn install(args: InstallCommand) -> Result<()> {
    let lanzaboote_stub =
        std::env::var("LANZABOOTE_STUB").context("Failed to read LANZABOOTE_STUB env variable")?;

    let local_signer = LocalKeyPair::new(
        &args.public_key.expect("Failed to obtain public key"),
        &args.private_key.expect("Failed to obtain private key"),
    );

    install::Installer::new(
        PathBuf::from(lanzaboote_stub),
        Architecture::from_nixos_system(&args.system)?,
        args.systemd,
        args.systemd_boot_loader_config,
        args.systemd_pcrlock,
        local_signer,
        args.configuration_limit,
        args.esp,
        args.generations,
    )
    .install()
}
