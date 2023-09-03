use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use clap::{Arg, Command, value_parser};

fn is_invoked_through_cargo() -> Result<bool, &'static str> {
    // Check, if subcommand was invoked through Cargo. When it is, first argument will
    // be the filename of the custom subcommand, as usual. The second argument will be
    // the subcommand name itself. For example, the second argument would be ${command}
    // when invoking cargo-${command}. Any additional arguments on the command line will
    // be forwarded unchanged.
    if env::args().count() < 2 {
        return Ok(false);
    }
    let subcommand_filename = match env::args().nth(0) {
        Some(arg) => arg,
        None => return Err("Failed to read subcommand filename from arguments!")
    };
    let subcommand_file_stem = match Path::new(&subcommand_filename).file_stem() {
        Some(file_stem) => match file_stem.to_str() {
            Some(file_stem_str) => file_stem_str,
            None => return Err("Failed to convert platform string to UTF-8!")
        },
        None => return Err("Failed to read subcommand filename!")
    };
    let subcommand_arg = match env::args().nth(1) {
        Some(arg) => format!("cargo-{}", arg),
        None => return Err("Failed to read subcommand name from arguments!")
    };
    Ok(subcommand_file_stem == subcommand_arg)
}

fn construct_cli() -> Result<Command, &'static str> {
    let is_invoked_through_cargo = is_invoked_through_cargo()?;
    // Custom subcommands may use the CARGO environment variable to call back to Cargo.
    // Add it as an optional parameter, if it's usefull.
    let mut m = Command::new("Cargo Subcommand")
        .author("Arttu Valo, arttu.valo@gmail.com")
        .version("0.1.0")
        .about("This is a template for Cargo subcommands")
        .after_help("Longer explanation to appear after the options when \
                     displaying the help information from --help or -h")
        .arg(Arg::new("cargo-bin")
            .long("cargo-bin")
            .help("Cargo binary used to invoke this subcommand")
            .env("CARGO")
            .value_name("Cargo binary path")
            .required(false)
            .value_parser(value_parser!(PathBuf)));
    // Add subcommand as optional hidden parameter, if invoked from Cargo
    if is_invoked_through_cargo {
        m = match env::args().nth(1) {
            Some(subcommand) => m.arg(Arg::new(subcommand).hide(true).required(false)),
            None => return Err("Failed to read subcommand name from arguments!")
        };
    }
    Ok(m)
}

fn main() -> ExitCode {
    let m = match construct_cli() {
        Ok(command) => command.get_matches(),
        Err(error) => {
            println!("Failed to construct CLI: {}", error);
            return ExitCode::FAILURE;
        }
    };
    match m.get_one::<PathBuf>("cargo-bin") {
        Some(cargo_bin) => println!("Using Cargo from path {:?}", cargo_bin),
        None => println!("Cargo binary not specified")
    };

    ExitCode::SUCCESS
}
