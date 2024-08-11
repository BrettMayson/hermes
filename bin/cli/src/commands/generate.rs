use std::{io::Write, path::PathBuf, str::FromStr};

use clap::{ArgMatches, Command};
use syncra::{config::Config, repo::Repository};

#[must_use]
pub fn cli() -> Command {
    Command::new("generate").about("Generate a Repo in the current directory")
}

pub async fn execute(_matches: &ArgMatches) {
    let path = PathBuf::from("syncra.toml");
    if !path.exists() {
        eprintln!("err: No `syncra.toml` in the current directory");
        return;
    }
    let config =
        Config::from_str(&std::fs::read_to_string(path).expect("Failed to read `syncra.yoml`"))
            .unwrap();
    println!("Generating {}", config.unit().name());

    // Cleanup .syncra
    let _ = std::fs::remove_dir_all(".syncra");
    std::fs::create_dir(".syncra").unwrap();

    let repo = Repository::from_config(config).unwrap();
    let mut out = std::fs::File::create("syncra.mpk").unwrap();
    out.write_all(&repo.to_blob()).unwrap();
    println!("`syncra.mpk` Created!")
}
