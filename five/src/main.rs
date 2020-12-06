use std::io::{self, BufRead};
use std::num;
use std::ops::Range;
use std::str::FromStr;

#[derive(Debug)]
enum Error {
    ParseInt(num::ParseIntError),
    Io(io::Error),
    ExtraInput(String),
    Parse(String),
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

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Clone, Copy)]
enum RowDivider {
    Front,
    Back,
}

impl From<RowDivider> for BinaryDivider {
    fn from(r: RowDivider) -> Self {
        match r {
            RowDivider::Front => Self::Lower,
            RowDivider::Back => Self::Upper,
        }
    }
}

impl FromStr for RowDivider {
    type Err = Error;
    fn from_str(input: &str) -> Result<Self> {
        match input {
            "F" => Ok(Self::Front),
            "B" => Ok(Self::Back),
            c => Err(Error::Parse(format!("expected L/R: {}", c))),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum ColumnDivider {
    Left,
    Right,
}

impl From<ColumnDivider> for BinaryDivider {
    fn from(c: ColumnDivider) -> Self {
        match c {
            ColumnDivider::Left => Self::Lower,
            ColumnDivider::Right => Self::Upper,
        }
    }
}

impl FromStr for ColumnDivider {
    type Err = Error;
    fn from_str(input: &str) -> Result<Self> {
        match input {
            "L" => Ok(Self::Left),
            "R" => Ok(Self::Right),
            c => Err(Error::Parse(format!("expected L/R: {}", c))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum BinaryDivider {
    Lower,
    Upper,
}

struct BinarySearcher {
    range: Range<u32>,
}

impl BinarySearcher {
    fn new(size: u32) -> Self {
        Self { range: 0..size }
    }

    fn keep_lower(&mut self) {
        self.range.end -= (self.range.end - self.range.start) / 2;
    }

    fn keep_upper(&mut self) {
        self.range.start += (self.range.end - self.range.start) / 2;
    }

    fn divide(&mut self, divider: BinaryDivider) {
        match divider {
            BinaryDivider::Lower => self.keep_lower(),
            BinaryDivider::Upper => self.keep_upper(),
        }
    }

    fn answer(self) -> u32 {
        assert_eq!(self.range.end, self.range.start + 1);
        self.range.start
    }
}

#[derive(Debug)]
struct BoardingPass {
    row: Vec<RowDivider>,
    column: Vec<ColumnDivider>,
}

impl BoardingPass {
    const NUM_ROWS: u32 = 128;
    const NUM_COLUMNS: u32 = 8;

    fn seat_id(&self) -> u32 {
        self.row_number() * 8 + self.column_number()
    }

    fn binary_search<T: Clone + Into<BinaryDivider>>(input: &[T], size: u32) -> u32 {
        let mut searcher = BinarySearcher::new(size);
        for divider in input {
            searcher.divide(divider.clone().into());
        }
        searcher.answer()
    }

    fn row_number(&self) -> u32 {
        Self::binary_search(&self.row, Self::NUM_ROWS)
    }

    fn column_number(&self) -> u32 {
        Self::binary_search(&self.column, Self::NUM_COLUMNS)
    }
}

fn parse_chars<T: FromStr>(iter: impl Iterator<Item = char>) -> Vec<T> {
    let mut v = vec![];
    for c in iter {
        if let Ok(t) = c.to_string().parse() {
            v.push(t);
        } else {
            break;
        }
    }
    v
}

fn require_no_remaining(remaining: &str) -> Result<()> {
    if !remaining.is_empty() {
        Err(Error::ExtraInput(remaining.to_owned()))
    } else {
        Ok(())
    }
}

impl FromStr for BoardingPass {
    type Err = Error;
    fn from_str(input: &str) -> Result<Self> {
        let row = parse_chars(input.chars());
        let column = parse_chars(input.chars().skip(row.len()));
        let remaining = input
            .chars()
            .skip(row.len() + column.len())
            .collect::<String>();
        require_no_remaining(&remaining)?;

        Ok(Self { row, column })
    }
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

fn part_one(passes: &[BoardingPass]) {
    let max_seat_id = passes.iter().map(|p| p.seat_id()).max();
    println!("{}", max_seat_id.unwrap());
}

fn part_two(passes: &[BoardingPass]) {
    let mut seat_ids: Vec<_> = passes.iter().map(|p| p.seat_id()).collect();
    seat_ids.sort();

    let mut iter = seat_ids.iter().peekable();

    let mut holes = vec![];
    while let Some(value) = iter.next() {
        if let Some(next_value) = iter.peek() {
            if value + 1 != **next_value {
                holes.push(value + 1);
            }
        }
    }

    assert_eq!(holes.len(), 1);
    println!("{}", holes[0]);
}

fn main() -> Result<()> {
    let passes: Vec<BoardingPass> = parse_lines(io::stdin().lock())?;

    println!("Part 1");
    part_one(&passes);

    println!("Part 2");
    part_two(&passes);

    Ok(())
}
