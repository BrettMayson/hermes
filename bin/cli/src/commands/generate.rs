use std::{io::Write, path::PathBuf, str::FromStr};

use clap::{ArgMatches, Command};
use harmony::{config::Config, repo::Repository};

#[must_use]
pub fn cli() -> Command {
    Command::new("generate").about("Generate a Repo in the current directory")
}

pub async fn execute(_matches: &ArgMatches) {
    let path = PathBuf::from("harmony.toml");
    if !path.exists() {
        eprintln!("err: No `harmony.toml` in the current directory");
        return;
    }
    let config =
        Config::from_str(&std::fs::read_to_string(path).expect("Failed to read `harmony.yoml`"))
            .unwrap();
    println!("Generating {}", config.unit().name());

    // Cleanup .harmony
    let _ = std::fs::remove_dir_all(".harmony");
    std::fs::create_dir(".harmony").unwrap();

    let repo = Repository::from_config(config).unwrap();
    let mut out = std::fs::File::create("harmony.mpk").unwrap();
    out.write_all(&repo.to_blob()).unwrap();
    println!("`harmony.mpk` Created!")
}
