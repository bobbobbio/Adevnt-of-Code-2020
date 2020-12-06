use std::io::{self, BufRead};
use std::str::FromStr;

#[derive(Debug)]
enum Error {
    Io(io::Error),
    Parse(String),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
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

#[derive(PartialEq, Clone, Copy)]
enum Tile {
    Tree,
    Nothing,
}

impl FromStr for Tile {
    type Err = Error;
    fn from_str(input: &str) -> Result<Self> {
        match input {
            "." => Ok(Self::Nothing),
            "#" => Ok(Self::Tree),
            t => Err(Error::Parse(format!("bad tile {}", t))),
        }
    }
}

struct Row(Vec<Tile>);

impl FromStr for Row {
    type Err = Error;
    fn from_str(input: &str) -> Result<Self> {
        Ok(Self(
            input
                .chars()
                .map(|c| c.to_string().parse())
                .collect::<Result<_>>()?,
        ))
    }
}

struct Field {
    rows: Vec<Row>,
}

impl Field {
    fn height(&self) -> usize {
        self.rows.len()
    }

    fn get(&self, x: usize, y: usize) -> Tile {
        let row = &self.rows[y];
        row.0[x % row.0.len()]
    }
}

fn count_trees(field: &Field, slope: (usize, usize)) -> usize {
    let mut trees = 0;
    let (mut x, mut y) = (0, 0);
    while y < field.height() {
        if field.get(x, y) == Tile::Tree {
            trees += 1;
        }
        x += slope.0;
        y += slope.1;
    }
    trees
}

fn part_one(field: &Field) {
    println!("{}", count_trees(field, (3, 1)));
}

fn part_two(field: &Field) {
    let slopes = [(1, 1), (3, 1), (5, 1), (7, 1), (1, 2)];
    let answer: usize = slopes.iter().map(|&s| count_trees(field, s)).product();
    println!("{}", answer);
}

fn main() -> Result<()> {
    let rows: Vec<Row> = parse_lines(io::stdin().lock())?;
    let field = Field { rows };

    println!("Part 1");
    part_one(&field);

    println!("Part 2");
    part_two(&field);

    Ok(())
}
