use std::{collections::VecDeque, fs, path::PathBuf, str::FromStr};

use anyhow::{anyhow, Error, Result};
use clap::Parser;
use env_logger::Env;
use log::{debug, info};
use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    character::complete::{char, line_ending, one_of},
    combinator::{map_res, recognize},
    multi::{many0, many1, separated_list1},
    sequence::terminated,
    Finish, IResult,
};

#[derive(Debug, Eq, PartialEq)]
struct Instruction {
    src: usize,
    dest: usize,
    amount: usize,
}

impl Instruction {
    fn parse(i: &str) -> IResult<&str, Self> {
        let (i, _) = tag("move ")(i)?;
        let (i, amount) = decimal_value(i)?;
        let (i, _) = tag(" from ")(i)?;
        let (i, src) = decimal_value(i)?;
        let (i, _) = tag(" to ")(i)?;
        let (i, dest) = decimal_value(i)?;

        // Convert from 1 based indexing to 0 based.
        Ok((
            i,
            Self {
                src: src - 1,
                dest: dest - 1,
                amount,
            },
        ))
    }
}

// Adapted from https://github.com/Geal/nom/blob/main/doc/nom_recipes.md#integers
fn decimal_value(input: &str) -> IResult<&str, usize> {
    map_res(
        recognize(many1(terminated(one_of("0123456789"), many0(char('_'))))),
        |value: &str| value.parse::<usize>(),
    )(input)
}

fn parse_empty_stack(i: &str) -> IResult<&str, Option<char>> {
    let (i, _) = tag("   ")(i)?;
    Ok((i, None))
}

fn parse_stack_content(i: &str) -> IResult<&str, Option<char>> {
    let (i, _) = char('[')(i)?;
    let (i, value) = take(1usize)(i)?;
    let (i, _) = char(']')(i)?;

    Ok((i, value.chars().next()))
}

fn parse_stack_position(i: &str) -> IResult<&str, Option<char>> {
    alt((parse_empty_stack, parse_stack_content))(i)
}

fn parse_stack_level(i: &str) -> IResult<&str, Vec<Option<char>>> {
    separated_list1(char(' '), parse_stack_position)(i)
}

fn parse_stack_index(i: &str) -> IResult<&str, u32> {
    let (i, _) = char(' ')(i)?;
    let (i, value) = map_res(take(1usize), |value: &str| value.parse::<u32>())(i)?;
    let (i, _) = char(' ')(i)?;

    Ok((i, value))
}

fn parse_stack_indices(i: &str) -> IResult<&str, Vec<u32>> {
    separated_list1(char(' '), parse_stack_index)(i)
}

#[derive(Debug, Eq, PartialEq)]
struct Stack {
    values: VecDeque<char>,
    index: u32,
}

impl Stack {
    fn pop(&mut self) -> Result<char> {
        self.values.pop_back().ok_or_else(|| anyhow!("stack empty"))
    }

    fn push(&mut self, val: char) {
        self.values.push_back(val)
    }

    fn take(&mut self, num_elements: usize) -> Result<VecDeque<char>> {
        if num_elements > self.values.len() {
            return Err(anyhow!(
                "Can't pop {num_elements} from stack of length {}",
                self.values.len()
            ));
        }
        Ok(self.values.split_off(self.values.len() - num_elements))
    }

    fn peek(&self) -> Result<char> {
        self.values
            .back()
            .copied()
            .ok_or_else(|| anyhow!("stack empty"))
    }
}

fn parse_stacks(input: &str) -> IResult<&str, Vec<Stack>> {
    let (input, levels) = separated_list1(line_ending, parse_stack_level)(input)?;
    let (input, _) = line_ending(input)?;
    let (input, indices) = parse_stack_indices(input)?;
    let (input, _) = line_ending(input)?;

    for level in &levels {
        assert_eq!(level.len(), indices.len())
    }

    let stacks: Vec<_> = indices
        .into_iter()
        .enumerate()
        .map(|(i, index)| {
            let values: VecDeque<_> = levels.iter().filter_map(|val| val[i]).fold(
                VecDeque::new(),
                |mut values, value| {
                    values.push_front(value);
                    values
                },
            );
            Stack { values, index }
        })
        .collect();

    Ok((input, stacks))
}

#[derive(Debug, Eq, PartialEq)]
struct Problem {
    stacks: Vec<Stack>,
    instructions: VecDeque<Instruction>,
}

impl Problem {
    fn parse(i: &str) -> IResult<&str, Self> {
        let (i, stacks) = parse_stacks(i)?;
        let (i, _) = line_ending(i)?;
        let (i, instructions) = separated_list1(line_ending, Instruction::parse)(i)?;
        let (i, _) = line_ending(i)?;

        Ok((
            i,
            Self {
                stacks,
                instructions: instructions.into(),
            },
        ))
    }

    fn step(&mut self) -> Result<()> {
        let instruction = &self
            .instructions
            .pop_front()
            .ok_or_else(|| anyhow!("step called with empty instructions"))?;
        for _ in 0..instruction.amount {
            let val = self.stacks[instruction.src].pop()?;
            self.stacks[instruction.dest].push(val);
        }

        Ok(())
    }

    fn execute(&mut self) -> Result<()> {
        while !self.instructions.is_empty() {
            self.step()?;
        }

        Ok(())
    }

    fn step2(&mut self) -> Result<()> {
        let instruction = &self
            .instructions
            .pop_front()
            .ok_or_else(|| anyhow!("step called with empty instructions"))?;
        debug!(
            "move {} from {} to {}",
            instruction.amount,
            // Convert back to 1 based indexing for printing.
            instruction.src + 1,
            instruction.dest + 1
        );
        let values = self.stacks[instruction.src].take(instruction.amount)?;
        for val in values {
            self.stacks[instruction.dest].push(val);
        }
        for stack in &self.stacks {
            debug!("  {}: {:?}", stack.index, stack.values);
        }

        Ok(())
    }

    fn execute2(&mut self) -> Result<()> {
        for stack in &self.stacks {
            debug!("  {}: {:?}", stack.index, stack.values);
        }

        while !self.instructions.is_empty() {
            self.step2()?;
        }

        Ok(())
    }
}

impl FromStr for Problem {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
            .finish()
            .map_err(|e| anyhow!("Error parsing pair: {}", e))
            .map(|val| val.1)
    }
}

fn solution_part1(input: &str) -> Result<String> {
    let mut problem = input.parse::<Problem>()?;
    problem.execute()?;
    problem.stacks.iter().map(|stack| stack.peek()).collect()
}

fn solution_part2(input: &str) -> Result<String> {
    let mut problem = input.parse::<Problem>()?;
    problem.execute2()?;
    problem.stacks.iter().map(|stack| stack.peek()).collect()
}

// Command line arguments.
#[derive(Debug, Parser)]
struct Args {
    input: PathBuf,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let args = Args::parse();
    let input = fs::read_to_string(&args.input)?;

    let top = solution_part1(&input)?;
    info!("[Part: 1] Top of stacks: {}", top);

    let top = solution_part2(&input)?;
    info!("[Part: 2] Top of stacks: {}", top);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = include_str!("example-input.txt");

    fn parsed_example() -> Problem {
        Problem {
            stacks: vec![
                Stack {
                    values: ['Z', 'N'].into(),
                    index: 1,
                },
                Stack {
                    values: ['M', 'C', 'D'].into(),
                    index: 2,
                },
                Stack {
                    values: ['P'].into(),
                    index: 3,
                },
            ],
            instructions: [
                Instruction {
                    src: 1,
                    dest: 0,
                    amount: 1,
                },
                Instruction {
                    src: 0,
                    dest: 2,
                    amount: 3,
                },
                Instruction {
                    src: 1,
                    dest: 0,
                    amount: 2,
                },
                Instruction {
                    src: 0,
                    dest: 1,
                    amount: 1,
                },
            ]
            .into(),
        }
    }

    #[test]
    fn test_parse_stack_level() {
        assert_eq!(
            parse_stack_level("    [D]    ").unwrap(),
            ("", vec![None, Some('D'), None])
        );
        assert_eq!(
            parse_stack_level("[N] [C]    ").unwrap(),
            ("", vec![Some('N'), Some('C'), None])
        );
        assert_eq!(
            parse_stack_level("[Z] [M] [P]").unwrap(),
            ("", vec![Some('Z'), Some('M'), Some('P')])
        );
    }

    #[test]
    fn test_parse_stacks() {
        assert_eq!(
            parse_stacks("    [D]    \n[N] [C]    \n[Z] [M] [P]\n 1   2   3 \n").unwrap(),
            ("", parsed_example().stacks)
        );
    }

    #[test]
    fn test_parse_problem() {
        assert_eq!(EXAMPLE_INPUT.parse::<Problem>().unwrap(), parsed_example());
    }

    #[test]
    fn test_parse_stack_indices() {
        assert_eq!(
            parse_stack_indices(" 1   2   3 ").unwrap(),
            ("", vec![1, 2, 3])
        );
    }

    #[test]
    fn test_parse_instruction() {
        assert_eq!(
            Instruction::parse("move 1 from 2 to 3").unwrap(),
            (
                "",
                Instruction {
                    src: 1,
                    dest: 2,
                    amount: 1
                }
            )
        );
    }
    #[test]
    fn test_stack_take() {
        let mut stack = Stack {
            values: ['A', 'B', 'C', 'D'].into(),
            index: 1,
        };

        assert_eq!(stack.take(2).unwrap(), ['C', 'D']);
        assert_eq!(stack.values, ['A', 'B']);
    }

    #[test]
    fn test_execute() {
        let mut problem = EXAMPLE_INPUT.parse::<Problem>().unwrap();
        problem.execute().unwrap();
        assert_eq!(
            problem,
            Problem {
                stacks: vec![
                    Stack {
                        values: ['C'].into(),
                        index: 1
                    },
                    Stack {
                        values: ['M'].into(),
                        index: 2
                    },
                    Stack {
                        values: ['P', 'D', 'N', 'Z'].into(),
                        index: 3
                    }
                ],
                instructions: [].into()
            }
        );
    }
    #[test]
    fn test_execute2() {
        let mut problem = EXAMPLE_INPUT.parse::<Problem>().unwrap();
        problem.execute2().unwrap();
        assert_eq!(
            problem,
            Problem {
                stacks: vec![
                    Stack {
                        values: ['M'].into(),
                        index: 1
                    },
                    Stack {
                        values: ['C'].into(),
                        index: 2
                    },
                    Stack {
                        values: ['P', 'Z', 'N', 'D'].into(),
                        index: 3
                    }
                ],
                instructions: [].into()
            }
        );
    }

    #[test]
    fn test_part1() {
        assert_eq!(solution_part1(EXAMPLE_INPUT).unwrap(), "CMZ".to_string());
    }

    #[test]
    fn test_part2() {
        assert_eq!(solution_part2(EXAMPLE_INPUT).unwrap(), "MCD".to_string());
    }
}
