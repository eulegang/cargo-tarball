use git2::{Repository, TreeWalkMode, TreeWalkResult};
use std::path::PathBuf;

use clap::Parser;

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

fn main() -> eyre::Result<()> {
    let args = Args::parse();
    let repo = Repository::discover(".")?;

    let tree = repo.head()?.peel_to_tree()?;

    tree.walk(TreeWalkMode::PreOrder, |_, entry| {
        dbg!(entry.name());

        TreeWalkResult::Ok
    });

    println!("Hello, world!");

    Ok(())
}
