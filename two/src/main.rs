use combine::parser::char::{char, digit, letter, spaces};
use combine::stream::Stream;
use combine::{many1, EasyParser, Parser};
use std::io::{self, BufRead};
use std::num;
use std::str::FromStr;

#[derive(Debug)]
enum Error {
    ParseInt(num::ParseIntError),
    Io(io::Error),
    ParseError(String),
    ExtraneousInput,
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

fn parse_lines<R: BufRead, T: FromStr>(lines: R) -> Result<Vec<T>>
where
    Error: From<<T as FromStr>::Err>,
{
    let mut values = vec![];
    for maybe_line in lines.lines() {
        values.push(maybe_line?.parse()?);
    }
    Ok(values)
}

#[derive(Debug)]
struct Numbers(usize, usize);

#[derive(Debug)]
struct PasswordPolicy {
    numbers: Numbers,
    letter: char,
}

fn require_no_remaining(remaining: &str) -> Result<()> {
    if remaining != "" {
        Err(Error::ExtraneousInput)
    } else {
        Ok(())
    }
}

impl PasswordPolicy {
    fn parser<Input>() -> impl Parser<Input, Output = Self>
    where
        Input: Stream<Token = char>,
    {
        let integer = || many1(digit()).map(|s: String| s.parse::<usize>().unwrap());
        let numbers = integer()
            .skip(char('-'))
            .and(integer())
            .map(|(start, end)| Numbers(start, end));
        let letter = spaces().with(letter());
        numbers
            .and(letter)
            .map(|(numbers, letter)| Self { numbers, letter })
    }

    fn part_one_validate(&self, password: &str) -> bool {
        let n = password.chars().filter(|c| *c == self.letter).count();
        n >= self.numbers.0 && n <= self.numbers.1
    }

    fn part_two_validate(&self, password: &str) -> bool {
        (password.chars().nth(self.numbers.0 - 1).unwrap() == self.letter)
            ^ (password.chars().nth(self.numbers.1 - 1).unwrap() == self.letter)
    }
}

impl FromStr for PasswordPolicy {
    type Err = Error;
    fn from_str(input: &str) -> Result<Self> {
        let (p, remaining): (Self, &str) = Self::parser().easy_parse(input)?;
        require_no_remaining(remaining)?;
        Ok(p)
    }
}

#[derive(Debug)]
struct PasswordDatabaseEntry {
    policy: PasswordPolicy,
    password: String,
}

impl PasswordDatabaseEntry {
    fn parser<Input>() -> impl Parser<Input, Output = Self>
    where
        Input: Stream<Token = char>,
    {
        let policy = PasswordPolicy::parser();
        let parser = policy.skip(char(':')).and(spaces().with(many1(letter())));
        parser.map(|(policy, password)| Self { policy, password })
    }

    fn part_one_valid(&self) -> bool {
        self.policy.part_one_validate(&self.password)
    }

    fn part_two_valid(&self) -> bool {
        self.policy.part_two_validate(&self.password)
    }
}

impl FromStr for PasswordDatabaseEntry {
    type Err = Error;
    fn from_str(input: &str) -> Result<Self> {
        let (p, remaining): (Self, &str) = Self::parser().easy_parse(input)?;
        require_no_remaining(remaining)?;
        Ok(p)
    }
}

fn part_one(entries: &[PasswordDatabaseEntry]) {
    println!(
        "{:?}",
        entries.iter().filter(|e| e.part_one_valid()).count()
    );
}

fn part_two(entries: &[PasswordDatabaseEntry]) {
    println!(
        "{:?}",
        entries.iter().filter(|e| e.part_two_valid()).count()
    );
}

fn main() -> Result<()> {
    let entries: Vec<PasswordDatabaseEntry> = parse_lines(io::stdin().lock())?;

    println!("Part 1");
    part_one(&entries);

    println!("Part 2");
    part_two(&entries);

    Ok(())
}
