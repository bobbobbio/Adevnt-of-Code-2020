use combine::parser::char::{char, digit, letter, space, string};
use combine::stream::Stream;
use combine::{attempt, many1, sep_by1, sep_end_by1, EasyParser, Parser};
use std::collections::HashSet;
use std::io::{self, Read};
use std::num;
use std::ops::RangeInclusive;
use std::str::FromStr;

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

fn number<Input>() -> impl Parser<Input, Output = u64>
where
    Input: Stream<Token = char>,
{
    many1(digit()).map(|s: String| s.parse::<u64>().unwrap())
}

#[derive(Debug)]
struct Rule {
    name: String,
    rules: Vec<RangeInclusive<u64>>,
}

impl Rule {
    fn validate_ticket_value(&self, value: u64) -> bool {
        self.rules
            .iter()
            .any(|r| value >= *r.start() && value <= *r.end())
    }
}

impl Rule {
    fn parser<Input>() -> impl Parser<Input, Output = Self>
    where
        Input: Stream<Token = char>,
    {
        let name = many1(letter().or(space()));
        let range = number().skip(char('-')).and(number()).map(|(s, e)| s..=e);
        let rules = sep_by1(range, string(" or "));
        let separator = string(": ");
        name.skip(separator)
            .and(rules)
            .map(|(name, rules)| Self { name, rules })
    }
}

parser_from_str!(Rule);

#[derive(Debug)]
struct Ticket(Vec<u64>);

impl Ticket {
    fn invalid_values(&self, rules: &Rules) -> Vec<u64> {
        self.0
            .iter()
            .filter(|v| !rules.validate_ticket_value(**v))
            .copied()
            .collect()
    }

    fn valid(&self, rules: &Rules) -> bool {
        self.0.iter().all(|v| rules.validate_ticket_value(*v))
    }
}

impl Ticket {
    fn parser<Input>() -> impl Parser<Input, Output = Self>
    where
        Input: Stream<Token = char>,
    {
        sep_by1(number(), char(',')).map(Self)
    }
}

parser_from_str!(Ticket);

#[derive(Debug)]
struct Rules(Vec<Rule>);

impl Rules {
    fn validate_ticket_value(&self, value: u64) -> bool {
        self.0.iter().any(|r| r.validate_ticket_value(value))
    }
}

impl Rules {
    fn parser<Input>() -> impl Parser<Input, Output = Self>
    where
        Input: Stream<Token = char>,
    {
        sep_end_by1(attempt(Rule::parser()), char('\n')).map(Self)
    }
}

parser_from_str!(Rules);

#[derive(Debug)]
struct Notes {
    rules: Rules,
    your_ticket: Ticket,
    nearby_tickets: Vec<Ticket>,
}

impl Notes {
    fn parser<Input>() -> impl Parser<Input, Output = Self>
    where
        Input: Stream<Token = char>,
    {
        let rules = Rules::parser();
        let your_ticket = string("your ticket:\n").with(Ticket::parser().skip(char('\n')));
        let nearby_tickets =
            string("nearby tickets:\n").with(sep_end_by1(Ticket::parser(), char('\n')));
        let all = rules
            .skip(char('\n'))
            .and(your_ticket)
            .skip(char('\n'))
            .and(nearby_tickets);
        all.map(|((rules, your_ticket), nearby_tickets)| Self {
            rules,
            your_ticket,
            nearby_tickets,
        })
    }
}

parser_from_str!(Notes);

fn part_one(notes: &Notes) {
    let mut invalid_values = vec![];
    for ticket in &notes.nearby_tickets {
        invalid_values.extend(ticket.invalid_values(&notes.rules).into_iter());
    }
    let answer: u64 = invalid_values.into_iter().sum();
    println!("{}", answer);
}

#[derive(Debug)]
struct PotentialField<'a> {
    rule: &'a Rule,
    positions: HashSet<usize>,
}

impl<'a> PotentialField<'a> {
    fn new(rule: &'a Rule, size: usize) -> Self {
        Self {
            rule,
            positions: (0..size).collect(),
        }
    }

    fn adjust_for(&mut self, ticket: &Ticket) {
        for (i, v) in ticket.0.iter().enumerate() {
            if !self.rule.validate_ticket_value(*v) {
                self.positions.remove(&i);
            }
        }
    }

    fn done(&self) -> bool {
        self.positions.len() == 1
    }

    fn position(&self) -> usize {
        assert!(self.done());
        *self.positions.iter().next().unwrap()
    }
}

fn gather_potential_fields<'a>(
    rules: &'a Rules,
    tickets: &'a [&'a Ticket],
) -> Vec<PotentialField<'a>> {
    let mut potential_fields = vec![];
    for rule in &rules.0 {
        let mut potential = PotentialField::new(rule, tickets[0].0.len());
        for ticket in tickets {
            potential.adjust_for(ticket);
        }
        potential_fields.push(potential);
    }
    potential_fields
}

fn collapse_fields(potential_fields: &mut [PotentialField<'_>]) {
    for i in 0..potential_fields.len() {
        let other_fields =
            potential_fields
                .iter()
                .enumerate()
                .filter_map(|(j, p)| if j != i { Some(p) } else { None });
        let other_values: HashSet<_> = other_fields
            .map(|p| p.positions.iter())
            .flatten()
            .copied()
            .collect();
        let this_field = &mut potential_fields[i];
        let pos: HashSet<_> = this_field
            .positions
            .iter()
            .filter(|v| !other_values.contains(v))
            .copied()
            .collect();
        if pos.len() == 1 {
            this_field.positions = pos
        }
    }
}

fn find_fields(rules: &Rules, tickets: &[&Ticket]) -> Vec<String> {
    let mut potential_fields = gather_potential_fields(rules, tickets);
    while !potential_fields.iter().all(|p| p.done()) {
        collapse_fields(&mut potential_fields);
    }
    potential_fields.sort_by_key(|p| p.position());
    potential_fields
        .iter()
        .map(|p| p.rule.name.to_owned())
        .collect()
}

fn part_two(notes: &Notes) {
    let valid_tickets: Vec<_> = notes
        .nearby_tickets
        .iter()
        .filter(|t| t.valid(&notes.rules))
        .collect();
    let fields = find_fields(&notes.rules, &valid_tickets);

    let named_fields = fields.iter().zip(notes.your_ticket.0.iter());
    let departure_fields = named_fields.filter(|(f, _)| f.starts_with("departure"));
    let answer: u64 = departure_fields.map(|(_, v)| *v).product();
    println!("{}", answer);
}

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let notes: Notes = input.parse()?;

    println!("Part 1");
    part_one(&notes);

    println!("Part 2");
    part_two(&notes);

    Ok(())
}
