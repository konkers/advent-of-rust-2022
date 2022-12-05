use std::{collections::HashSet, fs, path::PathBuf};

use anyhow::{anyhow, Result};
use clap::Parser;

struct Rucksack {
    // Define rucksack as having multiple compartments in expectation that part 2 will need it.
    compartments: Vec<HashSet<char>>,
}

impl Rucksack {
    fn parse(input: &str) -> Self {
        assert!((input.len() % 2) == 0);
        // Assumes only two compartments.
        let (a, b) = input.split_at(input.len() / 2);

        Rucksack {
            compartments: vec![a.chars().collect(), b.chars().collect()],
        }
    }

    fn shared_items(&self) -> Vec<char> {
        assert!(self.compartments.len() == 2);

        self.compartments[0]
            .intersection(&self.compartments[1])
            .copied()
            .collect()
    }

    fn shared_item_priority(&self) -> Result<u32> {
        let items = self.shared_items();
        assert_eq!(items.len(), 1);
        item_priority(items[0])
    }
}

fn item_priority(item: char) -> Result<u32> {
    if item.is_ascii_lowercase() {
        Ok(item as u32 - 'a' as u32 + 1)
    } else if item.is_ascii_uppercase() {
        Ok(item as u32 - 'A' as u32 + 27)
    } else {
        Err(anyhow!("'{}' is not an alphabetic character", item))
    }
}

fn part1(input: &str) -> Result<u32> {
    input
        .lines()
        .map(|line| Rucksack::parse(line).shared_item_priority())
        .sum()
}

// Command line arguments.
#[derive(Debug, Parser)]
struct Args {
    input: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let input = fs::read_to_string(&args.input)?;

    let total = part1(&input)?;

    println!("[Part 1] Sum of shared item priorities: {}", total);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const EXAMPLE_INPUT: &str = include_str!("example-input.txt");

    #[test]
    fn parse_rucksack() {
        let input = "vJrwpWtwJgWrhcsFMMfFFhFp";
        let sack = Rucksack::parse(input);
        assert_eq!(sack.compartments.len(), 2);
        assert_eq!(
            sack.compartments[0],
            vec!['v', 'J', 'r', 'w', 'p', 'W', 't', 'w', 'J', 'g', 'W', 'r']
                .into_iter()
                .collect()
        );
        assert_eq!(
            sack.compartments[1],
            vec!['h', 'c', 's', 'F', 'M', 'M', 'f', 'F', 'F', 'h', 'F', 'p']
                .into_iter()
                .collect()
        );
    }

    #[test]
    fn shared_items() {
        let input = "vJrwpWtwJgWrhcsFMMfFFhFp";
        let sack = Rucksack::parse(input);
        assert_eq!(sack.shared_items(), vec!['p']);
    }

    #[test]
    fn shared_item_priority() {
        let input = "vJrwpWtwJgWrhcsFMMfFFhFp";
        let sack = Rucksack::parse(input);
        assert_eq!(sack.shared_item_priority().unwrap(), 16);
    }

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

    #[test]
    fn test_part1() {
        assert_eq!(part1(EXAMPLE_INPUT).unwrap(), 157);
    }
}
