// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use regex_static::static_regex;
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("{msg}")]
pub struct ParseError {
    msg: String,
}

impl ParseError {
    pub fn from_msg<S: Into<String>>(msg: S) -> Self {
        ParseError { msg: msg.into() }
    }
}

pub fn parse_addr(hs: &str) -> Result<(&str, u16), ParseError> {
    let re = static_regex!(r"^(.+):(\d+)$");

    let (host, port_str) = match re.captures(hs) {
        Some(cap) => (cap.get(1).unwrap().as_str(), cap.get(2).unwrap().as_str()),
        None => {
            return Err(ParseError::from_msg(
                "bad format: expected 'hostname:port' or 'ipaddr:port'",
            ));
        },
    };

    let port = match u16::from_str(port_str) {
        Ok(n) => n,
        Err(err) => return Err(ParseError::from_msg(format!("bad port: {err}"))),
    };

    Ok((host, port))
}
