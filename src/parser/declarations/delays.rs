use crate::parser::*;
//use nom::branch::*;
//use nom::combinator::*;
use nom::error::*;
use nom::{Err, IResult};

// -----------------------------------------------------------------------------

#[derive(Debug)]
pub enum Delay3<'a> {
    DelayValue(DelayValue<'a>),
    Mintypmax(Delay3Mintypmax<'a>),
}

#[derive(Debug)]
pub struct Delay3Mintypmax<'a> {
    pub nodes: (
        MintypmaxExpression<'a>,
        Option<(MintypmaxExpression<'a>, Option<MintypmaxExpression<'a>>)>,
    ),
}

#[derive(Debug)]
pub enum Delay2<'a> {
    DelayValue(DelayValue<'a>),
    Mintypmax(Delay2Mintypmax<'a>),
}

#[derive(Debug)]
pub struct Delay2Mintypmax<'a> {
    pub nodes: (MintypmaxExpression<'a>, Option<MintypmaxExpression<'a>>),
}

#[derive(Debug)]
pub enum DelayValue<'a> {
    UnsignedNumber(&'a str),
    RealNumber(RealNumber<'a>),
    Identifier(Identifier<'a>),
    TimeLiteral(TimeLiteral<'a>),
    Step1,
}

// -----------------------------------------------------------------------------

pub fn delay3(s: &str) -> IResult<&str, Delay3> {
    Err(Err::Error(make_error(s, ErrorKind::Fix)))
}

pub fn delay2(s: &str) -> IResult<&str, Delay2> {
    Err(Err::Error(make_error(s, ErrorKind::Fix)))
}

pub fn delay_value(s: &str) -> IResult<&str, DelayValue> {
    Err(Err::Error(make_error(s, ErrorKind::Fix)))
}