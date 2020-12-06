use combine::parser::char::{char, letter};
use combine::stream::Stream;
use combine::{many1, sep_by, sep_end_by, EasyParser, Parser};
use std::collections::HashSet;
use std::io::{self, Read};
use std::num;
use std::str::FromStr;

fn require_no_remaining(remaining: &str) -> Result<()> {
    if remaining != "" {
        Err(Error::ExtraneousInput(remaining.to_owned()))
    } else {
        Ok(())
    }
}

macro_rules! parser_from_str {
    ($s:ident) => {
        impl FromStr for $s {
            type Err = Error;
            fn from_str(input: &str) -> Result<Self> {
                let (p, remaining): (Self, &str) = Self::parser().easy_parse(input)?;
                require_no_remaining(remaining)?;
                Ok(p)
            }
        }
    };
}

#[derive(Debug)]
enum Error {
    ParseInt(num::ParseIntError),
    Io(io::Error),
    ParseError(String),
    ExtraneousInput(String),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<num::ParseIntError> for Error {
    fn from(e: num::ParseIntError) -> Self {
        Self::ParseInt(e)
    }
}

impl From<combine::easy::ParseError<&str>> for Error {
    fn from(e: combine::easy::ParseError<&str>) -> Self {
        Self::ParseError(e.to_string())
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
struct Answer(HashSet<char>);

impl Answer {
    fn parser<Input>() -> impl Parser<Input, Output = Self>
    where
        Input: Stream<Token = char>,
    {
        many1(letter()).map(|v: Vec<char>| Self(v.iter().cloned().collect()))
    }
}

#[derive(Debug)]
struct Group(Vec<Answer>);

impl Group {
    fn parser<Input>() -> impl Parser<Input, Output = Self>
    where
        Input: Stream<Token = char>,
    {
        sep_end_by(Answer::parser(), char('\n')).map(|v: Vec<_>| Self(v))
    }

    fn anyone_yes_count(&self) -> usize {
        let mut all = HashSet::new();
        for a in &self.0 {
            all = all.union(&a.0).cloned().collect();
        }
        all.len()
    }

    fn everyone_yes_count(&self) -> usize {
        let mut all = self.0[0].0.clone();
        for a in &self.0 {
            all = all.intersection(&a.0).cloned().collect();
        }
        all.len()
    }
}

parser_from_str!(Group);

#[derive(Debug)]
struct GroupCollection(Vec<Group>);

impl GroupCollection {
    fn parser<Input>() -> impl Parser<Input, Output = Self>
    where
        Input: Stream<Token = char>,
    {
        sep_by(Group::parser(), char('\n')).map(|v| Self(v))
    }
}

impl FromStr for GroupCollection {
    type Err = Error;
    fn from_str(input: &str) -> Result<Self> {
        let (p, remaining): (Self, &str) = Self::parser().easy_parse(input)?;
        require_no_remaining(remaining)?;
        Ok(p)
    }
}

fn part_one(groups: &GroupCollection) {
    let answer: usize = groups.0.iter().map(|g| g.anyone_yes_count()).sum();
    println!("{}", answer);
}

fn part_two(groups: &GroupCollection) {
    let answer: usize = groups.0.iter().map(|g| g.everyone_yes_count()).sum();
    println!("{}", answer);
}

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let groups: GroupCollection = input.parse()?;

    println!("Part 1");
    part_one(&groups);

    println!("Part 2");
    part_two(&groups);

    Ok(())
}
