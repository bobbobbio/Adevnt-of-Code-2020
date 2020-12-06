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

fn part_one(numbers: &[u32]) {
    for (i, number1) in numbers.iter().enumerate() {
        for number2 in numbers[(i + 1)..].iter() {
            if number1 + number2 == 2020 {
                println!("{}", number1 * number2)
            }
        }
    }
}

fn part_two(numbers: &[u32]) {
    for (i, number1) in numbers.iter().enumerate() {
        for (j, number2) in numbers[(i + 1)..].iter().enumerate() {
            for number3 in numbers[(i + j + 2)..].iter() {
                if number1 + number2 + number3 == 2020 {
                    println!("{}", number1 * number2 * number3)
                }
            }
        }
    }
}

fn main() -> Result<()> {
    let numbers: Vec<u32> = parse_lines(io::stdin().lock())?;

    println!("Part 1");
    part_one(&numbers);

    println!("Part 2");
    part_two(&numbers);

    Ok(())
}
