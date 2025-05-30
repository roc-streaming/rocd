#[allow(unused_imports)]
pub use progenitor_client::{ByteStream, Error, ResponseValue};
#[allow(unused_imports)]
use progenitor_client::{encode_path, RequestBuilderExt};
/// Types used as operation parameters and responses.
#[allow(clippy::all)]
pub mod types {
    /// Error types.
    pub mod error {
        /// Error from a `TryFrom` or `FromStr` implementation.
        pub struct ConversionError(::std::borrow::Cow<'static, str>);
        impl ::std::error::Error for ConversionError {}
        impl ::std::fmt::Display for ConversionError {
            fn fmt(
                &self,
                f: &mut ::std::fmt::Formatter<'_>,
            ) -> Result<(), ::std::fmt::Error> {
                ::std::fmt::Display::fmt(&self.0, f)
            }
        }
        impl ::std::fmt::Debug for ConversionError {
            fn fmt(
                &self,
                f: &mut ::std::fmt::Formatter<'_>,
            ) -> Result<(), ::std::fmt::Error> {
                ::std::fmt::Debug::fmt(&self.0, f)
            }
        }
        impl From<&'static str> for ConversionError {
            fn from(value: &'static str) -> Self {
                Self(value.into())
            }
        }
        impl From<String> for ConversionError {
            fn from(value: String) -> Self {
                Self(value.into())
            }
        }
    }
    ///`ConnectionSpec`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "oneOf": [
    ///    {
    ///      "title": "EndpointConnection",
    ///      "type": "object",
    ///      "required": [
    ///        "connection_type",
    ///        "endpoint_uri"
    ///      ],
    ///      "properties": {
    ///        "connection_type": {
    ///          "$ref": "#/components/schemas/ConnectionType"
    ///        },
    ///        "endpoint_uri": {
    ///          "type": "string"
    ///        }
    ///      }
    ///    },
    ///    {
    ///      "title": "ExternalConnection",
    ///      "type": "object",
    ///      "required": [
    ///        "connection_type",
    ///        "control_uri",
    ///        "media_uri",
    ///        "repair_uri"
    ///      ],
    ///      "properties": {
    ///        "connection_type": {
    ///          "$ref": "#/components/schemas/ConnectionType"
    ///        },
    ///        "control_uri": {
    ///          "type": "string"
    ///        },
    ///        "media_uri": {
    ///          "type": "string"
    ///        },
    ///        "repair_uri": {
    ///          "type": "string"
    ///        }
    ///      }
    ///    }
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug, PartialEq)]
    #[serde(untagged)]
    pub enum ConnectionSpec {
        EndpointConnection {
            connection_type: ConnectionType,
            endpoint_uri: ::std::string::String,
        },
        ExternalConnection {
            connection_type: ConnectionType,
            control_uri: ::std::string::String,
            media_uri: ::std::string::String,
            repair_uri: ::std::string::String,
        },
    }
    impl ::std::convert::From<&Self> for ConnectionSpec {
        fn from(value: &ConnectionSpec) -> Self {
            value.clone()
        }
    }
    ///`ConnectionType`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "endpoint",
    ///    "external"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum ConnectionType {
        #[serde(rename = "endpoint")]
        Endpoint,
        #[serde(rename = "external")]
        External,
    }
    impl ::std::convert::From<&Self> for ConnectionType {
        fn from(value: &ConnectionType) -> Self {
            value.clone()
        }
    }
    impl ::std::fmt::Display for ConnectionType {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Endpoint => write!(f, "endpoint"),
                Self::External => write!(f, "external"),
            }
        }
    }
    impl ::std::str::FromStr for ConnectionType {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "endpoint" => Ok(Self::Endpoint),
                "external" => Ok(Self::External),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for ConnectionType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for ConnectionType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for ConnectionType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///`DriverId`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "unspecified",
    ///    "pipewire"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum DriverId {
        #[serde(rename = "unspecified")]
        Unspecified,
        #[serde(rename = "pipewire")]
        Pipewire,
    }
    impl ::std::convert::From<&Self> for DriverId {
        fn from(value: &DriverId) -> Self {
            value.clone()
        }
    }
    impl ::std::fmt::Display for DriverId {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Unspecified => write!(f, "unspecified"),
                Self::Pipewire => write!(f, "pipewire"),
            }
        }
    }
    impl ::std::str::FromStr for DriverId {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "unspecified" => Ok(Self::Unspecified),
                "pipewire" => Ok(Self::Pipewire),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for DriverId {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for DriverId {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for DriverId {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///`EndpointDir`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "input",
    ///    "output",
    ///    "duplex"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum EndpointDir {
        #[serde(rename = "input")]
        Input,
        #[serde(rename = "output")]
        Output,
        #[serde(rename = "duplex")]
        Duplex,
    }
    impl ::std::convert::From<&Self> for EndpointDir {
        fn from(value: &EndpointDir) -> Self {
            value.clone()
        }
    }
    impl ::std::fmt::Display for EndpointDir {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Input => write!(f, "input"),
                Self::Output => write!(f, "output"),
                Self::Duplex => write!(f, "duplex"),
            }
        }
    }
    impl ::std::str::FromStr for EndpointDir {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "input" => Ok(Self::Input),
                "output" => Ok(Self::Output),
                "duplex" => Ok(Self::Duplex),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for EndpointDir {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for EndpointDir {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for EndpointDir {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///`EndpointSpec`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "display_name",
    ///    "driver",
    ///    "endpoint_type",
    ///    "endpoint_uid",
    ///    "endpoint_uri",
    ///    "stream_direction",
    ///    "system_name"
    ///  ],
    ///  "properties": {
    ///    "display_name": {
    ///      "type": "string"
    ///    },
    ///    "driver": {
    ///      "$ref": "#/components/schemas/DriverId"
    ///    },
    ///    "endpoint_type": {
    ///      "$ref": "#/components/schemas/EndpointType"
    ///    },
    ///    "endpoint_uid": {
    ///      "type": "string"
    ///    },
    ///    "endpoint_uri": {
    ///      "type": "string"
    ///    },
    ///    "stream_direction": {
    ///      "$ref": "#/components/schemas/EndpointDir"
    ///    },
    ///    "system_name": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug, PartialEq)]
    pub struct EndpointSpec {
        pub display_name: ::std::string::String,
        pub driver: DriverId,
        pub endpoint_type: EndpointType,
        pub endpoint_uid: ::std::string::String,
        pub endpoint_uri: ::std::string::String,
        pub stream_direction: EndpointDir,
        pub system_name: ::std::string::String,
    }
    impl ::std::convert::From<&EndpointSpec> for EndpointSpec {
        fn from(value: &EndpointSpec) -> Self {
            value.clone()
        }
    }
    ///`EndpointType`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "system_device",
    ///    "streaming_device"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum EndpointType {
        #[serde(rename = "system_device")]
        SystemDevice,
        #[serde(rename = "streaming_device")]
        StreamingDevice,
    }
    impl ::std::convert::From<&Self> for EndpointType {
        fn from(value: &EndpointType) -> Self {
            value.clone()
        }
    }
    impl ::std::fmt::Display for EndpointType {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::SystemDevice => write!(f, "system_device"),
                Self::StreamingDevice => write!(f, "streaming_device"),
            }
        }
    }
    impl ::std::str::FromStr for EndpointType {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "system_device" => Ok(Self::SystemDevice),
                "streaming_device" => Ok(Self::StreamingDevice),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for EndpointType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for EndpointType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for EndpointType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///`PeerSpec`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "peer_uid",
    ///    "peer_uri"
    ///  ],
    ///  "properties": {
    ///    "peer_uid": {
    ///      "type": "string"
    ///    },
    ///    "peer_uri": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug, PartialEq)]
    pub struct PeerSpec {
        pub peer_uid: ::std::string::String,
        pub peer_uri: ::std::string::String,
    }
    impl ::std::convert::From<&PeerSpec> for PeerSpec {
        fn from(value: &PeerSpec) -> Self {
            value.clone()
        }
    }
    ///`StreamSpec`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "destination",
    ///    "source",
    ///    "stream_uid",
    ///    "stream_uri"
    ///  ],
    ///  "properties": {
    ///    "destination": {
    ///      "$ref": "#/components/schemas/ConnectionSpec"
    ///    },
    ///    "source": {
    ///      "$ref": "#/components/schemas/ConnectionSpec"
    ///    },
    ///    "stream_uid": {
    ///      "type": "string"
    ///    },
    ///    "stream_uri": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug, PartialEq)]
    pub struct StreamSpec {
        pub destination: ConnectionSpec,
        pub source: ConnectionSpec,
        pub stream_uid: ::std::string::String,
        pub stream_uri: ::std::string::String,
    }
    impl ::std::convert::From<&StreamSpec> for StreamSpec {
        fn from(value: &StreamSpec) -> Self {
            value.clone()
        }
    }
}
#[derive(Clone, Debug)]
/**Client for rocd REST API

Real-time audio streaming daemon.

Version: 0.1.0*/
pub struct Client {
    pub(crate) baseurl: String,
    pub(crate) client: reqwest::Client,
}
impl Client {
    /// Create a new client.
    ///
    /// `baseurl` is the base URL provided to the internal
    /// `reqwest::Client`, and should include a scheme and hostname,
    /// as well as port and a path stem if applicable.
    pub fn new(baseurl: &str) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let client = {
            let dur = std::time::Duration::from_secs(15);
            reqwest::ClientBuilder::new().connect_timeout(dur).timeout(dur)
        };
        #[cfg(target_arch = "wasm32")]
        let client = reqwest::ClientBuilder::new();
        Self::new_with_client(baseurl, client.build().unwrap())
    }
    /// Construct a new client with an existing `reqwest::Client`,
    /// allowing more control over its configuration.
    ///
    /// `baseurl` is the base URL provided to the internal
    /// `reqwest::Client`, and should include a scheme and hostname,
    /// as well as port and a path stem if applicable.
    pub fn new_with_client(baseurl: &str, client: reqwest::Client) -> Self {
        Self {
            baseurl: baseurl.to_string(),
            client,
        }
    }
    /// Get the base URL to which requests are made.
    pub fn baseurl(&self) -> &String {
        &self.baseurl
    }
    /// Get the internal `reqwest::Client` used to make requests.
    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }
    /// Get the version of this API.
    ///
    /// This string is pulled directly from the source OpenAPI
    /// document and may be in any format the API selects.
    pub fn api_version(&self) -> &'static str {
        "0.1.0"
    }
}
#[allow(clippy::all)]
#[allow(elided_named_lifetimes)]
impl Client {
    /**Sends a `GET` request to `/peers`

*/
    pub async fn list_peers<'a>(
        &'a self,
    ) -> Result<ResponseValue<::std::vec::Vec<types::PeerSpec>>, Error<()>> {
        let url = format!("{}/peers", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(self.api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .headers(header_map)
            .build()?;
        let result = self.client.execute(request).await;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Sends a `GET` request to `/peers/{peer_uid}`

*/
    pub async fn read_peer<'a>(
        &'a self,
        peer_uid: &'a str,
    ) -> Result<ResponseValue<types::PeerSpec>, Error<()>> {
        let url = format!(
            "{}/peers/{}", self.baseurl, encode_path(& peer_uid.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(self.api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .headers(header_map)
            .build()?;
        let result = self.client.execute(request).await;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Sends a `PUT` request to `/peers/{peer_uid}`

*/
    pub async fn update_peer<'a>(
        &'a self,
        peer_uid: &'a str,
    ) -> Result<ResponseValue<types::PeerSpec>, Error<()>> {
        let url = format!(
            "{}/peers/{}", self.baseurl, encode_path(& peer_uid.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(self.api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .put(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .headers(header_map)
            .build()?;
        let result = self.client.execute(request).await;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Sends a `GET` request to `/peers/{peer_uid}/endpoints`

*/
    pub async fn list_endpoints<'a>(
        &'a self,
        peer_uid: &'a str,
    ) -> Result<ResponseValue<::std::vec::Vec<types::EndpointSpec>>, Error<()>> {
        let url = format!(
            "{}/peers/{}/endpoints", self.baseurl, encode_path(& peer_uid.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(self.api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .headers(header_map)
            .build()?;
        let result = self.client.execute(request).await;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Sends a `GET` request to `/peers/{peer_uid}/endpoints/{endpoint_uid}`

*/
    pub async fn read_endpoint<'a>(
        &'a self,
        peer_uid: &'a str,
        endpoint_uid: &'a str,
    ) -> Result<ResponseValue<types::EndpointSpec>, Error<()>> {
        let url = format!(
            "{}/peers/{}/endpoints/{}", self.baseurl, encode_path(& peer_uid
            .to_string()), encode_path(& endpoint_uid.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(self.api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .headers(header_map)
            .build()?;
        let result = self.client.execute(request).await;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Sends a `PUT` request to `/peers/{peer_uid}/endpoints/{endpoint_uid}`

*/
    pub async fn update_endpoint<'a>(
        &'a self,
        peer_uid: &'a str,
        endpoint_uid: &'a str,
    ) -> Result<ResponseValue<types::EndpointSpec>, Error<()>> {
        let url = format!(
            "{}/peers/{}/endpoints/{}", self.baseurl, encode_path(& peer_uid
            .to_string()), encode_path(& endpoint_uid.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(self.api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .put(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .headers(header_map)
            .build()?;
        let result = self.client.execute(request).await;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Sends a `GET` request to `/streams`

*/
    pub async fn list_streams<'a>(
        &'a self,
    ) -> Result<ResponseValue<::std::vec::Vec<types::StreamSpec>>, Error<()>> {
        let url = format!("{}/streams", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(self.api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .headers(header_map)
            .build()?;
        let result = self.client.execute(request).await;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Sends a `GET` request to `/streams/{stream_uid}`

*/
    pub async fn read_stream<'a>(
        &'a self,
        stream_uid: &'a str,
    ) -> Result<ResponseValue<types::StreamSpec>, Error<()>> {
        let url = format!(
            "{}/streams/{}", self.baseurl, encode_path(& stream_uid.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(self.api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .headers(header_map)
            .build()?;
        let result = self.client.execute(request).await;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Sends a `PUT` request to `/streams/{stream_uid}`

*/
    pub async fn update_stream<'a>(
        &'a self,
        stream_uid: &'a str,
    ) -> Result<ResponseValue<types::StreamSpec>, Error<()>> {
        let url = format!(
            "{}/streams/{}", self.baseurl, encode_path(& stream_uid.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(self.api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .put(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .headers(header_map)
            .build()?;
        let result = self.client.execute(request).await;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
}
/// Items consumers will typically use such as the Client.
pub mod prelude {
    #[allow(unused_imports)]
    pub use super::Client;
}
