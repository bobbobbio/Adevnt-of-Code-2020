use combine::attempt;
use combine::parser::char::{char, digit, string};
use combine::parser::repeat::count_min_max;
use combine::stream::Stream;
use combine::{many1, sep_end_by1, EasyParser, Parser};
use std::collections::HashMap;
use std::convert::TryInto;
use std::io::{self, Read};
use std::str::FromStr;
use std::{fmt, num};

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

#[derive(Clone, Copy, PartialEq, Eq)]
enum MaskValue {
    One,
    Zero,
    Floating,
}

impl fmt::Debug for MaskValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::One => "1",
                Self::Zero => "0",
                Self::Floating => "X",
            }
        )
    }
}

impl MaskValue {
    fn parser<Input>() -> impl Parser<Input, Output = Self>
    where
        Input: Stream<Token = char>,
    {
        char('1')
            .map(|_| Self::One)
            .or(char('0').map(|_| Self::Zero))
            .or(char('X').map(|_| Self::Floating))
    }
}

parser_from_str!(MaskValue);

#[derive(Clone)]
struct Mask(Box<[MaskValue; 36]>);

impl fmt::Debug for Mask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for v in &*self.0 {
            write!(f, "{:?}", v)?;
        }
        Ok(())
    }
}

impl Mask {
    fn one_mask(&self) -> u64 {
        let mut mask = 0;
        for i in self.indexes(MaskValue::One) {
            mask |= 1 << i;
        }
        mask
    }

    fn zero_mask(&self) -> u64 {
        let mut mask = !0;
        for i in self.indexes(MaskValue::Zero) {
            mask &= !(1 << i);
        }
        mask
    }

    fn apply_v1(&self, mut value: u64) -> u64 {
        value |= self.one_mask();
        value &= self.zero_mask();
        value
    }

    fn with(&self, i: usize, value: MaskValue) -> Self {
        let mut new = self.clone();
        new.0[new.0.len() - 1 - i] = value;
        new
    }

    fn indexes<'a>(&'a self, value: MaskValue) -> impl Iterator<Item = usize> + 'a {
        self.0
            .iter()
            .rev()
            .enumerate()
            .filter_map(move |(i, b)| if *b == value { Some(i) } else { None })
    }

    fn apply_v2(&self, mut addr: u64) -> Vec<u64> {
        addr |= self.one_mask();

        let mut addresses = vec![];

        let first_floating = self.indexes(MaskValue::Floating).next();
        if let Some(i) = first_floating {
            let new_mask = self.with(i, MaskValue::Zero);

            let new_addr = addr | 1 << i;
            addresses.extend(new_mask.apply_v2(new_addr));

            let new_addr = addr & !(1 << i);
            addresses.extend(new_mask.apply_v2(new_addr));
        } else {
            addresses.push(addr);
        }

        addresses
    }
}

impl Mask {
    fn parser<Input>() -> impl Parser<Input, Output = Self>
    where
        Input: Stream<Token = char>,
    {
        count_min_max(36, 36, MaskValue::parser())
            .map(|v: Vec<_>| Self(v.into_boxed_slice().try_into().unwrap()))
    }
}

parser_from_str!(Mask);

#[derive(Debug)]
struct Program {
    mask: Mask,
    writes: Vec<(u64, u64)>,
}

impl Program {
    fn parser<Input>() -> impl Parser<Input, Output = Self>
    where
        Input: Stream<Token = char>,
    {
        let mask = string("mask = ").with(Mask::parser()).skip(char('\n'));

        let number = || many1(digit()).map(|v: String| v.parse::<u64>().unwrap());
        let write = string("mem[")
            .with(number())
            .and(string("] = ").with(number()));
        let writes = sep_end_by1(attempt(write), char('\n'));
        mask.and(writes).map(|(mask, writes)| Self { mask, writes })
    }
}

parser_from_str!(Program);

#[derive(Debug)]
struct ProgramCollection(Vec<Program>);

impl ProgramCollection {
    fn parser<Input>() -> impl Parser<Input, Output = Self>
    where
        Input: Stream<Token = char>,
    {
        let program = Program::parser();
        many1(program).map(|v| Self(v))
    }
}

parser_from_str!(ProgramCollection);

struct Machine {
    memory: HashMap<u64, u64>,
}

impl Machine {
    fn new() -> Self {
        Self {
            memory: HashMap::new(),
        }
    }

    fn run_v1(&mut self, program: &Program) {
        for (addr, value) in &program.writes {
            let cell = self.memory.entry(*addr).or_insert(0);
            *cell = program.mask.apply_v1(*value);
        }
    }

    fn run_v2(&mut self, program: &Program) {
        for (addr, value) in &program.writes {
            for new_addr in program.mask.apply_v2(*addr).into_iter() {
                let cell = self.memory.entry(new_addr).or_insert(0);
                *cell = *value;
            }
        }
    }

    fn sum_memory(&self) -> u64 {
        self.memory.values().copied().sum()
    }
}

fn find_answer<F: for<'a> Fn(&'a mut Machine, &'a Program)>(programs: &ProgramCollection, run: F) {
    let mut machine = Machine::new();
    for program in &programs.0 {
        run(&mut machine, program);
    }
    let answer = machine.sum_memory();
    println!("{}", answer);
}

fn part_one(programs: &ProgramCollection) {
    find_answer(programs, Machine::run_v1);
}

fn part_two(programs: &ProgramCollection) {
    find_answer(programs, Machine::run_v2);
}

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let programs: ProgramCollection = input.parse()?;

    println!("Part 1");
    part_one(&programs);

    println!("Part 2");
    part_two(&programs);

    Ok(())
}
