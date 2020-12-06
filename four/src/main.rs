use combine::parser::char::{alpha_num, char, digit, letter, string};
use combine::parser::repeat::count_min_max;
use combine::stream::Stream;
use combine::{many1, sep_by, sep_end_by, EasyParser, Parser};
use std::collections::HashMap;
use std::io::{self, Read};
use std::matches;
use std::num;
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

#[derive(Debug)]
struct Year(usize);

impl FromStr for Year {
    type Err = Error;
    fn from_str(input: &str) -> Result<Self> {
        Ok(Self(input.parse()?))
    }
}

#[derive(Debug)]
enum Length {
    Cm(usize),
    In(usize),
}

impl Length {
    fn parser<Input>() -> impl Parser<Input, Output = Self>
    where
        Input: Stream<Token = char>,
    {
        let number = many1(digit()).map(|v: String| v.parse().unwrap());
        let parser = number.and(string("cm").or(string("in")));
        parser.map(|(value, units)| match units {
            "cm" => Self::Cm(value),
            "in" => Self::In(value),
            _ => unreachable!(),
        })
    }
}

parser_from_str!(Length);

#[derive(Debug)]
struct HexColor(String);

impl HexColor {
    fn parser<Input>() -> impl Parser<Input, Output = Self>
    where
        Input: Stream<Token = char>,
    {
        let hex = digit()
            .or(char('a'))
            .or(char('b'))
            .or(char('c'))
            .or(char('d'))
            .or(char('e'))
            .or(char('f'));
        char('#')
            .and(count_min_max(6, 6, hex))
            .map(|(_, v)| Self(v))
    }
}

parser_from_str!(HexColor);

#[derive(Debug)]
struct SimpleColor(String);

impl FromStr for SimpleColor {
    type Err = Error;
    fn from_str(input: &str) -> Result<Self> {
        match input {
            "amb" => Ok(Self(input.to_owned())),
            "blu" => Ok(Self(input.to_owned())),
            "brn" => Ok(Self(input.to_owned())),
            "gry" => Ok(Self(input.to_owned())),
            "grn" => Ok(Self(input.to_owned())),
            "hzl" => Ok(Self(input.to_owned())),
            "oth" => Ok(Self(input.to_owned())),
            c => Err(Error::ParseError(format!("invalid simple color {}", c))),
        }
    }
}

#[derive(Debug)]
struct PassportId(usize);

impl PassportId {
    fn parser<Input>() -> impl Parser<Input, Output = Self>
    where
        Input: Stream<Token = char>,
    {
        count_min_max(9, 9, digit()).map(|v: String| Self(v.parse().unwrap()))
    }
}

parser_from_str!(PassportId);

#[derive(Debug)]
enum Value {
    Year(Year),
    HexColor(HexColor),
    SimpleColor(SimpleColor),
    Length(Length),
    PassportId(PassportId),
    Other(String),
}

impl Value {
    fn valid_birth_year(&self) -> bool {
        if let Self::Year(Year(y)) = self {
            *y >= 1920 && *y <= 2002
        } else {
            false
        }
    }

    fn valid_issue_year(&self) -> bool {
        if let Self::Year(Year(y)) = self {
            *y >= 2010 && *y <= 2020
        } else {
            false
        }
    }

    fn valid_expiration_year(&self) -> bool {
        if let Self::Year(Year(y)) = self {
            *y >= 2020 && *y <= 2030
        } else {
            false
        }
    }

    fn valid_height(&self) -> bool {
        if let Self::Length(Length::Cm(h)) = self {
            *h >= 150 && *h <= 193
        } else if let Self::Length(Length::In(h)) = self {
            *h >= 59 && *h <= 76
        } else {
            false
        }
    }

    fn valid_hair_color(&self) -> bool {
        matches!(self, Self::HexColor(_))
    }

    fn valid_eye_color(&self) -> bool {
        matches!(self, Self::SimpleColor(_))
    }

    fn valid_passport_id(&self) -> bool {
        matches!(self, Self::PassportId(_))
    }
}

#[derive(Debug)]
struct Passport(HashMap<String, Value>);

impl Passport {
    fn from_strings(map: HashMap<String, String>) -> Self {
        let mut new_map = HashMap::new();
        for (k, v) in map {
            match k.as_str() {
                "byr" | "iyr" | "eyr" => {
                    if let Ok(v) = v.parse() {
                        new_map.insert(k, Value::Year(v));
                        continue;
                    }
                }
                "hgt" => {
                    if let Ok(v) = v.parse() {
                        new_map.insert(k, Value::Length(v));
                        continue;
                    }
                }
                "hcl" => {
                    if let Ok(v) = v.parse() {
                        new_map.insert(k, Value::HexColor(v));
                        continue;
                    }
                }
                "ecl" => {
                    if let Ok(v) = v.parse() {
                        new_map.insert(k, Value::SimpleColor(v));
                        continue;
                    }
                }
                "pid" => {
                    if let Ok(v) = v.parse() {
                        new_map.insert(k, Value::PassportId(v));
                        continue;
                    }
                }
                _ => (),
            }
            new_map.insert(k, Value::Other(v));
        }
        Self(new_map)
    }

    fn parser<Input>() -> impl Parser<Input, Output = Self>
    where
        Input: Stream<Token = char>,
    {
        let value = alpha_num().or(char('#'));
        let key_value = many1(letter()).skip(char(':')).and(many1(value));
        let separator = char(' ').or(char('\n'));
        sep_end_by(key_value, separator).map(|v: Vec<_>| {
            Self::from_strings(v.iter().cloned().collect::<HashMap<String, String>>())
        })
    }

    fn part_one_valid(&self) -> bool {
        self.0.contains_key("byr")
            && self.0.contains_key("iyr")
            && self.0.contains_key("eyr")
            && self.0.contains_key("hgt")
            && self.0.contains_key("hcl")
            && self.0.contains_key("ecl")
            && self.0.contains_key("pid")
    }

    fn part_two_valid(&self) -> bool {
        if !self.part_one_valid() {
            return false;
        }
        for (k, v) in &self.0 {
            let valid = match k.as_str() {
                "byr" => v.valid_birth_year(),
                "iyr" => v.valid_issue_year(),
                "eyr" => v.valid_expiration_year(),
                "hgt" => v.valid_height(),
                "hcl" => v.valid_hair_color(),
                "ecl" => v.valid_eye_color(),
                "pid" => v.valid_passport_id(),
                _ => true,
            };
            if !valid {
                return false;
            }
        }
        true
    }
}

parser_from_str!(Passport);

#[derive(Debug)]
struct PassportCollection(Vec<Passport>);

impl PassportCollection {
    fn parser<Input>() -> impl Parser<Input, Output = Self>
    where
        Input: Stream<Token = char>,
    {
        let passport = Passport::parser();
        sep_by(passport, char('\n')).map(|v| Self(v))
    }
}

impl FromStr for PassportCollection {
    type Err = Error;
    fn from_str(input: &str) -> Result<Self> {
        let (p, remaining): (Self, &str) = Self::parser().easy_parse(input)?;
        require_no_remaining(remaining)?;
        Ok(p)
    }
}

fn part_one(passports: &PassportCollection) {
    println!(
        "{:?}",
        passports.0.iter().filter(|p| p.part_one_valid()).count()
    );
}

fn part_two(passports: &PassportCollection) {
    println!(
        "{:?}",
        passports.0.iter().filter(|p| p.part_two_valid()).count()
    );
}

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let passports: PassportCollection = input.parse()?;

    println!("Part 1");
    part_one(&passports);

    println!("Part 2");
    part_two(&passports);

    Ok(())
}
