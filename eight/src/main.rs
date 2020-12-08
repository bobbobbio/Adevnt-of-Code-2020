use combine::parser::char::{char, digit, spaces, string};
use combine::stream::{easy, position, Stream};
use combine::{attempt, eof, many1, EasyParser, Parser};
use std::collections::HashSet;
use std::io::{self, BufRead};
use std::num;
use std::str::FromStr;

macro_rules! parser_from_str {
    ($s:ident) => {
        impl FromStr for $s {
            type Err = Error;
            fn from_str(input: &str) -> Result<Self> {
                let (p, _): (Self, _) = Self::parser()
                    .skip(eof())
                    .easy_parse(position::Stream::new(input))?;
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

impl From<easy::Errors<char, &str, position::SourcePosition>> for Error {
    fn from(e: easy::Errors<char, &str, position::SourcePosition>) -> Self {
        Self::ParseError(e.to_string())
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Eq, Hash)]
struct BagName(String);

#[derive(Clone, Debug)]
enum Instruction {
    Noop(i32),
    Acc(i32),
    Jump(i32),
}

impl Instruction {
    fn parser<Input>() -> impl Parser<Input, Output = Self>
    where
        Input: Stream<Token = char>,
    {
        let number = || {
            attempt(char('+'))
                .or(char('-'))
                .and(many1(digit()))
                .map(|(c, n): (char, String)| {
                    let num = n.parse::<i32>().unwrap();
                    num * if c == '-' { -1 } else { 1 }
                })
        };
        let acc = string("acc")
            .and(spaces().with(number()))
            .map(|(_, n)| Self::Acc(n));
        let jump = string("jmp")
            .and(spaces().with(number()))
            .map(|(_, n)| Self::Jump(n));
        let noop = string("nop")
            .and(spaces().with(number()))
            .map(|(_, n)| Self::Noop(n));
        attempt(acc).or(attempt(jump)).or(noop)
    }
}

parser_from_str!(Instruction);

struct Machine<'a> {
    accumulator: usize,
    instruction_pointer: usize,
    visited_addresses: HashSet<usize>,
    instructions: &'a [Instruction],
}

impl<'a> Machine<'a> {
    fn new(instructions: &'a [Instruction]) -> Self {
        Self {
            accumulator: 0,
            instruction_pointer: 0,
            visited_addresses: HashSet::new(),
            instructions,
        }
    }

    fn run_one(&mut self) {
        self.visited_addresses.insert(self.instruction_pointer);

        let instr = &self.instructions[self.instruction_pointer];
        match instr {
            Instruction::Acc(v) => {
                self.accumulator = self.accumulator.wrapping_add(*v as usize);
                self.instruction_pointer += 1;
            }
            Instruction::Jump(v) => {
                self.instruction_pointer = self.instruction_pointer.wrapping_add(*v as usize);
            }
            Instruction::Noop(_) => {
                self.instruction_pointer += 1;
            }
        }
    }

    fn run_until_looped(&mut self) -> bool {
        loop {
            if self.visited_addresses.contains(&self.instruction_pointer) {
                return false;
            }
            if self.instruction_pointer == self.instructions.len() {
                return true;
            }
            self.run_one();
        }
    }
}

fn part_one(instructions: &[Instruction]) {
    let mut machine = Machine::new(instructions);
    machine.run_until_looped();
    println!("{}", machine.accumulator);
}

fn part_two_inner(instructions: &[Instruction]) -> usize {
    fn flip(inst: &mut Instruction) {
        match inst {
            Instruction::Jump(v) => *inst = Instruction::Noop(*v),
            Instruction::Noop(v) => *inst = Instruction::Jump(*v),
            _ => {}
        }
    }
    let mut instructions_copy = instructions.to_vec();
    for i in 0..instructions.len() {
        flip(&mut instructions_copy[i]);
        let mut machine = Machine::new(&instructions_copy);
        if machine.run_until_looped() {
            return machine.accumulator;
        }
        flip(&mut instructions_copy[i]);
    }
    panic!();
}

fn part_two(instructions: &[Instruction]) {
    println!("{}", part_two_inner(instructions));
}

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

fn main() -> Result<()> {
    let instructions: Vec<Instruction> = parse_lines(io::stdin().lock())?;

    println!("Part 1");
    part_one(&instructions);

    println!("Part 2");
    part_two(&instructions);

    Ok(())
}
