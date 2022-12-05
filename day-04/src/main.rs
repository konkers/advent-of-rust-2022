use std::{fs, ops::RangeInclusive, path::PathBuf, str::FromStr};

use anyhow::{anyhow, Error, Result};
use clap::Parser;
use nom::{
    bytes::complete::tag,
    character::complete::{char, one_of},
    combinator::{map_res, recognize},
    multi::{many0, many1},
    sequence::terminated,
    Finish, IResult,
};

trait ContainsRange<T: PartialOrd> {
    fn contains_range(&self, range: &RangeInclusive<T>) -> bool;
}

impl<T: PartialOrd> ContainsRange<T> for RangeInclusive<T> {
    fn contains_range(&self, range: &RangeInclusive<T>) -> bool {
        self.contains(range.start()) && self.contains(range.end())
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Pair {
    a: RangeInclusive<u32>,
    b: RangeInclusive<u32>,
}

impl Pair {
    fn parse(input: &str) -> IResult<&str, Self> {
        let (input, a) = range_value(input)?;
        let (input, _) = tag(",")(input)?;
        let (input, b) = range_value(input)?;

        Ok((input, Self { a, b }))
    }

    fn is_completely_overlapping(&self) -> bool {
        self.a.contains_range(&self.b) || self.b.contains_range(&self.a)
    }
}

impl FromStr for Pair {
    // the error must be owned as well
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
            .finish()
            .map_err(|e| anyhow!("Error parsing pair: {}", e))
            .map(|val| val.1)
    }
}

// Adapted from https://github.com/Geal/nom/blob/main/doc/nom_recipes.md#integers
fn decimal_value(input: &str) -> IResult<&str, u32> {
    map_res(
        recognize(many1(terminated(one_of("0123456789"), many0(char('_'))))),
        |value: &str| value.parse::<u32>(),
    )(input)
}

fn range_value(input: &str) -> IResult<&str, RangeInclusive<u32>> {
    let (input, start) = decimal_value(input)?;
    let (input, _) = tag("-")(input)?;
    let (input, end) = decimal_value(input)?;

    Ok((input, start..=end))
}

fn solution(input: &str) -> Result<u32> {
    input
        .lines()
        .map(|line| {
            // Rust bools are guaranteed to be 0 or 1.
            Ok(line.parse::<Pair>()?.is_completely_overlapping() as u32)
        })
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

    let total = solution(&input)?;
    println!(
        "[Part: 1] Number of completely overlapping ranges: {}",
        total
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const EXAMPLE_INPUT: &str = include_str!("example-input.txt");

    #[test]
    fn test_decimal_value() {
        assert_eq!(decimal_value("0").unwrap(), ("", 0));
    }

    #[test]
    fn test_range_value() {
        assert_eq!(range_value("0-1").unwrap(), ("", 0..=1));
    }

    #[test]
    fn parse_pair() {
        assert_eq!(
            "2-4,6-8".parse::<Pair>().unwrap(),
            Pair { a: 2..=4, b: 6..=8 }
        );
    }

    #[test]
    fn pair_overlap() {
        assert!(!"2-4,6-8"
            .parse::<Pair>()
            .unwrap()
            .is_completely_overlapping());
        assert!(!"2-3,4-6"
            .parse::<Pair>()
            .unwrap()
            .is_completely_overlapping());
        assert!(!"5-7,7-9"
            .parse::<Pair>()
            .unwrap()
            .is_completely_overlapping());
        assert!("2-8,3-7"
            .parse::<Pair>()
            .unwrap()
            .is_completely_overlapping());

        // Test all combinations of overlaps.
        assert!("6-6,4-6"
            .parse::<Pair>()
            .unwrap()
            .is_completely_overlapping());
        assert!("4-4,4-6"
            .parse::<Pair>()
            .unwrap()
            .is_completely_overlapping());
        assert!("4-6,4-6"
            .parse::<Pair>()
            .unwrap()
            .is_completely_overlapping());
        assert!("4-6,4-4"
            .parse::<Pair>()
            .unwrap()
            .is_completely_overlapping());

        assert!(!"2-6,4-8"
            .parse::<Pair>()
            .unwrap()
            .is_completely_overlapping());
    }

    #[test]
    fn test_solution() {
        assert_eq!(solution(EXAMPLE_INPUT).unwrap(), 2);
    }
}
