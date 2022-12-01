use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use day_01_lib::{part1, part1_fancy, part2};

// Command line arguments.
#[derive(Debug, Parser)]
struct Args {
    input: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let input = fs::read_to_string(&args.input)?;

    // Compute the answer both ways and assert that they match.
    let calories = part1(&input)?;
    let calories_fancy = part1_fancy(&input)?;
    assert_eq!(calories, calories_fancy);

    println!("[Part 1] Most calories carried by an elf: {}", calories);

    let top_3_calories = part2(&input)?;
    println!(
        "[Part 2] Calories carried by top 3 elevs: {}",
        top_3_calories
    );

    Ok(())
}
