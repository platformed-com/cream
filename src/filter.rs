use std::str::FromStr;

use axum::http::StatusCode;
use bumpalo::Bump;
use ijson::INumber;
use nom::Finish;

use crate::{
    error::{Error, ErrorType},
    META_RESOURCE_TYPE,
};

mod parse;

#[derive(Debug, PartialEq)]
pub(crate) enum Filter {
    Present(AttrPath),
    Compare(AttrPath, CompareOp, CompValue),
    Has(AttrPath, Box<Self>),
    And(Vec<Self>),
    Or(Vec<Self>),
    Not(Box<Self>),
}

pub(crate) fn default_visit_filter(visitor: &mut impl Visitor, filter: &mut Filter) {
    match filter {
        Filter::Present(attr_path) => visitor.visit_attr_path(attr_path),
        Filter::Compare(attr_path, _, value) => {
            visitor.visit_attr_path(attr_path);
            visitor.visit_comp_value(value);
        }
        Filter::Has(attr_path, filter) => {
            visitor.visit_attr_path(attr_path);
            visitor.visit_filter(filter);
        }
        Filter::And(filters) | Filter::Or(filters) => {
            for filter in filters {
                visitor.visit_filter(filter);
            }
        }
        Filter::Not(filter) => visitor.visit_filter(filter),
    }
}

pub(crate) fn default_visit_value_path(visitor: &mut impl Visitor, value_path: &mut ValuePath) {
    match value_path {
        ValuePath::Attr(attr_path) => visitor.visit_attr_path(attr_path),
        ValuePath::Filtered(attr_path, filter) => {
            visitor.visit_attr_path(attr_path);
            visitor.visit_filter(filter);
        }
    }
}

pub(crate) fn default_visit_attr_path(_visitor: &mut impl Visitor, _attr_path: &mut AttrPath) {}
pub(crate) fn default_visit_comp_value(_visitor: &mut impl Visitor, _comp_value: &mut CompValue) {}

pub(crate) trait Visitor: Sized {
    fn visit_filter(&mut self, filter: &mut Filter) {
        default_visit_filter(self, filter);
    }
    fn visit_attr_path(&mut self, attr_path: &mut AttrPath) {
        default_visit_attr_path(self, attr_path);
    }
    fn visit_comp_value(&mut self, comp_value: &mut CompValue) {
        default_visit_comp_value(self, comp_value);
    }
    fn visit_value_path(&mut self, value_path: &mut ValuePath) {
        default_visit_value_path(self, value_path);
    }
}

pub mod prelude {
    pub use super::{CompValueRef::*, CompareOp::*, FilterRef::*, ValuePathRef::*};
}

impl Filter {
    pub(crate) fn as_ref<'a>(&'a self, scope: &'a Bump) -> FilterRef<'a> {
        match self {
            Self::Present(attr_path) => FilterRef::Present(attr_path.as_ref()),
            Self::Compare(attr_path, op, value) => {
                FilterRef::Compare(attr_path.as_ref(), *op, value.as_ref())
            }
            Self::Has(attr_path, filter) => {
                FilterRef::ValuePath(attr_path.as_ref(), scope.alloc((**filter).as_ref(scope)))
            }
            Self::And(filters) => FilterRef::And(
                scope.alloc(
                    filters
                        .iter()
                        .map(|filter| filter.as_ref(scope))
                        .collect::<Vec<_>>(),
                ),
            ),
            Self::Or(filters) => FilterRef::Or(
                scope.alloc(
                    filters
                        .iter()
                        .map(|filter| filter.as_ref(scope))
                        .collect::<Vec<_>>(),
                ),
            ),
            Self::Not(filter) => FilterRef::Not(scope.alloc((**filter).as_ref(scope))),
        }
    }
    pub(crate) fn take_resource_type_filter(self) -> Result<(Option<Self>, String), Self> {
        dbg!(&self);
        match self {
            Self::And(filters) => {
                let mut remaining = Vec::new();
                let mut result = None;
                for filter in filters {
                    if result.is_none() {
                        match filter.take_resource_type_filter() {
                            Ok(found) => {
                                result = Some(found);
                            }
                            Err(filter) => {
                                remaining.push(filter);
                            }
                        }
                    } else {
                        remaining.push(filter);
                    }
                }
                if let Some((other, result)) = result {
                    if let Some(other) = other {
                        remaining.push(other);
                    }
                    match remaining.len() {
                        0 => Ok((None, result)),
                        1 => Ok((
                            Some(remaining.pop().expect("Already checked length")),
                            result,
                        )),
                        _ => Ok((Some(Self::And(remaining)), result)),
                    }
                } else {
                    Err(Self::And(remaining))
                }
            }
            Self::Compare(ref attr_path, CompareOp::Equal, CompValue::Str(resource_type))
                if attr_path.urn.is_none()
                    && attr_path.name.eq_ignore_ascii_case(META_RESOURCE_TYPE.name)
                    && attr_path.sub_attr.as_ref().is_some_and(|sub_attr| {
                        sub_attr.eq_ignore_ascii_case(
                            META_RESOURCE_TYPE
                                .sub_attr
                                .expect("Meta resource type has sub attribute"),
                        )
                    }) =>
            {
                Ok((None, resource_type))
            }
            _ => Err(self),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum FilterRef<'a> {
    Present(AttrPathRef<'a>),
    Compare(AttrPathRef<'a>, CompareOp, CompValueRef<'a>),
    ValuePath(AttrPathRef<'a>, &'a Self),
    And(&'a [Self]),
    Or(&'a [Self]),
    Not(&'a Self),
}

impl FilterRef<'_> {
    pub fn iter_cnf(&self) -> impl Iterator<Item = Self> {
        let items: Vec<_> = match self {
            Self::And(filters) => filters.iter().flat_map(Self::iter_cnf).collect(),
            _ => vec![*self],
        };
        items.into_iter()
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum ValuePath {
    Attr(AttrPath),
    Filtered(AttrPath, Filter),
}

impl ValuePath {
    pub(crate) fn as_ref<'a>(&'a self, scope: &'a Bump) -> ValuePathRef<'a> {
        match self {
            Self::Attr(attr_path) => ValuePathRef::Attr(attr_path.as_ref()),
            Self::Filtered(attr_path, filter) => {
                ValuePathRef::Filtered(attr_path.as_ref(), filter.as_ref(scope))
            }
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ValuePathRef<'a> {
    Attr(AttrPathRef<'a>),
    Filtered(AttrPathRef<'a>, FilterRef<'a>),
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct AttrPath {
    pub urn: Option<String>,
    pub name: String,
    pub sub_attr: Option<String>,
}

impl AttrPath {
    pub(crate) fn as_ref(&self) -> AttrPathRef {
        AttrPathRef {
            urn: self.urn.as_deref(),
            name: self.name.as_str(),
            sub_attr: self.sub_attr.as_deref(),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct AttrPathRef<'a> {
    pub urn: Option<&'a str>,
    pub name: &'a str,
    pub sub_attr: Option<&'a str>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CompareOp {
    Equal,
    NotEqual,
    Contains,
    StartsWith,
    EndsWith,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}

impl FromStr for CompareOp {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "eq" => Ok(Self::Equal),
            "ne" => Ok(Self::NotEqual),
            "co" => Ok(Self::Contains),
            "sw" => Ok(Self::StartsWith),
            "ew" => Ok(Self::EndsWith),
            "gt" => Ok(Self::GreaterThan),
            "ge" => Ok(Self::GreaterThanOrEqual),
            "lt" => Ok(Self::LessThan),
            "le" => Ok(Self::LessThanOrEqual),
            _ => Err(format!("{} is not a valid operator", s)),
        }
    }
}

// https://datatracker.ietf.org/doc/html/rfc7159
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum CompValue {
    Null,
    Bool(bool),
    Num(INumber),
    Str(String),
}

impl CompValue {
    pub(crate) fn as_ref(&self) -> CompValueRef {
        match self {
            Self::Null => CompValueRef::Null,
            Self::Bool(b) => CompValueRef::Bool(*b),
            Self::Num(n) => CompValueRef::Num(n),
            Self::Str(s) => CompValueRef::Str(s.as_str()),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CompValueRef<'a> {
    Null,
    Bool(bool),
    Num(&'a INumber),
    Str(&'a str),
}

pub(crate) fn parse_filter(input: &str) -> Result<Filter, Error> {
    let (remain, expression) = parse::filter(input)
        .map_err(|e| e.to_owned())
        .finish()
        .map_err(|e| Error {
            status: StatusCode::BAD_REQUEST,
            schemas: Default::default(),
            scim_type: Some(ErrorType::InvalidFilter),
            detail: format!("Invalid filter: {:?}", e.input),
        })?;
    if !remain.is_empty() {
        return Err(Error {
            status: StatusCode::BAD_REQUEST,
            schemas: Default::default(),
            scim_type: Some(ErrorType::InvalidFilter),
            detail: format!("Invalid filter: unexpected {:?}", remain),
        });
    }
    Ok(expression)
}

pub(crate) fn parse_value_path(input: &str) -> Result<ValuePath, Error> {
    let (remain, expression) = parse::value_path(input)
        .map_err(|e| e.to_owned())
        .finish()
        .map_err(|e| Error {
            status: StatusCode::BAD_REQUEST,
            schemas: Default::default(),
            scim_type: Some(ErrorType::InvalidPath),
            detail: format!("Invalid path: {:?}", e.input),
        })?;
    if !remain.is_empty() {
        return Err(Error {
            status: StatusCode::BAD_REQUEST,
            schemas: Default::default(),
            scim_type: Some(ErrorType::InvalidPath),
            detail: format!("Invalid path: unexpected {:?}", remain),
        });
    }
    Ok(expression)
}
pub(crate) fn parse_attr_path(input: &str) -> Result<AttrPath, Error> {
    let (remain, expression) = parse::attr_path(input)
        .map_err(|e| e.to_owned())
        .finish()
        .map_err(|e| Error {
            status: StatusCode::BAD_REQUEST,
            schemas: Default::default(),
            scim_type: Some(ErrorType::InvalidPath),
            detail: format!("Invalid attribute path: {:?}", e.input),
        })?;
    if !remain.is_empty() {
        return Err(Error {
            status: StatusCode::BAD_REQUEST,
            schemas: Default::default(),
            scim_type: Some(ErrorType::InvalidPath),
            detail: format!("Invalid attribute path: unexpected {:?}", remain),
        });
    }
    Ok(expression)
}
