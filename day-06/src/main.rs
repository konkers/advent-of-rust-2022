use std::{collections::HashSet, fs, path::PathBuf};

use anyhow::{anyhow, Result};
use clap::Parser;

fn find_marker<const N: usize>(input: &str) -> Result<usize> {
    for i in 0..input.len() - N {
        let chars: HashSet<_> = input[i..i + N].chars().collect();
        if chars.len() == N {
            return Ok(i + N);
        }
    }

    Err(anyhow!("unable to find start of frame sequence"))
}

// Command line arguments.
#[derive(Debug, Parser)]
struct Args {
    input: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let input = fs::read_to_string(&args.input)?;

    let start_of_frame = find_marker::<4>(&input)?;
    println!("[Part 1] Start of frame: {}", start_of_frame);

    let start_of_message = find_marker::<14>(&input)?;
    println!("[Part 2] Start of message: {}", start_of_message);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_of_frame() {
        assert_eq!(
            find_marker::<4>("mjqjpqmgbljsphdztnvjfqwrcgsmlb").unwrap(),
            7
        );
        assert_eq!(find_marker::<4>("bvwbjplbgvbhsrlpgdmjqwftvncz").unwrap(), 5);
        assert_eq!(find_marker::<4>("nppdvjthqldpwncqszvftbrmjlhg").unwrap(), 6);
        assert_eq!(
            find_marker::<4>("nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg").unwrap(),
            10
        );
        assert_eq!(
            find_marker::<4>("zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw").unwrap(),
            11
        );
    }

    #[test]
    fn start_of_message() {
        assert_eq!(
            find_marker::<14>("mjqjpqmgbljsphdztnvjfqwrcgsmlb").unwrap(),
            19
        );
        assert_eq!(
            find_marker::<14>("bvwbjplbgvbhsrlpgdmjqwftvncz").unwrap(),
            23
        );
        assert_eq!(
            find_marker::<14>("nppdvjthqldpwncqszvftbrmjlhg").unwrap(),
            23
        );
        assert_eq!(
            find_marker::<14>("nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg").unwrap(),
            29
        );
        assert_eq!(
            find_marker::<14>("zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw").unwrap(),
            26
        );
    }
}
