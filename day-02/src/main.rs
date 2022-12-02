use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::Parser;

mod part1;
mod part2;

// Command line arguments.
#[derive(Debug, Parser)]
struct Args {
    input: PathBuf,
}
fn main() -> Result<()> {
    let args = Args::parse();
    let input = fs::read_to_string(&args.input)?;

    let guide_1 = part1::parse_strategy_guide(&input)?;
    let score_1 = part1::game_score(&guide_1);
    println!("[Part 1] Score: {}", score_1);

    let guide_2 = part2::parse_strategy_guide(&input)?;
    let score_2 = part2::game_score(&guide_2);
    println!("[Part 2] Score: {}", score_2);

    Ok(())
}
