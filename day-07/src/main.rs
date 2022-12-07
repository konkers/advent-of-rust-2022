use std::{fmt, fs, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use env_logger::Env;
use indextree::{Arena, NodeEdge, NodeId};
use log::{debug, error, info};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1, char, line_ending, one_of, space1},
    combinator::{map_res, recognize},
    multi::{many0, many0_count, many1, separated_list1},
    sequence::{pair, terminated},
    IResult,
};

// Adapted from https://github.com/Geal/nom/blob/main/doc/nom_recipes.md#integers
fn decimal_value(input: &str) -> IResult<&str, u64> {
    map_res(
        recognize(many1(terminated(one_of("0123456789"), many0(char('_'))))),
        |value: &str| value.parse::<u64>(),
    )(input)
}

fn separator(input: &str) -> IResult<&str, &str> {
    alt((tag("_"), tag("-"), tag(".")))(input)
}

// Adapted from https://docs.rs/nom/latest/nom/recipes/index.html#rust-style-identifiers
fn file_name(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, separator)),
        many0_count(alt((alphanumeric1, separator))),
    ))(input)
}

#[derive(Debug, Eq, PartialEq)]
enum Directory {
    Root,
    Parent,
    Child(String),
}

impl Directory {
    fn parse_root(input: &str) -> IResult<&str, Self> {
        let (input, _) = tag("/")(input)?;
        Ok((input, Self::Root))
    }

    fn parse_parent(input: &str) -> IResult<&str, Self> {
        let (input, _) = tag("..")(input)?;
        Ok((input, Self::Parent))
    }

    fn parse_child(input: &str) -> IResult<&str, Self> {
        let (input, name) = file_name(input)?;
        Ok((input, Self::Child(name.into())))
    }

    fn parse(input: &str) -> IResult<&str, Self> {
        alt((Self::parse_root, Self::parse_parent, Self::parse_child))(input)
    }
}

#[derive(Debug, Eq, PartialEq)]
enum DirectoryEntry {
    File { name: String, size: u64 },
    Directory { name: String },
}

impl DirectoryEntry {
    fn parse_file(input: &str) -> IResult<&str, Self> {
        let (input, size) = decimal_value(input)?;
        let (input, _) = space1(input)?;
        let (input, name) = file_name(input)?;
        Ok((
            input,
            Self::File {
                name: name.into(),
                size,
            },
        ))
    }

    fn parse_directory(input: &str) -> IResult<&str, Self> {
        let (input, _) = tag("dir")(input)?;
        let (input, _) = space1(input)?;
        let (input, name) = file_name(input)?;
        Ok((input, Self::Directory { name: name.into() }))
    }

    fn parse(input: &str) -> IResult<&str, Self> {
        alt((Self::parse_file, Self::parse_directory))(input)
    }

    fn name(&self) -> &str {
        match self {
            Self::Directory { name } => name,
            Self::File { name, size: _ } => name,
        }
    }
}

impl fmt::Display for DirectoryEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Directory { name } => write!(f, "{name} (dir)"),
            Self::File { name, size } => write!(f, "{name} (file, size={size})"),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Command {
    Cd(Directory),
    Ls(Vec<DirectoryEntry>),
}

struct CommandIterator<'a> {
    input: &'a str,
}

impl Iterator for CommandIterator<'_> {
    type Item = Command;

    fn next(&mut self) -> Option<Self::Item> {
        if self.input.is_empty() {
            return None;
        }
        match Command::parse(self.input) {
            Ok((input, command)) => {
                self.input = input;
                debug!("parsed {:?}", command);
                Some(command)
            }
            Err(e) => {
                error!("parse error: {}", e);
                None
            }
        }
    }
}

impl Command {
    fn parse_cd(input: &str) -> IResult<&str, Self> {
        let (input, _) = tag("cd")(input)?;
        let (input, _) = space1(input)?;
        let (input, directory) = Directory::parse(input)?;
        let (input, _) = many1(line_ending)(input)?;

        Ok((input, Self::Cd(directory)))
    }

    fn parse_ls(input: &str) -> IResult<&str, Self> {
        let (input, _) = tag("ls")(input)?;
        let (input, _) = line_ending(input)?;
        let (input, entries) = separated_list1(line_ending, DirectoryEntry::parse)(input)?;
        let (input, _) = many1(line_ending)(input)?;

        Ok((input, Self::Ls(entries)))
    }

    fn parse(input: &str) -> IResult<&str, Self> {
        let (input, _) = tag("$")(input)?;
        let (input, _) = space1(input)?;
        alt((Self::parse_cd, Self::parse_ls))(input)
    }

    fn parse_multiple(input: &str) -> CommandIterator {
        CommandIterator { input }
    }
}

struct Filesystem {
    root: NodeId,
    arena: Arena<DirectoryEntry>,
}

impl Filesystem {
    fn parse(input: &str) -> Self {
        let mut arena = Arena::new();
        let root = arena.new_node(DirectoryEntry::Directory { name: "/".into() });
        let mut current_dir = root;

        for command in Command::parse_multiple(input) {
            match command {
                // Assume this only occurs at the start of the input and ignore
                Command::Cd(Directory::Root) => (),
                Command::Cd(Directory::Parent) => {
                    // Assume input is valid ("cd .." only occurs in directories
                    // with parents.
                    current_dir = arena.get(current_dir).unwrap().parent().unwrap();
                }
                Command::Cd(Directory::Child(name)) => {
                    // Linear search through directory entries.
                    for entry in current_dir.children(&arena) {
                        if arena.get(entry).unwrap().get()
                            == (&DirectoryEntry::Directory { name: name.clone() })
                        {
                            current_dir = entry;
                            break;
                        }
                    }
                }
                Command::Ls(entries) => {
                    for entry in entries {
                        current_dir.append(arena.new_node(entry), &mut arena);
                    }
                }
            }
        }

        Self { root, arena }
    }

    fn filter_subdirs_by_size(
        &self,
        filter: &impl Fn(u64) -> bool,
        dir: NodeId,
        dirs: &mut Vec<(String, u64)>,
    ) -> u64 {
        let mut size = 0;
        for child in dir.children(&self.arena) {
            match self.arena.get(child).unwrap().get() {
                DirectoryEntry::File {
                    name: _,
                    size: file_size,
                } => {
                    size += file_size;
                }
                DirectoryEntry::Directory { name: _ } => {
                    let dir_size = self.filter_subdirs_by_size(filter, child, dirs);
                    size += dir_size;
                }
            }
        }
        if filter(size) {
            dirs.push((self.arena.get(dir).unwrap().get().name().to_owned(), size))
        }

        size
    }

    fn filter_dirs_by_size(&self, filter: impl Fn(u64) -> bool + 'static) -> Vec<(String, u64)> {
        let mut dirs = Vec::new();
        self.filter_subdirs_by_size(&filter, self.root, &mut dirs);
        dirs
    }

    fn total_size(&self) -> u64 {
        let mut dirs = Vec::new();
        self.filter_subdirs_by_size(&|_| false, self.root, &mut dirs)
    }
}

impl fmt::Display for Filesystem {
    // Format according to the visual example in the challenge.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut indent = String::new();

        for edge in self.root.traverse(&self.arena) {
            match edge {
                NodeEdge::Start(id) => {
                    let node = self.arena.get(id).unwrap().get();
                    writeln!(f, "{}- {}", indent, node)?;

                    indent.push_str("  ")
                }
                NodeEdge::End(_) => {
                    indent.truncate(indent.len() - 2);
                }
            }
        }

        Ok(())
    }
}

fn solution_part1(fs: &Filesystem) -> u64 {
    fs.filter_dirs_by_size(|size| size <= 100000)
        .iter()
        .map(|(_name, size)| size)
        .sum()
}

fn solution_part2(fs: &Filesystem) -> u64 {
    // Calling `fs.total_size()` here causes fs to be traversed twice.  This
    // could be optimized by calculating directory sizes at Filesystem
    // creation time.
    let size_to_free = 30000000 - (70000000 - fs.total_size());
    let filter = move |size| size >= size_to_free;
    *fs.filter_dirs_by_size(filter)
        .iter()
        .map(|(_name, size)| size)
        .min()
        .unwrap()
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

    let fs = Filesystem::parse(&input);

    let total = solution_part1(&fs);
    info!("[Part 1] Sum of directory sizes under 100000: {total}");

    let size = solution_part2(&fs);
    info!("[Part 2] Size of directory to free: {size}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;

    const EXAMPLE_INPUT: &str = include_str!("example-input.txt");

    #[test]
    fn parse_directory() {
        assert_eq!(Directory::parse("/").unwrap(), ("", Directory::Root));
        assert_eq!(Directory::parse("..").unwrap(), ("", Directory::Parent));
        assert_eq!(
            Directory::parse("test").unwrap(),
            ("", Directory::Child("test".into()))
        );
    }

    #[test]
    fn parse_directory_entry() {
        assert_eq!(
            DirectoryEntry::parse("12345 test").unwrap(),
            (
                "",
                DirectoryEntry::File {
                    name: "test".into(),
                    size: 12345
                }
            )
        );

        assert_eq!(
            DirectoryEntry::parse("dir testdir").unwrap(),
            (
                "",
                DirectoryEntry::Directory {
                    name: "testdir".into(),
                }
            )
        );
    }

    #[test]
    fn parse_command() {
        assert_eq!(
            Command::parse("$ cd /\n").unwrap(),
            ("", Command::Cd(Directory::Root))
        );
        assert_eq!(
            Command::parse("$ cd ..\n").unwrap(),
            ("", Command::Cd(Directory::Parent))
        );
        assert_eq!(
            Command::parse("$ cd testdir\n").unwrap(),
            ("", Command::Cd(Directory::Child("testdir".into())))
        );
        assert_eq!(
            Command::parse(indoc! {r#"
                $ ls
                dir a
                14848514 b.txt
                8504156 c.dat
                dir d
            "#})
            .unwrap(),
            (
                "",
                Command::Ls(vec![
                    DirectoryEntry::Directory { name: "a".into() },
                    DirectoryEntry::File {
                        name: "b.txt".into(),
                        size: 14848514
                    },
                    DirectoryEntry::File {
                        name: "c.dat".into(),
                        size: 8504156
                    },
                    DirectoryEntry::Directory { name: "d".into() },
                ])
            )
        );
    }

    #[test]
    fn parse_multiple() {
        assert_eq!(
            Command::parse_multiple(indoc! {r#"
                $ cd /
                $ ls
                dir a
                14848514 b.txt
                8504156 c.dat
                dir d
            "#})
            .collect::<Vec<_>>(),
            vec![
                Command::Cd(Directory::Root),
                Command::Ls(vec![
                    DirectoryEntry::Directory { name: "a".into() },
                    DirectoryEntry::File {
                        name: "b.txt".into(),
                        size: 14848514
                    },
                    DirectoryEntry::File {
                        name: "c.dat".into(),
                        size: 8504156
                    },
                    DirectoryEntry::Directory { name: "d".into() },
                ]),
            ]
        );
    }

    #[test]
    fn parse_fs() {
        let text = format!("{}", Filesystem::parse(EXAMPLE_INPUT));
        println!("{text}");
        assert_eq!(
            text,
            indoc! {"
            - / (dir)
              - a (dir)
                - e (dir)
                  - i (file, size=584)
                - f (file, size=29116)
                - g (file, size=2557)
                - h.lst (file, size=62596)
              - b.txt (file, size=14848514)
              - c.dat (file, size=8504156)
              - d (dir)
                - j (file, size=4060174)
                - d.log (file, size=8033020)
                - d.ext (file, size=5626152)
                - k (file, size=7214296)
    "}
        )
    }

    #[test]
    fn filter_dirs() {
        let fs = Filesystem::parse(EXAMPLE_INPUT);
        assert_eq!(
            fs.filter_dirs_by_size(|size| size <= 100000),
            vec![("e".to_string(), 584), ("a".to_string(), 94853)]
        );
    }

    #[test]
    fn fs_size() {
        let fs = Filesystem::parse(EXAMPLE_INPUT);
        assert_eq!(fs.total_size(), 48381165);
    }

    #[test]
    fn part1() {
        let fs = Filesystem::parse(EXAMPLE_INPUT);
        assert_eq!(solution_part1(&fs), 95437);
    }

    #[test]
    fn part2() {
        let fs = Filesystem::parse(EXAMPLE_INPUT);
        assert_eq!(solution_part2(&fs), 24933642);
    }
}
