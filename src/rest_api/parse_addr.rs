// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use regex_static::static_regex;
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
#[error("{msg}")]
pub struct ParseError {
    msg: String,
}

impl ParseError {
    pub fn from_msg<S: Into<String>>(msg: S) -> Self {
        ParseError { msg: msg.into() }
    }
}

/// Parse host:port pair.
pub fn parse_addr(hs: &str) -> Result<(&str, u16), ParseError> {
    let re = static_regex!(r"^(.*):(\d+)$");

    let (mut host, port_str) = match re.captures(hs) {
        Some(cap) => (cap.get(1).unwrap().as_str(), cap.get(2).unwrap().as_str()),
        None => {
            return Err(ParseError::from_msg(
                "bad format: expected 'host:port' or 'ipv4:port' or '[ipv6]:port'",
            ));
        },
    };

    if host.is_empty() {
        host = "0.0.0.0";
    }

    let port = match u16::from_str(port_str) {
        Ok(n) => n,
        Err(err) => return Err(ParseError::from_msg(format!("bad port: {err}"))),
    };

    Ok((host, port))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_good() {
        let cases = vec![
            ("example.com:8080", ("example.com", 8080)),
            ("127.0.0.1:80", ("127.0.0.1", 80)),
            ("[::1]:443", ("[::1]", 443)),
            ("localhost:0", ("localhost", 0)),
            ("localhost:65535", ("localhost", 65535)),
            (":8080", ("0.0.0.0", 8080)),
            (":0", ("0.0.0.0", 0)),
        ];

        for (input, expected) in cases {
            let result = parse_addr(input);
            assert_eq!(result, Ok(expected), "Unexpected result for input <{}>", input);
        }
    }

    #[test]
    fn test_bad() {
        let invalid_cases = vec![
            ("localhost", "bad format"),
            ("localhost:-1", "bad format"),
            ("localhost:65536", "bad port"),
            ("localhost:abc", "bad format"),
            (":", "bad format"),
            ("::", "bad format"),
            ("[::]:", "bad format"),
            ("", "bad format"),
        ];

        for (input, expected_err) in invalid_cases {
            let result = parse_addr(input);
            assert!(result.is_err(), "Unexpected result for input: <{}>", input);

            let result_err = result.unwrap_err().to_string();
            assert!(
                result_err.starts_with(expected_err),
                "Unexpected result for input: <{}>, expected <{} ...>, got <{}>",
                input,
                expected_err,
                result_err
            );
        }
    }
}
