#![doc = "Local operator CLI for STE."]

use std::process::ExitCode;

use ste_cli::StorageCommand;

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
    eprintln!("unsupported command");
    ExitCode::from(2)
}
