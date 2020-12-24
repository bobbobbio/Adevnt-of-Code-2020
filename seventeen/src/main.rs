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
    Active,
    Inactive,
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Active => write!(f, "#"),
            Self::Inactive => write!(f, "."),
        }
    }
}

impl FromStr for Cell {
    type Err = Error;
    fn from_str(input: &str) -> Result<Self> {
        match input {
            "#" => Ok(Self::Active),
            "." => Ok(Self::Inactive),
            c => Err(Error::Parse(format!("invalid Cell {}", c))),
        }
    }
}

#[derive(Debug, Clone)]
struct Row(Vec<Cell>);

impl Row {
    fn new(width: usize) -> Self {
        Self(vec![Cell::Inactive; width])
    }

    fn width(&self) -> usize {
        self.0.len()
    }

    fn get_cell(&self, position: Position) -> Cell {
        self.0[position.x]
    }

    fn set_cell(&mut self, position: Position, cell: Cell) {
        self.0[position.x] = cell;
    }

    fn is_position_valid(&self, position: Position) -> bool {
        position.x < self.width()
    }

    fn cells<'a>(&'a self) -> impl Iterator<Item = Cell> + 'a {
        self.0.iter().copied()
    }

    fn positioned_cells<'a>(&'a self) -> impl Iterator<Item = (Position, Cell)> + 'a {
        self.0
            .iter()
            .enumerate()
            .map(|(x, c)| (Position::new_x(x), c.clone()))
    }

    fn grow(&mut self) {
        self.0.insert(0, Cell::Inactive);
        self.0.push(Cell::Inactive);
    }
}

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

#[derive(Debug, Clone)]
struct Plane {
    rows: Vec<Row>,
}

impl Plane {
    fn new(width: usize, height: usize) -> Self {
        Self {
            rows: vec![Row::new(width); height],
        }
    }

    fn with_rows(rows: Vec<Row>) -> Self {
        Self { rows }
    }

    fn width(&self) -> usize {
        self.rows[0].width()
    }

    fn height(&self) -> usize {
        self.rows.len()
    }

    fn get_cell(&self, position: Position) -> Cell {
        self.rows[position.y].get_cell(position)
    }

    fn set_cell(&mut self, position: Position, cell: Cell) {
        self.rows[position.y].set_cell(position, cell);
    }

    fn is_position_valid(&self, position: Position) -> bool {
        if position.y >= self.height() {
            return false;
        }

        let row = &self.rows[position.y];
        row.is_position_valid(position)
    }

    fn cells<'a>(&'a self) -> impl Iterator<Item = Cell> + 'a {
        self.rows.iter().map(|r| r.cells()).flatten()
    }

    fn positioned_cells<'a>(&'a self) -> impl Iterator<Item = (Position, Cell)> + 'a {
        self.rows
            .iter()
            .enumerate()
            .map(|(y, r)| r.positioned_cells().map(move |(pos, c)| (pos.with_y(y), c)))
            .flatten()
    }

    fn grow(&mut self) {
        for r in &mut self.rows {
            r.grow();
        }
        let width = self.rows[0].width();
        self.rows.insert(0, Row::new(width));
        self.rows.push(Row::new(width));
    }
}

#[derive(Debug, Clone, Copy)]
struct Position {
    x: usize,
    y: usize,
    z: usize,
    w: usize,
}

impl Position {
    fn new_x(x: usize) -> Self {
        Self {
            x,
            y: 0,
            z: 0,
            w: 0,
        }
    }

    fn with_y(mut self, y: usize) -> Self {
        self.y = y;
        self
    }

    fn with_z(mut self, z: usize) -> Self {
        self.z = z;
        self
    }

    fn with_w(mut self, w: usize) -> Self {
        self.w = w;
        self
    }
}

#[derive(Clone, Copy)]
struct Vector {
    x: isize,
    y: isize,
    z: isize,
    w: isize,
}

impl Vector {
    fn new_x_y(x: isize, y: isize) -> Self {
        Self { x, y, z: 0, w: 0 }
    }

    fn new_z(z: isize) -> Self {
        Self {
            x: 0,
            y: 0,
            z,
            w: 0,
        }
    }

    fn new_w(w: isize) -> Self {
        Self {
            x: 0,
            y: 0,
            z: 0,
            w,
        }
    }

    fn with_z(mut self, z: isize) -> Self {
        self.z = z;
        self
    }

    fn with_w(mut self, w: isize) -> Self {
        self.w = w;
        self
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
        self.z = self.z.wrapping_add(other.z as usize);
        self.w = self.w.wrapping_add(other.w as usize);
    }
}

struct Space {
    planes: Vec<Plane>,
}

impl fmt::Display for Space {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for plane in &self.planes {
            for row in &plane.rows {
                for c in &row.0 {
                    write!(f, "{}", c)?;
                }
                write!(f, "\n")?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

fn all_2d_directions() -> Vec<Vector> {
    let mut d = vec![];
    for y in -1..=1 {
        for x in -1..=1 {
            if x == 0 && y == 0 {
                continue;
            }
            d.push(Vector::new_x_y(x, y));
        }
    }
    d
}

fn all_3d_directions() -> Vec<Vector> {
    let mut d = vec![];
    for z in -1..=1 {
        for inner in all_2d_directions().into_iter() {
            d.push(inner.with_z(z));
        }
        if z != 0 {
            d.push(Vector::new_z(z));
        }
    }
    d
}

fn all_4d_directions() -> Vec<Vector> {
    let mut d = vec![];
    for w in -1..=1 {
        for inner in all_3d_directions().into_iter() {
            d.push(inner.with_w(w));
        }
        if w != 0 {
            d.push(Vector::new_w(w));
        }
    }
    d
}

impl Space {
    fn with_planes(planes: Vec<Plane>) -> Self {
        Self { planes }
    }

    fn new(width: usize, height: usize, depth: usize) -> Self {
        Self::with_planes(vec![Plane::new(width, height); depth])
    }

    fn width(&self) -> usize {
        self.planes[0].width()
    }

    fn height(&self) -> usize {
        self.planes[0].height()
    }

    fn depth(&self) -> usize {
        self.planes.len()
    }

    fn get_cell(&self, position: Position) -> Cell {
        self.planes[position.z].get_cell(position)
    }

    fn is_position_valid(&self, position: Position) -> bool {
        if position.z >= self.planes.len() {
            return false;
        }

        let plane = &self.planes[position.z];
        plane.is_position_valid(position)
    }

    fn adjacent(&self, position: Position) -> Vec<Cell> {
        let mut vec = vec![];
        for v in all_3d_directions().into_iter() {
            let mut position = position.clone();
            position += v;

            if self.is_position_valid(position) {
                vec.push(self.get_cell(position));
            }
        }
        vec
    }

    fn cells<'a>(&'a self) -> impl Iterator<Item = Cell> + 'a {
        self.planes.iter().map(|p| p.cells()).flatten()
    }

    fn positioned_cells<'a>(&'a self) -> impl Iterator<Item = (Position, Cell)> + 'a {
        self.planes
            .iter()
            .enumerate()
            .map(|(z, p)| p.positioned_cells().map(move |(pos, c)| (pos.with_z(z), c)))
            .flatten()
    }

    fn set_cell(&mut self, position: Position, cell: Cell) {
        self.planes[position.z].set_cell(position, cell);
    }

    fn apply_changes(&mut self, changes: Vec<(Position, Cell)>) {
        for (position, c) in changes.into_iter() {
            self.set_cell(position, c);
        }
    }

    fn tick(&mut self) {
        self.grow();

        let mut changes = vec![];
        for (position, cell) in self.positioned_cells() {
            let active_neighbors = self
                .adjacent(position)
                .into_iter()
                .filter(|&c| c == Cell::Active)
                .count();

            match cell {
                Cell::Active if active_neighbors < 2 || active_neighbors > 3 => {
                    changes.push((position, Cell::Inactive))
                }
                Cell::Inactive if active_neighbors == 3 => changes.push((position, Cell::Active)),
                _ => {}
            }
        }
        self.apply_changes(changes);
    }

    fn count_active(&self) -> usize {
        self.cells().filter(|&c| c == Cell::Active).count()
    }

    fn grow(&mut self) {
        for p in &mut self.planes {
            p.grow();
        }

        let width = self.planes[0].width();
        let height = self.planes[0].height();
        self.planes.insert(0, Plane::new(width, height));
        self.planes.push(Plane::new(width, height));
    }
}

fn run_space(rows: &[Row]) {
    let mut space = Space::with_planes(vec![Plane::with_rows(rows.to_owned())]);
    for _ in 0..6 {
        space.tick();
    }
    println!("{}", space.count_active());
}

fn part_one(rows: &[Row]) {
    run_space(rows);
}

struct HyperSpace {
    spaces: Vec<Space>,
}

impl HyperSpace {
    fn with_spaces(spaces: Vec<Space>) -> Self {
        Self { spaces }
    }

    fn get_cell(&self, position: Position) -> Cell {
        self.spaces[position.w].get_cell(position)
    }

    fn is_position_valid(&self, position: Position) -> bool {
        if position.w >= self.spaces.len() {
            return false;
        }

        let space = &self.spaces[position.w];
        space.is_position_valid(position)
    }

    fn adjacent(&self, position: Position) -> Vec<Cell> {
        let mut vec = vec![];
        for v in all_4d_directions().into_iter() {
            let mut position = position.clone();
            position += v;

            if self.is_position_valid(position) {
                vec.push(self.get_cell(position));
            }
        }
        vec
    }

    fn cells<'a>(&'a self) -> impl Iterator<Item = Cell> + 'a {
        self.spaces.iter().map(|p| p.cells()).flatten()
    }

    fn positioned_cells<'a>(&'a self) -> impl Iterator<Item = (Position, Cell)> + 'a {
        self.spaces
            .iter()
            .enumerate()
            .map(|(w, s)| s.positioned_cells().map(move |(pos, c)| (pos.with_w(w), c)))
            .flatten()
    }

    fn set_cell(&mut self, position: Position, cell: Cell) {
        self.spaces[position.w].set_cell(position, cell);
    }

    fn apply_changes(&mut self, changes: Vec<(Position, Cell)>) {
        for (position, c) in changes.into_iter() {
            self.set_cell(position, c);
        }
    }

    fn tick(&mut self) {
        self.grow();

        let mut changes = vec![];
        for (position, cell) in self.positioned_cells() {
            let active_neighbors = self
                .adjacent(position)
                .into_iter()
                .filter(|&c| c == Cell::Active)
                .count();

            match cell {
                Cell::Active if active_neighbors < 2 || active_neighbors > 3 => {
                    changes.push((position, Cell::Inactive))
                }
                Cell::Inactive if active_neighbors == 3 => changes.push((position, Cell::Active)),
                _ => {}
            }
        }
        self.apply_changes(changes);
    }

    fn count_active(&self) -> usize {
        self.cells().filter(|&c| c == Cell::Active).count()
    }

    fn grow(&mut self) {
        for s in &mut self.spaces {
            s.grow();
        }

        let width = self.spaces[0].width();
        let height = self.spaces[0].height();
        let depth = self.spaces[0].depth();
        self.spaces.insert(0, Space::new(width, height, depth));
        self.spaces.push(Space::new(width, height, depth));
    }
}

fn run_hyperspace(rows: &[Row]) {
    let mut hyper_space =
        HyperSpace::with_spaces(vec![Space::with_planes(vec![Plane::with_rows(
            rows.to_owned(),
        )])]);
    for _ in 0..6 {
        hyper_space.tick();
    }
    println!("{}", hyper_space.count_active());
}

fn part_two(rows: &[Row]) {
    run_hyperspace(rows);
}

fn main() -> Result<()> {
    let rows: Vec<Row> = parse_lines(io::stdin().lock())?;

    println!("Part 1");
    part_one(&rows);

    println!("Part 2");
    part_two(&rows);

    Ok(())
}
