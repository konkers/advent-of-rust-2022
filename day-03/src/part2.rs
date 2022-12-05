use std::collections::HashSet;

use anyhow::{anyhow, Result};
use itertools::Itertools;

use crate::item_priority;

struct Rucksack {
    items: HashSet<char>,
}

impl Rucksack {
    fn parse(input: &str) -> Self {
        Rucksack {
            items: input.chars().collect(),
        }
    }

    fn shared_item(&self, a: &Self, b: &Self) -> Result<char> {
        let shared_items: HashSet<_> = self.items.intersection(&a.items).copied().collect();
        let shared_items: Vec<_> = shared_items.intersection(&b.items).copied().collect();

        if shared_items.is_empty() {
            Err(anyhow!("no shared items between rucksacks"))
        } else if shared_items.len() > 1 {
            Err(anyhow!(
                "more chan one shared items between rucksacks: {:?}",
                shared_items
            ))
        } else {
            Ok(shared_items[0])
        }
    }
}

pub fn solution(input: &str) -> Result<u32> {
    input
        .lines()
        .chunks(3)
        .into_iter()
        .map(|mut chunks| {
            let a = chunks
                .next()
                .ok_or_else(|| anyhow!("wrong number of elements in input"))?;
            let b = chunks
                .next()
                .ok_or_else(|| anyhow!("wrong number of elements in input"))?;
            let c = chunks
                .next()
                .ok_or_else(|| anyhow!("wrong number of elements in input"))?;
            let sack_a = Rucksack::parse(a);
            let sack_b = Rucksack::parse(b);
            let sack_c = Rucksack::parse(c);
            let item = sack_a.shared_item(&sack_b, &sack_c)?;
            item_priority(item)
        })
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
        assert_eq!(
            sack.items,
            vec![
                'v', 'J', 'r', 'w', 'p', 'W', 't', 'w', 'J', 'g', 'W', 'r', 'h', 'c', 's', 'F',
                'M', 'M', 'f', 'F', 'F', 'h', 'F', 'p'
            ]
            .into_iter()
            .collect()
        );
    }

    #[test]
    fn shared_item() {
        let sack1 = Rucksack::parse("vJrwpWtwJgWrhcsFMMfFFhFp");
        let sack2 = Rucksack::parse("jqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL");
        let sack3 = Rucksack::parse("PmmdzqPrVvPwwTWBwg");

        assert_eq!(sack1.shared_item(&sack2, &sack3).unwrap(), 'r');

        let sack1 = Rucksack::parse("a");
        let sack2 = Rucksack::parse("b");
        let sack3 = Rucksack::parse("c");
        assert!(sack1.shared_item(&sack2, &sack3).is_err());

        let sack1 = Rucksack::parse("abc");
        let sack2 = Rucksack::parse("abd");
        let sack3 = Rucksack::parse("abe");
        assert!(sack1.shared_item(&sack2, &sack3).is_err());
    }

    #[test]
    fn test_solution() {
        assert_eq!(solution(EXAMPLE_INPUT).unwrap(), 70);
    }
}
