//! `systemprompt-kit-export` — turns a marketplace declared in `services/`
//! into a kit repository tree: Anthropic marketplace format plus the two
//! systemprompt sidecars, and **no** `access:` block.
//!
//! This is how a kit is seeded from this repository so that every kit has the
//! same shape by construction (`deploy/kit/` is the template it drops into).
//! The tree is proven before it is reported: the exporter re-imports it with
//! `import_anthropic_tree(strict)` into a scratch directory and diffs the
//! resulting marketplace, plugin and skill configs against `services/`.
//! Composition forbids an id both local and bundled, so once the kit is
//! published and pinned the exported ids are removed from `services/`.

use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::Parser;
use systemprompt::loader::ConfigLoader;
use systemprompt::logging::CliService;
use systemprompt_kit_export::{export_kit, round_trip};

#[derive(Debug, Parser)]
#[command(
    name = "systemprompt-kit-export",
    about = "Export a marketplace from services/ as a kit tree in Anthropic marketplace format"
)]
struct Cli {
    #[arg(help = "Marketplace id, e.g. enterprise-demo")]
    marketplace: String,
    #[arg(help = "Directory to write the kit tree into (created; must be empty)")]
    out: PathBuf,
    #[arg(
        long,
        default_value = "services",
        help = "The services tree to export from"
    )]
    services: PathBuf,
    #[arg(long, help = "Write the tree but skip the re-import proof")]
    no_verify: bool,
    #[arg(
        long,
        hide = true,
        help = "Accepted for `plugins run` parity; the output is the report either way"
    )]
    json: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config_path = cli.services.join("config").join("config.yaml");
    let services = ConfigLoader::load_from_path(&config_path)
        .with_context(|| format!("loading {}", config_path.display()))?;

    if cli.out.exists() && std::fs::read_dir(&cli.out)?.next().is_some() {
        bail!("{} exists and is not empty", cli.out.display());
    }
    std::fs::create_dir_all(&cli.out)?;

    let report = export_kit(&services, &cli.services, &cli.marketplace, &cli.out)?;
    CliService::output(&report.summary());

    if cli.no_verify {
        CliService::info("verify: skipped (--no-verify)");
        return Ok(());
    }
    let diffs = round_trip(&cli.out, &cli.services, &report)?;
    if diffs.is_empty() {
        CliService::output(&format!(
            "verify: re-import reproduces services/ for {} plugin(s), {} skill(s)",
            report.plugins.len(),
            report.skills.len()
        ));
        return Ok(());
    }
    for d in &diffs {
        CliService::error(&format!("verify: {d}"));
    }
    bail!("{} difference(s) after re-import — see above", diffs.len());
}
