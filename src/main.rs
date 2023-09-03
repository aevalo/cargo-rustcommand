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
    // Custom subcommands may use the CARGO environment variable to call back to Cargo.
    // Add it as an optional parameter, if it's usefull.
    let invoked_through_cargo = is_invoked_through_cargo()?;
    let command = Command::new("Cargo Subcommand")
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
            .value_parser(value_parser!(PathBuf))
            .global(true))
        .propagate_version(true)
        .subcommand_required(true)
        .disable_help_subcommand(true)
        .arg_required_else_help(true);
    let subcommands = [
        Command::new("add")
                .about("Adds files to myapp")
                .arg(Arg::new("name").value_name("NAME")),
        Command::new("environment")
                .about("Display environment information")
    ];
    if invoked_through_cargo {
        let subcommand = match env::args().nth(1) {
            Some(subcommand_name) => Command::new(subcommand_name)
                .hide(true)
                .subcommand_required(true)
                .subcommands(subcommands),
            None => {
                return Err("Failed to read subcommand name from arguments!");
            }
        };
        Ok(command.bin_name("cargo").subcommand(subcommand))
    } else {
        Ok(command.subcommands(subcommands))
    }
}

fn main() -> ExitCode {
    let invoked_through_cargo = match is_invoked_through_cargo() {
        Ok(is_invoked_through_cargo) => is_invoked_through_cargo,
        Err(error) => {
            println!("Failed to check invocation method: {}", error);
            return ExitCode::FAILURE;
        }
    };

    let command = match construct_cli() {
        Ok(cli) => cli,
        Err(error) => {
            println!("Failed to construct CLI: {}", error);
            return ExitCode::FAILURE;
        }
    };

    let matches = command.get_matches();

    let subcommands = if invoked_through_cargo {
        match matches.subcommand() {
            Some(("subcommand", sub_matches)) => sub_matches.subcommand(),
            _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`")
        }
    } else {
        matches.subcommand()
    };

    match subcommands {
        Some(("add", sub_matches)) => println!(
            "'myapp add' was used, name is: {:?}",
            sub_matches.get_one::<String>("name")
        ),
        Some(("environment", sub_matches)) => {
            println!("Cargo binary path: {:?}", sub_matches.get_one::<PathBuf>("cargo-bin"));
        },
        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`")
    };

    ExitCode::SUCCESS
}
