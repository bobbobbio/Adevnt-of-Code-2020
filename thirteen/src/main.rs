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

#[derive(Debug, Clone)]
struct Bus {
    id: Option<u64>,
}

impl FromStr for Bus {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        match input {
            "x" => Ok(Self { id: None }),
            v => Ok(Self {
                id: Some(v.parse()?),
            }),
        }
    }
}

#[derive(Debug)]
struct Busses(Vec<Bus>);

impl FromStr for Busses {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        let parts = input.split(",");
        Ok(Self(parts.map(|p| Ok(p.parse()?)).collect::<Result<_>>()?))
    }
}

fn part_one(depart: u64, busses: &Busses) {
    let mut best_bus = None;
    let mut min_minutes = u64::MAX;
    for bus in &busses.0 {
        if let Some(id) = bus.id {
            let minutes = id - (depart % id);
            if minutes < min_minutes {
                min_minutes = minutes;
                best_bus = Some(bus.clone());
            }
        }
    }
    let best_bus_id = best_bus.unwrap().id.unwrap();
    println!("{}", best_bus_id * min_minutes)
}

fn check_time(t: u64, busses: &Busses) -> bool {
    for (i, b) in busses.0.iter().enumerate() {
        if let Some(id) = &b.id {
            let t_prime = t + i as u64;
            if t_prime % *id != 0 {
                return false;
            }
        }
    }
    true
}

fn part_two(busses: &Busses) {
    let mut found = HashSet::new();
    let mut t = 0;
    let mut incr = 1;
    let to_find = busses.0.iter().filter(|b| b.id.is_some()).count();

    loop {
        for (i, b) in busses.0.iter().enumerate() {
            if let Some(id) = &b.id {
                let t_prime = t + i as u64;
                if t_prime % *id == 0 && !found.contains(&i) {
                    found.insert(i);
                    incr = ::num::integer::lcm(incr, *id);
                }
            }
        }
        if found.len() >= to_find {
            break;
        }
        t += incr;
    }
    assert!(check_time(t, busses));

    println!("{}", t);
}

fn main() -> Result<()> {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    let depart: u64 = lines.next().unwrap()?.parse()?;
    let busses: Busses = lines.next().unwrap()?.parse()?;

    println!("Part 1");
    part_one(depart, &busses);

    println!("Part 2");
    part_two(&busses);

    Ok(())
}
