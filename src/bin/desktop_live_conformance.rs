use std::path::PathBuf;

use clap::Parser;
use gyrognome::desktop_eligibility::DesktopOnlineOperation;
use gyrognome::desktop_live_conformance::{DesktopLiveExperimentOptions, DesktopLiveStage, run};

#[derive(Debug, Parser)]
#[command(
    name = "desktop-live-conformance",
    about = "Feature-gated disposable classic desktop conformance experiment"
)]
struct Args {
    #[arg(long, default_value = "Spoltog")]
    realm: String,
    #[arg(long, value_enum, default_value = "legacy")]
    stage: DesktopLiveStage,
    #[arg(long, value_enum, value_delimiter = ',')]
    operations: Vec<DesktopOnlineOperation>,
    #[arg(long)]
    max_mutation_attempts: Option<usize>,
    #[arg(long)]
    allow_existing_disposable: bool,
    #[arg(long)]
    initial_guild_leave: bool,
    #[arg(long)]
    preparation_reconciliation_seconds: Option<u64>,
    #[arg(long)]
    confirm_preparation_reconciliation: bool,
    #[arg(long)]
    operator_join_reconciliation_seconds: Option<u64>,
    #[arg(long)]
    confirm_operator_accepted_join: bool,
    #[arg(long)]
    confirm_no_competing_join: bool,
    #[arg(long, default_value = "Gyrognome manual consumption marker")]
    manual_motto: String,
    #[arg(long)]
    guild_change_designation: Option<String>,
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
    #[arg(long, default_value = "")]
    guild_designation: String,
    #[arg(long, default_value_t = 28_800)]
    max_active_seconds: u64,
    #[arg(long, default_value_t = 60)]
    classification_poll_seconds: u64,
}

fn main() {
    let args = Args::parse();
    let options = DesktopLiveExperimentOptions {
        realm: args.realm,
        stage: args.stage,
        operations: args.operations,
        max_mutation_attempts: args.max_mutation_attempts,
        allow_existing_disposable: args.allow_existing_disposable,
        initial_guild_leave: args.initial_guild_leave,
        preparation_reconciliation_seconds: args.preparation_reconciliation_seconds,
        confirm_preparation_reconciliation: args.confirm_preparation_reconciliation,
        operator_join_reconciliation_seconds: args.operator_join_reconciliation_seconds,
        confirm_operator_accepted_join: args.confirm_operator_accepted_join,
        confirm_no_competing_join: args.confirm_no_competing_join,
        manual_motto: args.manual_motto,
        guild_change_designation: args.guild_change_designation,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_explicit_pemptus_stages_and_rejects_unknown_operations() {
        let arguments = [
            "desktop-live-conformance",
            "--realm",
            "Pemptus",
            "--stage",
            "immediate",
            "--operations",
            "manual-brag,motto,guild",
            "--max-mutation-attempts",
            "7",
            "--experiment-dir",
            "synthetic-private-directory",
            "--evidence",
            "synthetic-evidence.json",
        ];
        let args = Args::try_parse_from(arguments).unwrap();
        assert_eq!(args.realm, "Pemptus");
        assert_eq!(args.stage, DesktopLiveStage::Immediate);
        assert_eq!(
            args.operations,
            [
                DesktopOnlineOperation::ManualBrag,
                DesktopOnlineOperation::Motto,
                DesktopOnlineOperation::Guild,
            ]
        );
        assert_eq!(args.max_mutation_attempts, Some(7));
        assert!(!args.confirm_live_submission);
        let existing = Args::try_parse_from(arguments.into_iter().chain([
            "--allow-existing-disposable",
            "--initial-guild-leave",
            "--control-motto",
            "",
        ]))
        .unwrap();
        assert!(existing.allow_existing_disposable && existing.initial_guild_leave);
        assert!(existing.control_motto.is_empty());
        let recovery = Args::try_parse_from(arguments.into_iter().chain([
            "--preparation-reconciliation-seconds",
            "60",
            "--confirm-preparation-reconciliation",
        ]))
        .unwrap();
        assert_eq!(recovery.preparation_reconciliation_seconds, Some(60));
        assert!(recovery.confirm_preparation_reconciliation);
        let operator = Args::try_parse_from(arguments.into_iter().chain([
            "--operator-join-reconciliation-seconds",
            "60",
            "--confirm-operator-accepted-join",
            "--confirm-no-competing-join",
        ]))
        .unwrap();
        assert_eq!(operator.operator_join_reconciliation_seconds, Some(60));
        assert!(operator.confirm_operator_accepted_join && operator.confirm_no_competing_join);
        assert!(
            Args::try_parse_from(arguments.into_iter().chain(["--operations", "unsupported"]))
                .is_err()
        );
    }
}
