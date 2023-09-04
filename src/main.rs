use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use clap::{Arg, Command, crate_version, value_parser};

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

fn get_subcommand_name() -> Result<String, &'static str> {
    let invoked_through_cargo = is_invoked_through_cargo()?;
    if invoked_through_cargo {
        match env::args().nth(1) {
            Some(subcommand_arg) => Ok(subcommand_arg),
            None => Err("Failed to read subcommand name from arguments!")
        }
    } else {
        Err("Not invoked through Cargo")
    }
}

fn construct_cli() -> Result<Command, &'static str> {
    // Custom subcommands may use the CARGO environment variable to call back to Cargo.
    // Add it as an optional parameter, if it's usefull.
    let invoked_through_cargo = is_invoked_through_cargo()?;
    let command = Command::new("Cargo Subcommand")
        .author("Arttu Valo, arttu.valo@gmail.com")
        .version(crate_version!())
        .about("This is a template for Cargo subcommands")
        .after_help("Longer explanation to appear after the options when \
                     displaying the help information from --help or -h")
        .propagate_version(false)
        .disable_version_flag(true)
        .disable_help_subcommand(false)
        .disable_help_flag(true)
        .subcommand_required(true)
        .arg_required_else_help(true);
    let args = [
        Arg::new("version")
            .short('V')
            .long("version")
            .action(clap::ArgAction::Version),
        Arg::new("help")
            .short('h')
            .long("help")
            .action(clap::ArgAction::Help),
        Arg::new("cargo-bin")
            .long("cargo-bin")
            .help("Cargo binary used to invoke this subcommand")
            .env("CARGO")
            .value_name("Cargo binary path")
            .required(false)
            .value_parser(value_parser!(PathBuf))
            .global(true)
    ];
    let subcommands = [
        Command::new("add")
                .about("Adds files to myapp")
                .arg(Arg::new("name").value_name("NAME")),
        Command::new("environment")
                .about("Display environment information")
    ];
    if invoked_through_cargo {
        let subcommand_name = get_subcommand_name()?;
        let subcommand = Command::new(subcommand_name)
            .display_name("Cargo Subcommand")
            .version(crate_version!())
            .hide(true)
            .subcommand_required(true)
            .arg_required_else_help(true)
            .subcommands(subcommands)
            .args(args);
        Ok(command.bin_name("cargo").subcommand(subcommand))
    } else {
        Ok(command.args(args).subcommands(subcommands).subcommand_required(true))
    }
}

fn main() -> ExitCode {
    let cmd = match construct_cli() {
        Ok(cmd) => cmd,
        Err(error) => {
            println!("Failed to construct CLI: {}", error);
            return ExitCode::FAILURE;
        }
    };
    let matches = cmd.get_matches();
    let matches = match matches.subcommand() {
        Some(("subcommand", matches)) => matches,
        _ => unreachable!("clap should ensure we don't get here"),
    };
    let cargo_bin = matches.get_one::<PathBuf>("cargo-bin");
    println!("{cargo_bin:?}");
    ExitCode::SUCCESS
}
