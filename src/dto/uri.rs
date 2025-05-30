// Copyright (c) Roc Streaming authors
// Licensed under MPL-2.0
use crate::dto::uid::*;
use crate::dto::validate::*;

use regex_static::static_regex;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Cow;
use std::fmt;
use url::Url;

/// Resource identifier.
///
/// Immutable.
///
/// Locates some resource, either internal or external to the peer's network.
///
/// For internal resources (peers, endpoints, streams, etc), URI consists
/// of the resource type and one or a few UIDs. Such URI is formatted
/// as a relative URI that matches rocd REST API, for example:
///
///   /peers/rx4sse-w0zas1-gf2s1o/endpoints/6azqwm-ihg3c9-p9b2aq
///
/// For external resources (e.g. static address used as stream destination),
/// URI stores a string with the absolute URL, like:
///
///   rtp+rs8m://192.168.0.101:30000
///
/// NOTE: Uri is a newtype (struct) instead of enum to prevent callers
/// creating Uri from parts or modifying parts bypassing constructor.
/// We want to guarantee that only valid URI can be constructed.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Uri(UriParts);

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UriParts {
    Peer { peer_uid: Uid },
    Endpoint { peer_uid: Uid, endpoint_uid: Uid },
    Stream { stream_uid: Uid },
    External { url: String },
}

#[derive(Copy, Clone, PartialEq, Debug, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum UriKind {
    Peer,
    Endpoint,
    Stream,
    External,
}

impl Uri {
    /// Parse URI from string.
    pub fn parse(text: &str) -> Result<Self, ValidationError> {
        let rx = static_regex!(
            r"(?x)
            ^(?:
              | (?: /peers/(?P<peer_uid>[^/]+)
                      (?:
                         /endpoints/(?P<endpoint_uid>[^/]+)
                      )?
                 )
              | (?: /streams/(?P<stream_uid>[^/]+) )
              | (?P<external> [a-z0-9+]+ :// .+ )
            )$"
        );

        let cap =
            rx.captures(text).ok_or_else(|| ValidationError::UriFormatError(text.into()))?;

        if let Some(peer_uid) = cap.name("peer_uid") {
            let peer_uid = Uid::parse(peer_uid.as_str())?;

            if let Some(endpoint_uid) = cap.name("endpoint_uid") {
                let endpoint_uid = Uid::parse(endpoint_uid.as_str())?;
                return Ok(Uri(UriParts::Endpoint { peer_uid, endpoint_uid }));
            }

            return Ok(Uri(UriParts::Peer { peer_uid }));
        }

        if let Some(stream_uid) = cap.name("stream_uid") {
            let stream_uid = Uid::parse(stream_uid.as_str())?;
            return Ok(Uri(UriParts::Stream { stream_uid }));
        }

        if let Some(external) = cap.name("external") {
            // ensure this is a valid url
            _ = Url::parse(external.as_str())
                .map_err(|err| ValidationError::UrlFormatError(text.into(), err))?;
            return Ok(Uri(UriParts::External { url: external.as_str().into() }));
        }

        Err(ValidationError::UriFormatError(text.into()))
    }

    /// Create peer URI.
    pub fn from_peer(peer_uid: &Uid) -> Uri {
        Uri(UriParts::Peer { peer_uid: *peer_uid })
    }

    /// Create endpoint URI.
    pub fn from_endpoint(peer_uid: &Uid, endpoint_uid: &Uid) -> Uri {
        Uri(UriParts::Endpoint { peer_uid: *peer_uid, endpoint_uid: *endpoint_uid })
    }

    /// Create stream URI.
    pub fn from_stream(stream_uid: &Uid) -> Uri {
        Uri(UriParts::Stream { stream_uid: *stream_uid })
    }

    /// Determine URI type.
    pub fn kind(&self) -> UriKind {
        match self.0 {
            UriParts::Peer { .. } => UriKind::Peer,
            UriParts::Endpoint { .. } => UriKind::Endpoint,
            UriParts::Stream { .. } => UriKind::Stream,
            UriParts::External { .. } => UriKind::External,
        }
    }

    /// Get peer UID, if URI has it.
    pub fn peer_uid(&self) -> Option<Uid> {
        match self.0 {
            UriParts::Peer { peer_uid, .. } => Some(peer_uid),
            UriParts::Endpoint { peer_uid, .. } => Some(peer_uid),
            _ => None,
        }
    }

    /// Get endpoint UID, if URI has it.
    pub fn endpoint_uid(&self) -> Option<Uid> {
        match self.0 {
            UriParts::Endpoint { endpoint_uid, .. } => Some(endpoint_uid),
            _ => None,
        }
    }

    /// Get stream UID, if URI has it.
    pub fn stream_uid(&self) -> Option<Uid> {
        match self.0 {
            UriParts::Stream { stream_uid, .. } => Some(stream_uid),
            _ => None,
        }
    }
}

/// Uri from String
impl TryFrom<String> for Uri {
    type Error = ValidationError;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        Uri::parse(&text)
    }
}

/// Uri from &str
impl TryFrom<&str> for Uri {
    type Error = ValidationError;

    fn try_from(text: &str) -> Result<Self, Self::Error> {
        Uri::parse(text)
    }
}

/// Uri to String
impl From<Uri> for String {
    fn from(uri: Uri) -> String {
        uri.to_string()
    }
}

impl fmt::Display for Uri {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            UriParts::Peer { peer_uid } => {
                write!(f, "/peers/{peer_uid}")?;
            },
            UriParts::Endpoint { peer_uid, endpoint_uid } => {
                write!(f, "/peers/{peer_uid}/endpoints/{endpoint_uid}")?;
            },
            UriParts::Stream { stream_uid } => {
                write!(f, "/streams/{stream_uid}")?;
            },
            UriParts::External { url } => {
                write!(f, "{url}")?;
            },
        }
        Ok(())
    }
}

impl fmt::Debug for Uri {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Uri({})", self)
    }
}

impl Serialize for Uri {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Uri {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <Cow<'de, str> as Deserialize<'de>>::deserialize(deserializer).and_then(|text| {
            Uri::parse(&text).map_err(|err| D::Error::custom(err.to_string()))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertables::*;

    #[test]
    fn test_parse_ok() {
        let cases = vec![
            (UriKind::Peer, "/peers/111111-222222-333333"),
            (UriKind::Endpoint, "/peers/111111-222222-333333/endpoints/aaaaaa-bbbbbb-bbbbbb"),
            (UriKind::Stream, "/streams/rx4sse-w0zas1-gf2s1o"),
            (UriKind::External, "rtp+rs8m://192.168.0.101:30000"),
            (UriKind::External, "rtsp://user:password@example.com:554/music.ogg?ts=1000"),
        ];

        for (kind, text) in &cases {
            let uri = Uri::parse(text).expect(text);

            assert_eq!(*text, uri.to_string());
            assert_eq!(*kind, uri.kind());
        }
    }

    #[test]
    fn test_parse_err() {
        let cases = vec![
            "/peers/111111-222222-33333",
            "peers/111111-222222-333333",
            "/peers//111111-222222-333333",
            "/peers/111111-222222-333333/",
            "/peers/111111-222222-333333/xxx/aaaaaa-bbbbbb-bbbbbb",
            "/peers/111111-222222-333333/endpoints/aaaaaa-bbbbbb-bbbbb",
            "/peers/111111-222222-333333/endpoints/aaaaaa-bbbbbb-bbbbbb/",
            "/peers//endpoints/",
            "rtp+rs8m:/192.168.0.101:30000",
            "rtp+rs8m://192.168.0.101::",
            "//",
            "/",
            "",
        ];

        for text in &cases {
            assert_matches!(
                Uri::parse(text),
                Err(ValidationError::UriFormatError(_)
                    | ValidationError::UidFormatError(_)
                    | ValidationError::UrlFormatError(_, _))
            );
        }
    }

    #[test]
    fn test_convert() {
        let cases = vec![
            "/peers/111111-222222-333333",
            "/peers/111111-222222-333333/endpoints/aaaaaa-bbbbbb-bbbbbb",
            "/streams/rx4sse-w0zas1-gf2s1o",
        ];

        for text in cases {
            let uri = Uri::parse(text).expect(text);

            // Uri from String
            {
                let s: String = text.to_string();
                let u = Uri::try_from(s).expect(text);
                assert_eq!(u, uri);
            }

            // Uri from str
            {
                let s: &str = text;
                let u = Uri::try_from(s).expect(text);
                assert_eq!(u, uri);
            }

            // String from Uri
            {
                let u = uri.clone();
                let s = String::from(u);
                assert_eq!(text, s);
            }

            // Uri into String
            {
                let u = uri.clone();
                let s: String = u.into();
                assert_eq!(text, s);
            }
        }
    }

    #[test]
    fn test_fmt() {
        let cases = vec![
            "/peers/111111-222222-333333",
            "/peers/111111-222222-333333/endpoints/aaaaaa-bbbbbb-bbbbbb",
            "/streams/rx4sse-w0zas1-gf2s1o",
        ];

        for text in &cases {
            let uri = Uri::parse(text).expect(text);

            // fmt::Display
            assert_eq!(*text, format!("{}", uri));
            // fmt::Debug
            assert_eq!(format!("Uri({})", text), format!("{:?}", uri));
        }
    }

    #[test]
    fn test_cmp() {
        let cases = vec![
            "/peers/111111-222222-333333",
            "/peers/111111-222222-333333/endpoints/aaaaaa-bbbbbb-bbbbbb",
            "/streams/rx4sse-w0zas1-gf2s1o",
        ];

        for text1 in &cases {
            for text2 in &cases {
                let uri1 = Uri::parse(text1).unwrap();
                let uri2 = Uri::parse(text2).unwrap();

                assert!(uri1 == uri1);
                assert!(uri1 < uri2 || uri2 < uri1 || uri1 == uri2);

                if text1 == text2 {
                    assert!(uri1 == uri2);
                } else {
                    assert!(uri1 != uri2);
                }
            }
        }
    }

    #[test]
    fn test_serde() {
        #[derive(PartialEq, Debug, Serialize, Deserialize)]
        struct TestDto {
            pub uri: Uri,
        }

        let in_json = r#"{"uri":"/streams/rx4sse-w0zas1-gf2s1o"}"#;

        let uri = Uri::parse("/streams/rx4sse-w0zas1-gf2s1o").unwrap();
        let dto: TestDto = serde_json::from_str(in_json).unwrap();

        assert_eq!(dto.uri, uri);
        assert_eq!(dto, TestDto { uri: uri });

        let out_json = serde_json::to_string(&dto).unwrap();

        assert_eq!(in_json, out_json);
    }
}
