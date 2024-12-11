use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, tag_no_case, take, take_while_m_n};
use nom::character::complete::{alpha1, alphanumeric1, char, digit1, space0, space1};
use nom::combinator::{map, map_opt, map_res, opt, recognize, value, verify};
use nom::multi::{fold_many0, many0_count, many1, separated_list1};
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};
use nom::{IResult, Parser};

use super::*;

pub fn filter(i: &str) -> IResult<&str, Filter> {
    or_filter(i)
}

pub fn attr_filter(i: &str) -> IResult<&str, Filter> {
    alt((
        map(
            terminated(attr_path, pair(space1, tag_no_case("pr"))),
            Filter::Present,
        ),
        map(
            tuple((
                terminated(attr_path, space1),
                terminated(compare_op, space1),
                comp_value,
            )),
            |(attr_path, compare_op, comp_value)| {
                Filter::Compare(attr_path, compare_op, comp_value)
            },
        ),
    ))(i)
}

pub fn or_filter(i: &str) -> IResult<&str, Filter> {
    map(
        separated_list1(tuple((space1, tag_no_case("or"), space1)), and_filter),
        |clauses| {
            if clauses.len() == 1 {
                clauses.into_iter().next().expect("Already checked length")
            } else {
                Filter::Or(clauses)
            }
        },
    )(i)
}

pub fn and_filter(i: &str) -> IResult<&str, Filter> {
    map(
        separated_list1(tuple((space1, tag_no_case("and"), space1)), group_filter),
        |clauses| {
            if clauses.len() == 1 {
                clauses.into_iter().next().expect("Already checked length")
            } else {
                Filter::And(clauses)
            }
        },
    )(i)
}

pub fn group_filter(i: &str) -> IResult<&str, Filter> {
    alt((
        map(
            separated_pair(
                map(opt(tag_no_case("not")), |not| not.is_some()),
                space0,
                delimited(char('('), filter, char(')')),
            ),
            |(not, filter)| {
                if not {
                    Filter::Not(Box::new(filter))
                } else {
                    filter
                }
            },
        ),
        attr_filter,
        has_filter,
    ))(i)
}

pub fn has_filter(i: &str) -> IResult<&str, Filter> {
    map(
        pair(attr_path, delimited(char('['), filter, char(']'))),
        |(attr_path, mut filter)| {
            AttrPathPrefixer { parent: &attr_path }.visit_filter(&mut filter);
            Filter::Has(attr_path, Box::new(filter))
        },
    )(i)
}

struct AttrPathPrefixer<'a> {
    parent: &'a AttrPath,
}

impl Visitor for AttrPathPrefixer<'_> {
    fn visit_attr_path(&mut self, attr_path: &mut AttrPath) {
        attr_path.sub_attr = Some(attr_path.name.clone());
        attr_path.name = self.parent.name.clone();
        attr_path.urn = self.parent.urn.clone();
    }
}

pub fn value_path(i: &str) -> IResult<&str, ValuePath> {
    map(
        pair(
            attr_path,
            opt(pair(delimited(char('['), filter, char(']')), opt(sub_attr))),
        ),
        |(mut attr_path, filter)| {
            if let Some((mut filter, sub_attr)) = filter {
                // Prefix all attributes inside the filter with the parent attribute
                AttrPathPrefixer { parent: &attr_path }.visit_filter(&mut filter);
                attr_path.sub_attr = sub_attr;
                ValuePath::Filtered(attr_path, filter)
            } else {
                ValuePath::Attr(attr_path)
            }
        },
    )(i)
}

pub fn attr_path(i: &str) -> IResult<&str, AttrPath> {
    map(
        tuple((opt(urn), attr_name, opt(sub_attr))),
        |(urn, name, sub_attr)| AttrPath {
            urn,
            name,
            sub_attr,
        },
    )(i)
}

pub fn urn(i: &str) -> IResult<&str, String> {
    map(
        many1(terminated(many1(alt((alphanumeric1, tag(".")))), tag(":"))),
        |namespaces| {
            let uri: String = namespaces
                .into_iter()
                .fold("".to_string(), |acc, namespace| {
                    acc + &namespace.join("") + ":"
                });

            uri[0..uri.len() - 1].to_string()
        },
    )(i)
}

pub fn compare_op(i: &str) -> IResult<&str, CompareOp> {
    map_res(take(2usize), CompareOp::from_str)(i)
}

pub fn comp_value(i: &str) -> IResult<&str, CompValue> {
    alt((
        value(CompValue::Null, tag("null")),
        value(CompValue::Bool(false), tag("false")),
        value(CompValue::Bool(true), tag("true")),
        map(
            map_res(many1(alt((digit1, tag(".")))), |digit| {
                serde_json::from_str(&digit.join(""))
            }),
            CompValue::Num,
        ),
        map(parse_string, CompValue::Str),
    ))(i)
}

pub fn attr_name(i: &str) -> IResult<&str, String> {
    map(
        recognize(pair(
            alpha1,
            many0_count(alt((alphanumeric1, tag("_"), tag("-")))),
        )),
        Into::into,
    )(i)
}

pub fn sub_attr(i: &str) -> IResult<&str, String> {
    preceded(char('.'), attr_name)(i)
}

fn parse_unicode(input: &str) -> IResult<&str, char>
where
{
    let parse_hex = preceded(
        char('u'),
        take_while_m_n(4, 4, |c: char| c.is_ascii_hexdigit()),
    );
    let parse_u32 = map_res(parse_hex, move |hex| u32::from_str_radix(hex, 16));
    map_opt(parse_u32, std::char::from_u32).parse(input)
}

/// Parse an escaped character: \n, \t, \r, \u00AC, etc.
fn parse_escaped_char(input: &str) -> IResult<&str, char> {
    preceded(
        char('\\'),
        alt((
            value('"', char('"')),
            value('\\', char('\\')),
            value('/', char('/')),
            value('\u{08}', char('b')),
            value('\u{0C}', char('f')),
            value('\n', char('n')),
            value('\r', char('r')),
            value('\t', char('t')),
            parse_unicode,
        )),
    )
    .parse(input)
}

/// Parse a non-empty block of text that doesn't include \ or "
fn parse_literal(input: &str) -> IResult<&str, &str> {
    let not_quote_slash = is_not("\"\\");
    verify(not_quote_slash, |s: &str| !s.is_empty()).parse(input)
}

/// A string fragment contains a fragment of a string being parsed: either
/// a non-empty Literal (a series of non-escaped characters), a single
/// parsed escaped character, or a block of escaped whitespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StringFragment<'a> {
    Literal(&'a str),
    EscapedChar(char),
}

/// Combine parse_literal, parse_escaped_whitespace, and parse_escaped_char
/// into a StringFragment.
fn parse_fragment(input: &str) -> IResult<&str, StringFragment> {
    alt((
        map(parse_literal, StringFragment::Literal),
        map(parse_escaped_char, StringFragment::EscapedChar),
    ))
    .parse(input)
}

/// Parse a string. Use a loop of parse_fragment and push all of the fragments
/// into an output string.
fn parse_string(input: &str) -> IResult<&str, String> {
    let build_string = fold_many0(parse_fragment, String::new, |mut string, fragment| {
        match fragment {
            StringFragment::Literal(s) => string.push_str(s),
            StringFragment::EscapedChar(c) => string.push(c),
        }
        string
    });
    delimited(char('"'), build_string, char('"')).parse(input)
}
