use std::io::{self, BufRead};
use std::str::FromStr;
use std::{num, ops};

#[derive(Debug)]
enum Error {
    ParseInt(num::ParseIntError),
    Io(io::Error),
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

#[derive(Debug, Clone, Copy)]
struct Position {
    x: isize,
    y: isize,
}

impl Position {
    fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }

    fn manhattan_distance(&self) -> usize {
        (self.x.abs() + self.y.abs()) as usize
    }

    fn rotate(&self, around: Position, degrees: isize) -> Self {
        let start = Vector::from_positions(around, *self);
        let end = match degrees {
            90 | -270 => Vector::new(-start.y, start.x),
            -90 | 270 => Vector::new(start.y, -start.x),
            -180 | 180 => start * -1,
            d => panic!("bad degrees {}", d),
        };
        around + end
    }
}

#[derive(Clone, Copy)]
struct Vector {
    x: isize,
    y: isize,
}

impl Vector {
    const fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }

    fn from_positions(a: Position, b: Position) -> Self {
        Self {
            x: b.x - a.x,
            y: b.y - a.y,
        }
    }
}

impl ops::Mul<isize> for Vector {
    type Output = Self;
    fn mul(mut self, rhs: isize) -> Self {
        self *= rhs;
        self
    }
}

impl ops::MulAssign<isize> for Vector {
    fn mul_assign(&mut self, rhs: isize) {
        self.x = self.x * rhs;
        self.y = self.y * rhs;
    }
}

impl ops::Add<Vector> for Position {
    type Output = Position;

    fn add(mut self, other: Vector) -> Self {
        self += other;
        self
    }
}

impl ops::AddAssign<Vector> for Position {
    fn add_assign(&mut self, other: Vector) {
        self.x += other.x;
        self.y += other.y;
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    fn to_vector(&self) -> Vector {
        match self {
            Self::North => Vector::new(0, -1),
            Self::South => Vector::new(0, 1),
            Self::East => Vector::new(1, 0),
            Self::West => Vector::new(-1, 0),
        }
    }

    fn turn(&self, degrees: isize) -> Self {
        let directions = &[Self::North, Self::East, Self::South, Self::West];
        let mut position = directions.iter().position(|d| d == self).unwrap();
        position = position.wrapping_add((degrees / 90) as usize) % directions.len();
        directions[position]
    }
}

#[derive(Debug, Clone)]
enum Move {
    Forward(usize),
    Right(usize),
    Left(usize),
    Direction(Direction, usize),
}

impl FromStr for Move {
    type Err = Error;
    fn from_str(input: &str) -> Result<Self> {
        let (letter, number) = input.split_at(1);
        let number = number.parse()?;
        match letter {
            "F" => Ok(Self::Forward(number)),
            "R" => Ok(Self::Right(number)),
            "L" => Ok(Self::Left(number)),
            "N" => Ok(Self::Direction(Direction::North, number)),
            "S" => Ok(Self::Direction(Direction::South, number)),
            "E" => Ok(Self::Direction(Direction::East, number)),
            "W" => Ok(Self::Direction(Direction::West, number)),
            c => Err(Error::Parse(format!("unknown move {}", c))),
        }
    }
}

struct Ship {
    position: Position,
    direction: Direction,
}

impl Ship {
    fn new() -> Self {
        Self {
            position: Position::new(0, 0),
            direction: Direction::East,
        }
    }
    fn apply(&mut self, m: Move) {
        match m {
            Move::Forward(a) => self.position += self.direction.to_vector() * (a as isize),
            Move::Right(d) => self.direction = self.direction.turn(d as isize),
            Move::Left(d) => self.direction = self.direction.turn(-(d as isize)),
            Move::Direction(d, a) => self.position += d.to_vector() * (a as isize),
        }
    }
}

#[derive(Debug)]
struct Ship2 {
    position: Position,
    waypoint: Position,
}

impl Ship2 {
    fn new() -> Self {
        Self {
            position: Position::new(0, 0),
            waypoint: Position::new(10, -1),
        }
    }
    fn apply(&mut self, m: Move) {
        match m {
            Move::Forward(a) => {
                let v = Vector::from_positions(self.position, self.waypoint) * (a as isize);
                self.position += v;
                self.waypoint += v;
            }
            Move::Right(d) => self.waypoint = self.waypoint.rotate(self.position, d as isize),
            Move::Left(d) => self.waypoint = self.waypoint.rotate(self.position, -(d as isize)),
            Move::Direction(d, a) => self.waypoint += d.to_vector() * (a as isize),
        }
    }
}

fn part_one(moves: &[Move]) {
    let mut ship = Ship::new();
    for m in moves {
        ship.apply(m.clone());
    }
    println!("{}", ship.position.manhattan_distance());
}

fn part_two(moves: &[Move]) {
    let mut ship = Ship2::new();
    for m in moves {
        ship.apply(m.clone());
    }
    println!("{}", ship.position.manhattan_distance());
}

fn main() -> Result<()> {
    let moves: Vec<Move> = parse_lines(io::stdin().lock())?;

    println!("Part 1");
    part_one(&moves);

    println!("Part 2");
    part_two(&moves);

    Ok(())
}
