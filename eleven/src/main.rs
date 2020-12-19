use std::io::{self, BufRead};
use std::str::FromStr;
use std::{fmt, num, ops};

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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Cell {
    EmptySeat,
    OccupiedSeat,
    Floor,
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySeat => write!(f, "L"),
            Self::OccupiedSeat => write!(f, "#"),
            Self::Floor => write!(f, "."),
        }
    }
}

impl FromStr for Cell {
    type Err = Error;
    fn from_str(input: &str) -> Result<Self> {
        match input {
            "L" => Ok(Self::EmptySeat),
            "#" => Ok(Self::OccupiedSeat),
            "." => Ok(Self::Floor),
            c => Err(Error::Parse(format!("invalid Cell {}", c))),
        }
    }
}

#[derive(Debug, Clone)]
struct Row(Vec<Cell>);

fn parse_chars<T: FromStr>(iter: impl Iterator<Item = char>) -> Result<Vec<T>>
where
    Error: From<<T as FromStr>::Err>,
{
    iter.map(|c| Ok(c.to_string().parse()?)).collect()
}

impl FromStr for Row {
    type Err = Error;
    fn from_str(input: &str) -> Result<Self> {
        Ok(Self(parse_chars(input.chars())?))
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

#[derive(Debug, Clone, Copy)]
struct Position {
    x: usize,
    y: usize,
}

impl Position {
    fn new(x: usize, y: usize) -> Self {
        Self { x, y }
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
        self.x = self.x.wrapping_add(other.x as usize);
        self.y = self.y.wrapping_add(other.y as usize);
    }
}

struct Board {
    rows: Vec<Row>,
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in &self.rows {
            for c in &row.0 {
                write!(f, "{}", c)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

const ALL_DIRECTIONS: &'static [Vector] = &[
    Vector::new(-1, -1),
    Vector::new(-1, 0),
    Vector::new(-1, 1),
    Vector::new(0, -1),
    Vector::new(0, 1),
    Vector::new(1, -1),
    Vector::new(1, 0),
    Vector::new(1, 1),
];

impl Board {
    fn new(rows: Vec<Row>) -> Self {
        Self { rows }
    }

    fn try_get_cell(&self, position: Position) -> Option<Cell> {
        if self.is_position_valid(position) {
            Some(self.rows[position.y].0[position.x])
        } else {
            None
        }
    }

    fn get_cell(&self, position: Position) -> Cell {
        self.rows[position.y].0[position.x]
    }

    fn is_position_valid(&self, position: Position) -> bool {
        if position.y >= self.rows.len() {
            return false;
        }

        let row = &self.rows[position.y];

        if position.x >= row.0.len() {
            return false;
        }
        true
    }

    fn adjacent(&self, position: Position) -> Vec<Cell> {
        let mut vec = vec![];
        for &v in ALL_DIRECTIONS {
            let mut position = position.clone();
            position += v;

            if self.is_position_valid(position) {
                vec.push(self.get_cell(position))
            }
        }
        vec
    }

    fn visible(&self, position: Position) -> Vec<Cell> {
        let mut vec = vec![];
        for &v in ALL_DIRECTIONS {
            let mut position = position.clone();
            position += v;

            loop {
                let maybe_cell = self.try_get_cell(position);
                if maybe_cell.is_none() || maybe_cell.unwrap() != Cell::Floor {
                    break;
                }
                position += v;
            }

            if self.is_position_valid(position) {
                vec.push(self.get_cell(position));
            }
        }
        vec
    }

    fn cells<'a>(&'a self) -> impl Iterator<Item = Cell> + 'a {
        self.rows.iter().map(|r| r.0.iter()).flatten().copied()
    }

    fn positioned_cells<'a>(&'a self) -> impl Iterator<Item = (Position, Cell)> + 'a {
        let row_width = self.rows[0].0.len();
        self.cells()
            .enumerate()
            .map(move |(i, c)| (Position::new(i % row_width, i / row_width), c))
    }

    fn set_cell(&mut self, pos: Position, cell: Cell) {
        self.rows[pos.y].0[pos.x] = cell;
    }

    fn apply_changes(&mut self, changes: Vec<(Position, Cell)>) {
        for (pos, c) in changes.into_iter() {
            self.set_cell(pos, c);
        }
    }

    fn tick(&mut self) {
        let mut changes = vec![];
        for (pos, cell) in self.positioned_cells() {
            let occupied = self
                .adjacent(pos)
                .into_iter()
                .filter(|&c| c == Cell::OccupiedSeat)
                .count();

            match cell {
                Cell::EmptySeat if occupied == 0 => changes.push((pos, Cell::OccupiedSeat)),
                Cell::OccupiedSeat if occupied >= 4 => changes.push((pos, Cell::EmptySeat)),
                _ => {}
            }
        }
        self.apply_changes(changes);
    }

    fn tick2(&mut self) {
        let mut changes = vec![];
        for (pos, cell) in self.positioned_cells() {
            let occupied = self
                .visible(pos)
                .into_iter()
                .filter(|&c| c == Cell::OccupiedSeat)
                .count();

            match cell {
                Cell::EmptySeat if occupied == 0 => changes.push((pos, Cell::OccupiedSeat)),
                Cell::OccupiedSeat if occupied >= 5 => changes.push((pos, Cell::EmptySeat)),
                _ => {}
            }
        }
        self.apply_changes(changes);
    }

    fn count_occupied_seats(&self) -> usize {
        self.cells().filter(|&c| c == Cell::OccupiedSeat).count()
    }
}

fn run_board<F: Fn(&mut Board)>(rows: &[Row], tick: F) {
    let mut board = Board::new(rows.to_owned());
    let mut last_occupied_seats = board.count_occupied_seats();
    loop {
        tick(&mut board);

        let occupied_seats = board.count_occupied_seats();

        if last_occupied_seats == occupied_seats {
            break;
        }
        last_occupied_seats = occupied_seats;
    }
    println!("{}", last_occupied_seats);
}

fn part_one(rows: &[Row]) {
    run_board(rows, Board::tick);
}

fn part_two(rows: &[Row]) {
    run_board(rows, Board::tick2);
}

fn main() -> Result<()> {
    let rows: Vec<Row> = parse_lines(io::stdin().lock())?;

    println!("Part 1");
    part_one(&rows);

    println!("Part 2");
    part_two(&rows);

    Ok(())
}
