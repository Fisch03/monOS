use clap::Parser;
use std::{fs, path::PathBuf};
use monoscript_emu::run_script;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    script_path: PathBuf,
}

fn main() {
    let args = Args::parse();

    let script = fs::read_to_string(args.script_path).expect("failed to read script");
    run_script(&script).unwrap();
}
