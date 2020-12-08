use combine::parser::char::{digit, letter, spaces, string};
use combine::stream::{easy, position, Stream};
use combine::{attempt, eof, many1, sep_by1, EasyParser, Parser};
use std::collections::{HashMap, HashSet};
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

#[derive(Debug)]
struct Bag {
    name: BagName,
    can_contain: Vec<(usize, BagName)>,
}

impl Bag {
    fn can_contain(&self, bag_name: &BagName) -> bool {
        self.can_contain.iter().find(|v| &v.1 == bag_name).is_some()
    }
}

impl Bag {
    fn parser<Input>() -> impl Parser<Input, Output = Self>
    where
        Input: Stream<Token = char>,
    {
        let word = || many1(letter());
        let bags = || attempt(string("bags")).or(string("bag"));
        let bag_adj = || {
            word()
                .and(spaces().with(word()))
                .skip(spaces().with(bags()))
        };
        let bag_name = || {
            bag_adj().map(|(adj, color): (String, String)| BagName(format!("{} {}", adj, color)))
        };

        let number = many1(digit()).map(|n: String| n.parse::<usize>().unwrap());
        let bag_desc = number.and(spaces().with(bag_name()));
        let no_bags = string("no other bags").map(|_| vec![]);
        let bag_list = attempt(sep_by1(bag_desc, string(", "))).or(no_bags);
        let contains = spaces()
            .with(string("contain"))
            .with(spaces().with(bag_list));

        bag_name()
            .and(contains)
            .skip(string("."))
            .map(|(name, can_contain)| Self { name, can_contain })
    }
}

parser_from_str!(Bag);

fn build_bag_map(bags: &[Bag]) -> HashMap<&BagName, &Bag> {
    let mut map: HashMap<&BagName, &Bag> = HashMap::new();
    for bag in bags {
        assert!(map.insert(&bag.name, &bag).is_none());
    }
    map
}

fn bags_contain_bag<'a>(
    bag_map: &'a HashMap<&'a BagName, &'a Bag>,
    bag_name: &'a BagName,
) -> HashSet<&'a BagName> {
    let contain: HashSet<&BagName> = bag_map
        .values()
        .filter_map(|b| {
            if b.can_contain(&bag_name) {
                Some(&b.name)
            } else {
                None
            }
        })
        .collect();
    let mut more_contain: HashSet<&BagName> = HashSet::new();
    for &c in contain.iter() {
        more_contain = more_contain
            .union(&bags_contain_bag(bag_map, c))
            .cloned()
            .collect();
    }
    contain.union(&more_contain).cloned().collect()
}

fn part_one(bags: &[Bag]) {
    let bag_map = build_bag_map(bags);
    println!(
        "{}",
        bags_contain_bag(&bag_map, &BagName("shiny gold".into())).len()
    );
}

fn bag_must_contain<'a>(
    bag_map: &'a HashMap<&'a BagName, &'a Bag>,
    bag_name: &'a BagName,
) -> usize {
    let can_contain = &bag_map.get(bag_name).unwrap().can_contain;
    let mut total = 0;
    for (num, name) in can_contain {
        total += num * (1 + bag_must_contain(bag_map, name));
    }
    total
}

fn part_two(bags: &[Bag]) {
    let bag_map = build_bag_map(bags);
    println!(
        "{}",
        bag_must_contain(&bag_map, &BagName("shiny gold".into()))
    );
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
    let bags: Vec<Bag> = parse_lines(io::stdin().lock())?;

    println!("Part 1");
    part_one(&bags);

    println!("Part 2");
    part_two(&bags);

    Ok(())
}
