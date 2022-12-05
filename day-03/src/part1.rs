use std::collections::HashSet;

use anyhow::Result;

use crate::item_priority;

struct Rucksack {
    // Define rucksack as having multiple compartments in expectation that part 2 will need it.
    compartments: Vec<HashSet<char>>,
}

impl Rucksack {
    pub fn parse(input: &str) -> Self {
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

pub fn solution(input: &str) -> Result<u32> {
    input
        .lines()
        .map(|line| Rucksack::parse(line).shared_item_priority())
        .sum()
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
    fn test_solution() {
        assert_eq!(solution(EXAMPLE_INPUT).unwrap(), 157);
    }
}
