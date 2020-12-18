use std::collections::HashSet;
use std::io::{self, BufRead};
use std::num;
use std::str::FromStr;

#[derive(Debug)]
enum Error {
    ParseInt(num::ParseIntError),
    Io(io::Error),
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

fn all_sums(input: &[u64]) -> HashSet<u64> {
    let mut all = HashSet::new();
    for (i, a) in input.iter().enumerate() {
        for b in &input[(i + 1)..] {
            all.insert(a + b);
        }
    }
    all
}

const PREAMBLE_SIZE: usize = 25;

fn is_valid(preamble: &[u64], value: u64) -> bool {
    let all_sums = all_sums(preamble);
    all_sums.contains(&value)
}

fn find_first_invalid(numbers: &[u64]) -> Option<u64> {
    for start in 0..(numbers.len() - PREAMBLE_SIZE) {
        let end = start + PREAMBLE_SIZE;
        let preamble = &numbers[start..end];
        let v = numbers[end];
        if !is_valid(preamble, v) {
            return Some(v);
        }
    }
    None
}

fn part_one(numbers: &[u64]) {
    println!("{}", find_first_invalid(numbers).unwrap());
}

fn find_range_summing_to(numbers: &[u64], value: u64) -> Option<&[u64]> {
    for i in 0..numbers.len() {
        for j in (i + 2)..numbers.len() {
            let range = &numbers[i..j];
            if range.iter().sum::<u64>() == value {
                return Some(range);
            }
        }
    }
    None
}

fn part_two(numbers: &[u64]) {
    let part_one_answer = find_first_invalid(numbers).unwrap();
    let range = find_range_summing_to(numbers, part_one_answer).unwrap();
    let min = range.iter().fold(u64::MAX, |a, &b| a.min(b));
    let max = range.iter().fold(0, |a, &b| a.max(b));
    println!("{}", min + max);
}

fn main() -> Result<()> {
    let numbers: Vec<u64> = parse_lines(io::stdin().lock())?;

    println!("Part 1");
    part_one(&numbers);

    println!("Part 2");
    part_two(&numbers);

    Ok(())
}
