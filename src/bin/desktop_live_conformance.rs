use std::path::PathBuf;

use clap::Parser;
use gyrognome::desktop_live_conformance::{DesktopLiveExperimentOptions, run};

#[derive(Debug, Parser)]
#[command(
    name = "desktop-live-conformance",
    about = "Feature-gated disposable classic desktop conformance experiment"
)]
struct Args {
    #[arg(long)]
    confirm_disposable: bool,
    #[arg(long)]
    confirm_live_submission: bool,
    #[arg(long)]
    confirm_client_stopped: bool,
    #[arg(long)]
    confirm_cleanup: bool,
    #[arg(long)]
    validate_only: bool,
    #[arg(long)]
    immediate_only: bool,
    #[arg(long)]
    experiment_dir: PathBuf,
    #[arg(long)]
    evidence: PathBuf,
    #[arg(long, default_value = "Gyrognome official control")]
    control_motto: String,
    #[arg(long, default_value = "Gyrognome disposable test")]
    motto: String,
    #[arg(long)]
    guild_designation: String,
    #[arg(long, default_value_t = 28_800)]
    max_active_seconds: u64,
    #[arg(long, default_value_t = 60)]
    classification_poll_seconds: u64,
}

fn main() {
    let args = Args::parse();
    let options = DesktopLiveExperimentOptions {
        confirm_disposable: args.confirm_disposable,
        confirm_live_submission: args.confirm_live_submission,
        confirm_client_stopped: args.confirm_client_stopped,
        confirm_cleanup: args.confirm_cleanup,
        validate_only: args.validate_only,
        immediate_only: args.immediate_only,
        experiment_dir: args.experiment_dir,
        evidence_path: args.evidence,
        control_motto: args.control_motto,
        motto: args.motto,
        guild_designation: args.guild_designation,
        max_active_seconds: args.max_active_seconds,
        classification_poll_seconds: args.classification_poll_seconds,
    };
    if let Err(error) = run(&options) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
