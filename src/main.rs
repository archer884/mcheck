use std::{
    fs::{self, File},
    io::{self, BufReader},
    path::{Path, PathBuf},
};

use clap::Parser;
use hashbrown::HashMap;
use serde::Deserialize;

#[derive(Clone, Debug, Parser)]
struct Opts {
    manifest: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(transparent)]
struct Manifest {
    entries: HashMap<String, String>,
}

fn main() {
    let opts = Opts::parse();
    if let Err(e) = run(&opts) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn run(opts: &Opts) -> io::Result<()> {
    use owo_colors::OwoColorize;

    let target = target_dir(&opts.manifest)?;
    let manifest = read_manifest(&opts.manifest)?;

    for (name, hash) in manifest.entries {
        let actual = get_actual_hash(&target, &name)?;
        if hash != actual {
            eprintln!("{} {}", "mismatch".red(), name);
        }
    }

    Ok(())
}

fn get_actual_hash(base: &Path, name: &str) -> io::Result<String> {
    let name = Path::new(name);
    let path = base.join(name);

    let mut reader = File::open(path).map(BufReader::new)?;
    let mut hasher = blake3::Hasher::new();

    io::copy(&mut reader, &mut hasher)?;

    Ok(hasher.finalize().to_string())
}

fn read_manifest(path: &str) -> io::Result<Manifest> {
    let manifest = fs::read_to_string(path)?;
    let manifest = serde_json::from_str(&manifest)?;
    Ok(manifest)
}

fn target_dir(path: impl AsRef<Path>) -> io::Result<PathBuf> {
    path.as_ref()
        .parent()
        .map(|path| path.to_owned())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "apparently, your file is not in a directory",
            )
        })
        .or_else(|_| std::env::current_dir())
}
