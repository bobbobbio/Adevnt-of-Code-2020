use combine::parser::char::{char, digit, string};
use combine::stream::{easy, position, Stream};
use combine::{attempt, between, choice, eof, many, many1, parser, EasyParser, Parser};
use std::convert::Infallible;
use std::io::{self, BufRead};
use std::num;
use std::str::FromStr;

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

impl From<Infallible> for Error {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
enum Expression {
    Number(u64),
    Multiply(Box<Expression>, Box<Expression>),
    Add(Box<Expression>, Box<Expression>),
}

impl Expression {
    fn evaluate(&self) -> u64 {
        match self {
            Self::Number(n) => *n,
            Self::Multiply(a, b) => a.evaluate() * b.evaluate(),
            Self::Add(a, b) => a.evaluate() + b.evaluate(),
        }
    }
}

parser! {
    fn expr_part1_parser_recurse[Input]()(Input) -> Expression
    where [Input: Stream<Token = char>]
    {
        Expression::part1_parser()
    }
}

parser! {
    fn expr_part2_parser_recurse[Input]()(Input) -> Expression
    where [Input: Stream<Token = char>]
    {
        Expression::part2_parser()
    }
}

impl Expression {
    fn part1_parser<Input>() -> impl Parser<Input, Output = Self>
    where
        Input: Stream<Token = char>,
    {
        let number = || many1(digit()).map(|s: String| Self::Number(s.parse::<u64>().unwrap()));
        let subexp = || between(char('('), char(')'), expr_part1_parser_recurse());
        let number_or_subexp = || number().or(subexp());

        let add = number_or_subexp()
            .skip(string(" + "))
            .and(expr_part1_parser_recurse())
            .map(|(a, b)| Self::Add(Box::new(a), Box::new(b)));
        let mult = number_or_subexp()
            .skip(string(" * "))
            .and(expr_part1_parser_recurse())
            .map(|(a, b)| Self::Multiply(Box::new(a), Box::new(b)));
        choice((attempt(add), attempt(mult), number(), subexp()))
    }

    fn part1_parse(input: &str) -> Result<Self> {
        let input: String = input
            .chars()
            .rev()
            .map(|c| match c {
                '(' => ')',
                ')' => '(',
                c => c,
            })
            .collect();
        let (p, _): (Self, _) = Self::part1_parser()
            .skip(eof())
            .easy_parse(position::Stream::new(&input[..]))?;
        Ok(p)
    }

    fn part2_parser<Input>() -> impl Parser<Input, Output = Self>
    where
        Input: Stream<Token = char>,
    {
        let recurse = || expr_part2_parser_recurse();
        let number = || many1(digit()).map(|s: String| Self::Number(s.parse::<u64>().unwrap()));
        let subexp = || between(char('('), char(')'), recurse());
        let number_or_subexp = || number().or(subexp());

        let sep = attempt(string(" + ")).or(string(" * "));
        number_or_subexp()
            .and(many(sep.and(number_or_subexp())))
            .map(|(f, r): (_, Vec<_>)| collapse(f, r))
    }

    fn part2_parse(input: &str) -> Result<Self> {
        let (p, _): (Self, _) = Self::part2_parser().easy_parse(position::Stream::new(input))?;
        Ok(p)
    }
}

fn collapse(f: Expression, mut rest: Vec<(&str, Expression)>) -> Expression {
    if rest.is_empty() {
        return f;
    }

    let (op, next) = rest.remove(0);

    if op == " + " {
        collapse(Expression::Add(Box::new(f), Box::new(next)), rest)
    } else if op == " * " {
        let rest = collapse(next, rest);
        Expression::Multiply(Box::new(f), Box::new(rest))
    } else {
        unreachable!()
    }
}

fn part_one(expressions: &[String]) {
    let expressions = expressions
        .iter()
        .map(|e| Expression::part1_parse(&e[..]).unwrap());
    let answer: u64 = expressions.map(|e| e.evaluate()).sum();
    println!("{}", answer);
}

fn part_two(expressions: &[String]) {
    let expressions = expressions
        .iter()
        .map(|e| Expression::part2_parse(&e[..]).expect(&format!("{}", e)));
    let answer: u64 = expressions.map(|e| e.evaluate()).sum();
    println!("{}", answer);
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
    let expressions: Vec<String> = parse_lines(io::stdin().lock())?;

    println!("Part 1");
    part_one(&expressions);

    println!("Part 2");
    part_two(&expressions);

    Ok(())
}
