// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::dto::error::ValidationError;

use rand::RngCore;
use regex_static::static_regex;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};
use std::borrow::Cow;
use std::fmt;

const UID_LEN: usize = 20;

// 3 segments of 6 chars
const SEG_COUNT: usize = 3;
const SEG_LEN: usize = 6;
const SEG_CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";

/// Globally unique identifier.
///
/// Stored as a fixed-size byte array. Convertible from/into string and
/// (de)serealized to/from json as string.
///
/// Formatted as 20-char ASCII string (18 lowercase letters/digits + 2 separators).
/// Has ~93 bits of entropy.
///
/// Example: rx4sse-w0zas1-gf2s1o
///
/// If a new random UID is generated each minute concurrently on 10'000 hosts over
/// 50 years, the probabilities of a single collision and me getting hit by a
/// meteor are roughly the same.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Uid([u8; UID_LEN]);

impl Uid {
    /// Parse uid from string.
    pub fn parse(text: &str) -> Result<Self, ValidationError> {
        let rx = static_regex!("^([a-z0-9]{6})-([a-z0-9]{6})-([a-z0-9]{6})$");

        let cap = rx.captures(text).ok_or_else(|| ValidationError::UidError(text.into()))?;

        let mut uid = [0u8; UID_LEN];
        let mut pos = 0;

        for n in 0..SEG_COUNT {
            if n != 0 {
                uid[pos] = b'-';
                pos += 1;
            }
            uid[pos..pos + SEG_LEN]
                .copy_from_slice(cap.get(n + 1).unwrap().as_str().as_bytes());
            pos += SEG_LEN;
        }
        debug_assert!(pos == UID_LEN);

        Ok(Uid(uid))
    }

    /// Encode raw bytes into Uid.
    /// Uses only first UID_LEN bytes of the slice.
    fn encode(bytes: &[u8]) -> Uid {
        assert!(bytes.len() >= UID_LEN);

        let mut uid = [0u8; UID_LEN];
        let mut out_pos = 0;
        let mut in_pos = 0;

        for i in 0..SEG_COUNT {
            if i != 0 {
                uid[out_pos] = b'-';
                out_pos += 1;
            }
            for _ in 0..SEG_LEN {
                let in_byte = bytes[in_pos] as usize;
                let out_char = SEG_CHARS[in_byte % SEG_CHARS.len()];

                uid[out_pos] = out_char;
                out_pos += 1;
                in_pos += 1;
            }
        }
        debug_assert!(out_pos == UID_LEN);

        Uid(uid)
    }

    /// Generate UID from CSPRNG.
    pub fn generate_random() -> Uid {
        let mut bytes = [0u8; UID_LEN];
        rand::rng().fill_bytes(&mut bytes);

        Self::encode(&bytes)
    }

    /// Generate UID from SHA-256 hash of scope and name.
    /// Scope argument is for convenience. If a caller uses a unique scope,
    /// it can be sure that a different caller with different scope won't
    /// produce same uid from same name.
    pub fn generate_reproducible(scope: &str, name: &str) -> Uid {
        let mut hasher = Sha256::new();
        hasher.update(scope);
        hasher.update(name);

        let bytes = hasher.finalize();
        Self::encode(&bytes)
    }

    /// Uid to &str
    pub fn as_str(&self) -> &str {
        // SAFETY: constructors (parse, generate_xxx) guarantee that Uid is
        // a fixed-size ASCII string.
        unsafe { str::from_utf8_unchecked(&self.0) }
    }
}

/// Uid from String
impl TryFrom<String> for Uid {
    type Error = ValidationError;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        Uid::parse(&text)
    }
}

/// Uid from &str
impl TryFrom<&str> for Uid {
    type Error = ValidationError;

    fn try_from(text: &str) -> Result<Self, Self::Error> {
        Uid::parse(text)
    }
}

/// Uid to String
impl From<Uid> for String {
    fn from(uid: Uid) -> String {
        uid.to_string()
    }
}

/// Uid to &str
impl<'a> From<&'a Uid> for &'a str {
    fn from(uid: &'a Uid) -> &'a str {
        uid.as_str()
    }
}

impl fmt::Display for Uid {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(self.as_str())
    }
}

impl fmt::Debug for Uid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Uid({})", self.as_str())
    }
}

impl Serialize for Uid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Uid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <Cow<'de, str> as Deserialize<'de>>::deserialize(deserializer).and_then(|text| {
            Uid::parse(&text).map_err(|err| D::Error::custom(err.to_string()))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use assertables::*;
    use std::collections::HashSet;

    #[test]
    fn test_parse() {
        {
            let good =
                vec!["111111-222222-333333", "aaaaaa-bbbbbb-bbbbbb", "rx4sse-w0zas1-gf2s1o"];

            for text in good {
                let uid = Uid::parse(text).expect(text);

                assert_eq!(text, uid.as_str());
                assert_eq!(text, uid.to_string());
            }
        }

        {
            let bad = vec![
                "111111-222222-33333",
                "Aaaaaa-bbbbbb-bbbbbb",
                "111111-222222-33333-",
                "111111-22222233333",
                "--",
                "",
            ];

            for text in &bad {
                assert_matches!(Uid::parse(text), Err(ValidationError::UidError(_)));
            }
        }
    }

    #[test]
    fn test_random() {
        let mut all_uids = HashSet::new();

        for _ in 0..100 {
            let uid = Uid::generate_random();

            assert!(!all_uids.contains(&uid.to_string()));
            all_uids.insert(uid.to_string());

            let parsed_uid = Uid::parse(&uid.to_string()).unwrap();
            assert_eq!(uid, parsed_uid);
            assert_eq!(uid.as_str(), parsed_uid.as_str());
        }
    }

    #[test]
    fn test_reproducible() {
        let mut all_uids = HashSet::new();

        for i in 0..100 {
            let uid = Uid::generate_reproducible("test_scope", &i.to_string());

            assert!(!all_uids.contains(&uid.to_string()));
            all_uids.insert(uid.to_string());

            let parsed_uid = Uid::parse(&uid.to_string()).unwrap();
            assert_eq!(uid, parsed_uid);
            assert_eq!(uid.as_str(), parsed_uid.as_str());
        }
    }

    #[test]
    fn test_scopes() {
        for i in 0..100 {
            for j in 0..100 {
                let a_i = Uid::generate_reproducible("scope_a", &i.to_string());
                let a_j = Uid::generate_reproducible("scope_a", &j.to_string());

                let b_i = Uid::generate_reproducible("scope_b", &i.to_string());
                let b_j = Uid::generate_reproducible("scope_b", &j.to_string());

                assert!(a_i != b_i);
                assert!(a_j != b_j);

                if i == j {
                    assert!(a_i == a_j);
                    assert!(b_i == b_j);
                } else {
                    assert!(a_i != a_j);
                    assert!(b_i != b_j);
                }
            }
        }
    }

    #[test]
    fn test_convert() {
        let cases =
            vec!["111111-222222-333333", "aaaaaa-bbbbbb-bbbbbb", "rx4sse-w0zas1-gf2s1o"];

        for text in cases {
            let uid = Uid::parse(text).expect(text);

            // Uid from String
            {
                let s: String = text.to_string();
                let u = Uid::try_from(s).expect(text);
                assert_eq!(u, uid);
            }

            // Uid from str
            {
                let s: &str = text;
                let u = Uid::try_from(s).expect(text);
                assert_eq!(u, uid);
            }

            // String from Uid
            {
                let u = uid;
                let s = String::from(u);
                assert_eq!(text, s);
            }

            // str from Uid
            {
                type StrRef<'a> = &'a str;
                let u = uid;
                let s = StrRef::from(&u);
                assert_eq!(text, s);
            }

            // Uid into String
            {
                let u = uid;
                let s: String = u.into();
                assert_eq!(text, s);
            }

            // Uid into str
            {
                let u = uid;
                let s: &str = (&u).into();
                assert_eq!(text, s);
            }
        }
    }

    #[test]
    fn test_fmt() {
        let cases =
            vec!["111111-222222-333333", "aaaaaa-bbbbbb-bbbbbb", "rx4sse-w0zas1-gf2s1o"];

        for text in cases {
            let uid = Uid::parse(text).expect(text);

            // fmt::Display
            assert_eq!(text, format!("{}", uid));
            // fmt::Debug
            assert_eq!(format!("Uid({})", text), format!("{:?}", uid));
        }
    }

    #[test]
    fn test_cmp() {
        let cases =
            vec!["111111-222222-333333", "aaaaaa-bbbbbb-bbbbbb", "rx4sse-w0zas1-gf2s1o"];

        for text1 in &cases {
            for text2 in &cases {
                let uid1 = Uid::parse(text1).unwrap();
                let uid2 = Uid::parse(text2).unwrap();

                assert!(uid1 == uid1);
                assert!(uid1 < uid2 || uid2 < uid1 || uid1 == uid2);

                if text1 == text2 {
                    assert!(uid1 == uid2);
                } else {
                    assert!(uid1 != uid2);
                }
            }
        }
    }

    #[test]
    fn test_serde() {
        #[derive(PartialEq, Debug, Serialize, Deserialize)]
        struct TestDto {
            pub uid: Uid,
        }

        let in_json = r#"{"uid":"rx4sse-w0zas1-gf2s1o"}"#;

        let uid = Uid::parse("rx4sse-w0zas1-gf2s1o").unwrap();
        let dto: TestDto = serde_json::from_str(in_json).unwrap();

        assert_eq!(dto.uid, uid);
        assert_eq!(dto, TestDto { uid: uid });

        let out_json = serde_json::to_string(&dto).unwrap();

        assert_eq!(in_json, out_json);
    }
}
