use std::{fs, path::PathBuf, str::FromStr};

use anyhow::{anyhow, bail, Error, Result};
use clap::Parser;

// Command line arguments.
#[derive(Debug, Parser)]
struct Args {
    input: PathBuf,
}

#[derive(Debug, PartialEq)]
enum Move {
    Rock,
    Paper,
    Scissors,
}

impl Move {
    fn score(&self) -> i32 {
        match self {
            Self::Rock => 1,
            Self::Paper => 2,
            Self::Scissors => 3,
        }
    }
}
impl FromStr for Move {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "A" | "X" => Ok(Self::Rock),
            "B" | "Y" => Ok(Self::Paper),
            "C" | "Z" => Ok(Self::Scissors),
            _ => Err(anyhow!("unknown move type: {}", s)),
        }
    }
}

#[derive(Debug, PartialEq)]
struct Round {
    opponent: Move,
    ours: Move,
}

impl Round {
    pub fn score(&self) -> i32 {
        let outcome_score = match (&self.opponent, &self.ours) {
            // Wins
            (Move::Rock, Move::Paper)
            | (Move::Paper, Move::Scissors)
            | (Move::Scissors, Move::Rock) => 6,

            // Draws
            (Move::Rock, Move::Rock)
            | (Move::Paper, Move::Paper)
            | (Move::Scissors, Move::Scissors) => 3,

            // Losses
            (Move::Rock, Move::Scissors)
            | (Move::Paper, Move::Rock)
            | (Move::Scissors, Move::Paper) => 0,
        };

        outcome_score + self.ours.score()
    }
}

impl FromStr for Round {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let moves: Vec<_> = s.split(' ').collect();
        if moves.len() != 2 {
            bail!("'{}' does not contain exactly two moves", s);
        }
        let opponent = moves[0].parse()?;
        let ours = moves[1].parse()?;

        Ok(Round { opponent, ours })
    }
}

fn parse_strategy_guide(s: &str) -> Result<Vec<Round>> {
    s.lines().map(|line| line.parse()).collect()
}

fn game_score(guide: &[Round]) -> i32 {
    guide.iter().map(|round| round.score()).sum()
}

fn main() -> Result<()> {
    let args = Args::parse();
    let input = fs::read_to_string(&args.input)?;

    let guide = parse_strategy_guide(&input)?;

    let score = game_score(&guide);

    println!("[Part 1] Score: {}", score);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const EXAMPLE_INPUT: &str = include_str!("example-input.txt");

    #[test]
    fn parse_move() {
        assert_eq!(Move::Rock, "A".parse().unwrap());
        assert_eq!(Move::Paper, "B".parse().unwrap());
        assert_eq!(Move::Scissors, "C".parse().unwrap());

        assert_eq!(Move::Rock, "X".parse().unwrap());
        assert_eq!(Move::Paper, "Y".parse().unwrap());
        assert_eq!(Move::Scissors, "Z".parse().unwrap());

        assert!("".parse::<Move>().is_err());
        assert!("D".parse::<Move>().is_err());
    }

    #[test]
    fn parse_round() {
        assert_eq!(
            Round {
                opponent: Move::Rock,
                ours: Move::Paper
            },
            "A Y".parse().unwrap()
        );

        assert!("".parse::<Round>().is_err());
        assert!("A".parse::<Round>().is_err());
        assert!("A Y Z".parse::<Round>().is_err());
    }

    #[test]
    fn test_parse_strategy_guide() {
        assert_eq!(
            parse_strategy_guide(EXAMPLE_INPUT).unwrap(),
            vec![
                Round {
                    opponent: Move::Rock,
                    ours: Move::Paper
                },
                Round {
                    opponent: Move::Paper,
                    ours: Move::Rock,
                },
                Round {
                    opponent: Move::Scissors,
                    ours: Move::Scissors,
                },
            ]
        )
    }

    #[test]
    fn round_score() {
        assert_eq!(
            Round {
                opponent: Move::Rock,
                ours: Move::Paper
            }
            .score(),
            8
        );
        assert_eq!(
            Round {
                opponent: Move::Paper,
                ours: Move::Rock,
            }
            .score(),
            1
        );
        assert_eq!(
            Round {
                opponent: Move::Scissors,
                ours: Move::Scissors,
            }
            .score(),
            6
        );
    }

    #[test]
    fn test_game_score() {
        assert_eq!(
            game_score(&parse_strategy_guide(EXAMPLE_INPUT).unwrap()),
            15
        );
    }
}
