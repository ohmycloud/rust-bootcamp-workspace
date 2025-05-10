use std::net::IpAddr;

use winnow::combinator::separated;
use winnow::{ModalResult, Parser, ascii::digit1};

#[derive(Debug)]
pub struct NginxLog {
    pub addr: IpAddr,
    pub datetime: String,
    pub method: String,
    pub url: String,
    pub protocol: String,
    pub status: u16,
    pub body_bytes: u64,
    pub referer: String,
    pub user_agent: String,
}

pub fn parse_ip(input: &mut &str) -> ModalResult<()> {
    separated(4, digit1, ".").parse_next(input)
}

pub fn take_list<'s>(input: &mut &'s str) -> ModalResult<&'s str> {
    parse_ip.take().parse_next(input)
}
