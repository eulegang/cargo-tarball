use git2::{ObjectType, Repository, TreeWalkMode, TreeWalkResult};
use std::path::PathBuf;
use tar::{Builder, Header};

use clap::Parser;
use flate2::{write::GzEncoder, Compression};
use serde::Deserialize;

#[derive(Parser)]
pub struct Args {
    #[clap(
        short,
        long,
        default_value = ".",
        env = "CARGO_TARBALL_OUT",
        value_name = "DIR"
    )]
    output: PathBuf,

    #[clap(short, long)]
    verbose: bool,
}

#[derive(Deserialize)]
pub struct Manifest {
    package: Package,
}

#[derive(Deserialize)]
pub struct Package {
    name: String,
    version: String,
}

fn main() -> eyre::Result<()> {
    let args = Args::parse();
    let repo = Repository::discover(".")?;

    let name = find_name(&args)?;

    let mut output = args.output;
    output.push(format!("{name}.tgz"));
    let file = std::fs::File::options()
        .create(true)
        .write(true)
        .truncate(true)
        .open(output)?;

    let gz_out = GzEncoder::new(file, Compression::default());
    let mut tar = Builder::new(gz_out);

    let tree = repo.head()?.peel_to_tree()?;

    let root = PathBuf::from(name);

    tree.walk(TreeWalkMode::PreOrder, |name, entry| {
        if entry.kind() == Some(ObjectType::Blob) {
            let mut path = root.clone();
            path.push(name);
            path.push(entry.name().unwrap());

            if args.verbose {
                println!("{}", path.display());
            }

            let blob = entry
                .to_object(&repo)
                .expect("failed to find object")
                .peel_to_blob()
                .expect("failed to peel to blob");

            let content = blob.content();
            let mut header = Header::new_gnu();
            header.set_path(path).unwrap();
            header.set_size(content.len() as u64);
            header.set_mode(entry.filemode() as u32);
            header.set_cksum();

            tar.append(&header, content).unwrap();
        }

        TreeWalkResult::Ok
    })?;

    tar.into_inner()?.finish()?;

    Ok(())
}

fn find_name(_: &Args) -> eyre::Result<String> {
    let content = std::fs::read_to_string("Cargo.toml")?;

    let manifest: Manifest = toml::de::from_str(&content)?;

    Ok(format!(
        "{}-{}",
        manifest.package.name, manifest.package.version
    ))
}
