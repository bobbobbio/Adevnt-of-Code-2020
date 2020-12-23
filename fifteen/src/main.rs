use combine::parser::char::{char, digit};
use combine::stream::Stream;
use combine::{many1, sep_by1, EasyParser, Parser};
use std::collections::HashMap;
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
struct Numbers(Vec<u64>);

impl Numbers {
    fn parser<Input>() -> impl Parser<Input, Output = Self>
    where
        Input: Stream<Token = char>,
    {
        let number = many1(digit()).map(|n: String| n.parse::<u64>().unwrap());
        sep_by1(number, char(','))
            .map(|v| Numbers(v))
            .skip(char('\n'))
    }
}

parser_from_str!(Numbers);

struct Game {
    history: HashMap<u64, u64>,
    current_turn: u64,
    last_number: Option<u64>,
}

impl Game {
    fn new() -> Self {
        Self {
            history: HashMap::new(),
            current_turn: 0,
            last_number: None,
        }
    }

    fn take_turn_with_number(&mut self, n: u64) {
        if let Some(n) = self.last_number {
            self.history.insert(n, self.current_turn);
        }
        self.current_turn += 1;
        self.last_number = Some(n);
    }

    fn starting_number(&mut self, n: u64) {
        self.take_turn_with_number(n);
    }

    fn turn(&mut self) {
        let number = if let Some(t) = self.history.get(&self.last_number.unwrap()) {
            self.current_turn - t
        } else {
            0
        };
        self.take_turn_with_number(number);
    }
}

fn print_game_number(numbers: &Numbers, turns: u64) {
    let mut game = Game::new();
    for n in &numbers.0 {
        game.starting_number(*n);
    }
    while game.current_turn < turns {
        game.turn();
    }
    println!("{}", game.last_number.unwrap());
}

fn part_one(numbers: &Numbers) {
    print_game_number(numbers, 2020);
}

fn part_two(numbers: &Numbers) {
    print_game_number(numbers, 30_000_000);
}

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let numbers: Numbers = input.parse()?;

    println!("Part 1");
    part_one(&numbers);

    println!("Part 2");
    part_two(&numbers);

    Ok(())
}
