use std::path::PathBuf;
use std::{cmp, fs};

use anyhow::{anyhow, Result};
use clap::Parser;

// Command line arguments.
#[derive(Debug, Parser)]
struct Args {
    input: PathBuf,
}

// Parse challenge input into a Vec of Vecs.
//
// This implementation uses a straight forward imperative approach.
fn parse_part_1_input(text: &str) -> Result<Vec<Vec<i32>>> {
    let mut elves = Vec::new();
    let mut elf = Vec::new();
    for line in text.lines() {
        if line.is_empty() {
            elves.push(elf);
            elf = Vec::new();
        } else {
            let calories: i32 = line
                .parse()
                .map_err(|e| anyhow!("Error parsing '{}': {}", text, e))?;
            elf.push(calories);
        }
    }
    elves.push(elf);

    Ok(elves)
}

// Parse challenge input into a Vec of Vecs.
//
// This implementation uses a "fancier" more functional approach.
fn parse_part_1_input_fancy(text: &str) -> Result<Vec<Vec<i32>>> {
    text.lines()
        .try_fold(vec![vec![]], |mut elves, line| -> Result<Vec<Vec<i32>>> {
            if line.is_empty() {
                elves.push(Vec::new());
                Ok(elves)
            } else {
                let calories: i32 = line
                    .parse()
                    .map_err(|e| anyhow!("Error parsing '{}': {}", text, e))?;
                elves.last_mut().unwrap().push(calories);
                Ok(elves)
            }
        })
}

// Find the max calories of any elf.
//
// This implementation uses a straight forward imperative approach.
fn find_max_calories(elves: &Vec<Vec<i32>>) -> i32 {
    let mut max = i32::MIN;
    for elf in elves {
        let mut total_calories = 0;
        for calories in elf {
            total_calories += calories;
        }

        max = cmp::max(max, total_calories);
    }

    max
}

// Find the max calories of any elf.
//
// This implementation uses a "fancier" more functional approach.
fn find_max_calories_fancy(elves: &Vec<Vec<i32>>) -> i32 {
    elves
        .iter()
        .fold(i32::MIN, |max, elf| cmp::max(max, elf.iter().sum()))
}

// Compute the answer to part 1 using the imperative methods.
fn part1(input: &str) -> Result<i32> {
    let elves = parse_part_1_input(input).unwrap();
    Ok(find_max_calories(&elves))
}

// Compute the answer to part 1 using the fancy methods.
fn part1_fancy(input: &str) -> Result<i32> {
    let elves = parse_part_1_input_fancy(input).unwrap();
    Ok(find_max_calories_fancy(&elves))
}

fn main() -> Result<()> {
    let args = Args::parse();

    let input = fs::read_to_string(&args.input)?;

    // Compute the answer both ways and assert that they match.
    let calories = part1(&input)?;
    let calories_fancy = part1_fancy(&input)?;
    assert_eq!(calories, calories_fancy);

    println!("[Part 1] Most calories carried by an elf: {}", calories);
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_INPUT_1: &'static str = include_str!("example-input-1.txt");

    fn parsed_example_input_1() -> Vec<Vec<i32>> {
        vec![
            vec![1000, 2000, 3000],
            vec![4000],
            vec![5000, 6000],
            vec![7000, 8000, 9000],
            vec![10000],
        ]
    }

    #[test]
    fn test_parse_part_1_input() {
        assert_eq!(
            parse_part_1_input(EXAMPLE_INPUT_1).unwrap(),
            parsed_example_input_1()
        );
    }

    #[test]
    fn test_parse_part_1_input_fancy() {
        assert_eq!(
            parse_part_1_input_fancy(EXAMPLE_INPUT_1).unwrap(),
            parsed_example_input_1()
        );
    }

    #[test]
    fn test_find_max_calories() {
        let elves = parsed_example_input_1();
        assert_eq!(find_max_calories(&elves), 24000);
    }

    #[test]
    fn test_find_max_calories_fancy() {
        let elves = parsed_example_input_1();
        assert_eq!(find_max_calories_fancy(&elves), 24000);
    }

    #[test]
    fn test_part1() {
        assert_eq!(part1(EXAMPLE_INPUT_1).unwrap(), 24000);
    }
}
