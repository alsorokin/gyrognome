use std::path::PathBuf;

use clap::Parser;
use gyrognome::pemptus_acceptance_probe::{ProbeOptions, run};

#[derive(Parser)]
#[command(about = "Development-only one-shot Pemptus manual-report diagnostic")]
struct Args {
    #[arg(long)]
    save: PathBuf,
    #[arg(long)]
    confirm_disposable: bool,
    #[arg(long)]
    confirm_client_stopped: bool,
    #[arg(long)]
    confirm_live_submission: bool,
    #[arg(long)]
    validate_only: bool,
}

fn main() {
    let args = Args::parse();
    let options = ProbeOptions {
        confirm_disposable: args.confirm_disposable,
        confirm_client_stopped: args.confirm_client_stopped,
        confirm_live_submission: args.confirm_live_submission,
        validate_only: args.validate_only,
    };
    match run(&args.save, &options) {
        Ok(result) => match serde_json::to_string_pretty(&result) {
            Ok(output) => println!("{output}"),
            Err(_) => {
                eprintln!("could not serialize safe probe result");
                std::process::exit(1);
            }
        },
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
