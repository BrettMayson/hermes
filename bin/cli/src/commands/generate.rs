use std::{io::Write, path::PathBuf, str::FromStr};

use clap::{ArgMatches, Command};
use hermes::{config::Config, repo::Repository};

#[must_use]
pub fn cli() -> Command {
    Command::new("generate").about("Generate a Repo in the current directory")
}

pub async fn execute(_matches: &ArgMatches) {
    let path = PathBuf::from("hermes.toml");
    if !path.exists() {
        eprintln!("err: No `hermes.toml` in the current directory");
        return;
    }
    let config =
        Config::from_str(&std::fs::read_to_string(path).expect("Failed to read `hermes.toml`"))
            .unwrap();
    println!("Generating {}", config.unit().name());

    // Cleanup .hermes
    let _ = std::fs::remove_dir_all(".hermes");
    std::fs::create_dir(".hermes").unwrap();

    let repo = Repository::from_config(config).unwrap();
    let mut out = std::fs::File::create("hermes.mpk").unwrap();
    out.write_all(&repo.to_blob()).unwrap();
    println!("`hermes.mpk` Created!")
}
