#![doc = "Local operator CLI for STE."]

use std::process::ExitCode;

use ste_cli::{DiagnosticsCommand, ObservationReplayCommand, ReplayCommand, StorageCommand};

fn main() -> ExitCode {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    if arguments.is_empty() {
        println!("Somatic Topology Engine {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }
    if arguments.first().map(String::as_str) == Some("storage") {
        if StorageCommand::parse(&arguments[1..]).is_err() {
            eprintln!("invalid storage command arguments");
            return ExitCode::from(2);
        }
        // Direct process invocation has no authenticated local-IPC identity or
        // fresh policy decision. The daemon composition boundary executes the
        // parsed command only after supplying both.
        eprintln!("active authorization required");
        return ExitCode::from(77);
    }
    if arguments.first().map(String::as_str) == Some("replay") {
        if arguments.get(1).map(String::as_str) == Some("observation") {
            if ObservationReplayCommand::parse(&arguments[2..]).is_err() {
                eprintln!("invalid observation replay command arguments");
                return ExitCode::from(2);
            }
            eprintln!("active authorization required");
            return ExitCode::from(77);
        }
        if ReplayCommand::parse(&arguments[1..]).is_err() {
            eprintln!("invalid replay command arguments");
            return ExitCode::from(2);
        }
        eprintln!("active authorization required");
        return ExitCode::from(77);
    }
    if arguments.first().map(String::as_str) == Some("diagnostics") {
        if DiagnosticsCommand::parse(&arguments[1..]).is_err() {
            eprintln!("invalid diagnostics command arguments");
            return ExitCode::from(2);
        }
        eprintln!("active authorization required");
        return ExitCode::from(77);
    }
    eprintln!("unsupported command");
    ExitCode::from(2)
}
