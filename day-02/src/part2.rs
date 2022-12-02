use std::str::FromStr;

use anyhow::{anyhow, bail, Error, Result};

#[derive(Debug, PartialEq)]
pub enum Move {
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
            "A" => Ok(Self::Rock),
            "B" => Ok(Self::Paper),
            "C" => Ok(Self::Scissors),
            _ => Err(anyhow!("unknown move type: {}", s)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Outcome {
    Loss,
    Tie,
    Win,
}

impl Outcome {
    fn calc_move(&self, opponent: &Move) -> Move {
        match (self, opponent) {
            (Self::Loss, Move::Rock) => Move::Scissors,
            (Self::Loss, Move::Paper) => Move::Rock,
            (Self::Loss, Move::Scissors) => Move::Paper,

            (Self::Tie, Move::Rock) => Move::Rock,
            (Self::Tie, Move::Paper) => Move::Paper,
            (Self::Tie, Move::Scissors) => Move::Scissors,

            (Self::Win, Move::Rock) => Move::Paper,
            (Self::Win, Move::Paper) => Move::Scissors,
            (Self::Win, Move::Scissors) => Move::Rock,
        }
    }

    fn score(&self) -> i32 {
        match self {
            Self::Loss => 0,
            Self::Tie => 3,
            Self::Win => 6,
        }
    }
}

impl FromStr for Outcome {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "X" => Ok(Self::Loss),
            "Y" => Ok(Self::Tie),
            "Z" => Ok(Self::Win),
            _ => Err(anyhow!("unknown outcome: {}", s)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Round {
    opponent: Move,
    outcome: Outcome,
}

impl Round {
    fn score(&self) -> i32 {
        let our_move = self.outcome.calc_move(&self.opponent);
        our_move.score() + self.outcome.score()
    }
}

impl FromStr for Round {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let moves: Vec<_> = s.split(" ").collect();
        if moves.len() != 2 {
            bail!("'{}' does not contain exactly two moves", s);
        }
        let opponent = moves[0].parse()?;
        let outcome = moves[1].parse()?;

        Ok(Round { opponent, outcome })
    }
}

pub fn parse_strategy_guide(s: &str) -> Result<Vec<Round>> {
    s.lines().map(|line| line.parse()).collect()
}

pub fn game_score(guide: &Vec<Round>) -> i32 {
    guide.iter().map(|round| round.score()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    const EXAMPLE_INPUT: &'static str = include_str!("example-input.txt");

    #[test]
    fn parse_move() {
        assert_eq!(Move::Rock, "A".parse().unwrap());
        assert_eq!(Move::Paper, "B".parse().unwrap());
        assert_eq!(Move::Scissors, "C".parse().unwrap());

        assert!("".parse::<Move>().is_err());
        assert!("D".parse::<Move>().is_err());
        assert!("X".parse::<Move>().is_err());
        assert!("Y".parse::<Move>().is_err());
        assert!("Z".parse::<Move>().is_err());
    }

    #[test]
    fn parse_outcome() {
        assert_eq!(Outcome::Loss, "X".parse().unwrap());
        assert_eq!(Outcome::Tie, "Y".parse().unwrap());
        assert_eq!(Outcome::Win, "Z".parse().unwrap());

        assert!("".parse::<Outcome>().is_err());
        assert!("A".parse::<Outcome>().is_err());
        assert!("B".parse::<Outcome>().is_err());
        assert!("C".parse::<Outcome>().is_err());
    }

    #[test]
    fn parse_round() {
        assert_eq!(
            Round {
                opponent: Move::Rock,
                outcome: Outcome::Tie
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
                    outcome: Outcome::Tie
                },
                Round {
                    opponent: Move::Paper,
                    outcome: Outcome::Loss
                },
                Round {
                    opponent: Move::Scissors,
                    outcome: Outcome::Win,
                },
            ]
        )
    }
    #[test]
    fn test_outcome_move() {
        // This is a bit of a "change detector" test but does verify core
        // game logic.
        assert_eq!(Outcome::Loss.calc_move(&Move::Rock), Move::Scissors);
        assert_eq!(Outcome::Loss.calc_move(&Move::Paper), Move::Rock);
        assert_eq!(Outcome::Loss.calc_move(&Move::Scissors), Move::Paper);

        assert_eq!(Outcome::Tie.calc_move(&Move::Rock), Move::Rock);
        assert_eq!(Outcome::Tie.calc_move(&Move::Paper), Move::Paper);
        assert_eq!(Outcome::Tie.calc_move(&Move::Scissors), Move::Scissors);

        assert_eq!(Outcome::Win.calc_move(&Move::Rock), Move::Paper);
        assert_eq!(Outcome::Win.calc_move(&Move::Paper), Move::Scissors);
        assert_eq!(Outcome::Win.calc_move(&Move::Scissors), Move::Rock);
    }

    #[test]
    fn round_score() {
        assert_eq!(
            Round {
                opponent: Move::Rock,
                outcome: Outcome::Tie,
            }
            .score(),
            4
        );
        assert_eq!(
            Round {
                opponent: Move::Paper,
                outcome: Outcome::Loss,
            }
            .score(),
            1
        );
        assert_eq!(
            Round {
                opponent: Move::Scissors,
                outcome: Outcome::Win,
            }
            .score(),
            7
        );
    }

    #[test]
    fn test_game_score() {
        assert_eq!(
            game_score(&parse_strategy_guide(EXAMPLE_INPUT).unwrap()),
            12
        );
    }
}
