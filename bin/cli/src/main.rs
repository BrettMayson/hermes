mod commands;

use clap::{ArgAction, Command};

#[must_use]
pub fn cli() -> Command {
    #[allow(unused_mut)]
    let mut global = Command::new(env!("CARGO_PKG_NAME"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand_required(false)
        .arg_required_else_help(true)
        .subcommand(commands::generate::cli());
    // global = global.arg(
    //     clap::Arg::new("threads")
    //         .global(true)
    //         .help("Number of threads, defaults to # of CPUs")
    //         .action(ArgAction::Set)
    //         .long("threads")
    //         .short('t'),
    // );
    // global = global.arg(
    //     clap::Arg::new("verbosity")
    //         .global(true)
    //         .help("Verbosity level")
    //         .action(ArgAction::Count)
    //         .short('v'),
    // );
    global = global.arg(
        clap::Arg::new("version")
            .global(false)
            .help("Print version")
            .action(ArgAction::SetTrue)
            .long("version"),
    );
    global
}

fn main() {
    let matches = cli().get_matches();

    rayon::ThreadPoolBuilder::new()
        // .num_threads(threads.parse::<usize>().unwrap())
        .num_threads(4)
        .build_global()
        .unwrap();

    match matches.subcommand() {
        Some(("generate", matches)) => commands::generate::execute(matches),
        _ => unreachable!(),
    }
}
