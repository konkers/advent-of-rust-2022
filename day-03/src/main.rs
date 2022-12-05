use std::{fs, path::PathBuf};

use anyhow::{anyhow, Result};
use clap::Parser;

mod part1;
mod part2;

fn item_priority(item: char) -> Result<u32> {
    if item.is_ascii_lowercase() {
        Ok(item as u32 - 'a' as u32 + 1)
    } else if item.is_ascii_uppercase() {
        Ok(item as u32 - 'A' as u32 + 27)
    } else {
        Err(anyhow!("'{}' is not an alphabetic character", item))
    }
}

// Command line arguments.
#[derive(Debug, Parser)]
struct Args {
    input: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let input = fs::read_to_string(&args.input)?;

    let total = part1::solution(&input)?;
    println!("[Part 1] Sum of shared item priorities: {}", total);

    let total = part2::solution(&input)?;
    println!("[Part 2] Sum group priorities: {}", total);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_priority() {
        // Check valid ranges.
        assert_eq!(item_priority('a').unwrap(), 1);
        assert_eq!(item_priority('z').unwrap(), 26);
        assert_eq!(item_priority('A').unwrap(), 27);
        assert_eq!(item_priority('Z').unwrap(), 52);

        // Check edges of valid ranges.
        assert!(item_priority('`').is_err()); // Comes before 'a'.
        assert!(item_priority('{').is_err()); // Comes after 'z'.
        assert!(item_priority('@').is_err()); // Comes before 'A'.
        assert!(item_priority('[').is_err()); // Comes after 'Z'.

        // Non alphabetic characters are not valid.
        assert!(item_priority('0').is_err());

        // Non ascii characters are not valid.
        assert!(item_priority('🎄').is_err());
    }
}
