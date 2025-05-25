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
    ///`AddressAnchorSpec`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "control_uri",
    ///    "repair_uri",
    ///    "source_uri",
    ///    "type"
    ///  ],
    ///  "properties": {
    ///    "control_uri": {
    ///      "type": "string"
    ///    },
    ///    "repair_uri": {
    ///      "type": "string"
    ///    },
    ///    "source_uri": {
    ///      "type": "string"
    ///    },
    ///    "type": {
    ///      "$ref": "#/components/schemas/AnchorType"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug, PartialEq)]
    pub struct AddressAnchorSpec {
        pub control_uri: ::std::string::String,
        pub repair_uri: ::std::string::String,
        pub source_uri: ::std::string::String,
        #[serde(rename = "type")]
        pub type_: AnchorType,
    }
    impl ::std::convert::From<&AddressAnchorSpec> for AddressAnchorSpec {
        fn from(value: &AddressAnchorSpec) -> Self {
            value.clone()
        }
    }
    ///`AnchorSpec`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "oneOf": [
    ///    {
    ///      "type": "object",
    ///      "required": [
    ///        "endpoint"
    ///      ],
    ///      "properties": {
    ///        "endpoint": {
    ///          "$ref": "#/components/schemas/EndpointAnchorSpec"
    ///        }
    ///      }
    ///    },
    ///    {
    ///      "type": "object",
    ///      "required": [
    ///        "address"
    ///      ],
    ///      "properties": {
    ///        "address": {
    ///          "$ref": "#/components/schemas/AddressAnchorSpec"
    ///        }
    ///      }
    ///    }
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug, PartialEq)]
    pub enum AnchorSpec {
        #[serde(rename = "endpoint")]
        Endpoint(EndpointAnchorSpec),
        #[serde(rename = "address")]
        Address(AddressAnchorSpec),
    }
    impl ::std::convert::From<&Self> for AnchorSpec {
        fn from(value: &AnchorSpec) -> Self {
            value.clone()
        }
    }
    impl ::std::convert::From<EndpointAnchorSpec> for AnchorSpec {
        fn from(value: EndpointAnchorSpec) -> Self {
            Self::Endpoint(value)
        }
    }
    impl ::std::convert::From<AddressAnchorSpec> for AnchorSpec {
        fn from(value: AddressAnchorSpec) -> Self {
            Self::Address(value)
        }
    }
    ///`AnchorType`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "endpoint",
    ///    "address"
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
    pub enum AnchorType {
        #[serde(rename = "endpoint")]
        Endpoint,
        #[serde(rename = "address")]
        Address,
    }
    impl ::std::convert::From<&Self> for AnchorType {
        fn from(value: &AnchorType) -> Self {
            value.clone()
        }
    }
    impl ::std::fmt::Display for AnchorType {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Endpoint => write!(f, "endpoint"),
                Self::Address => write!(f, "address"),
            }
        }
    }
    impl ::std::str::FromStr for AnchorType {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "endpoint" => Ok(Self::Endpoint),
                "address" => Ok(Self::Address),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for AnchorType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for AnchorType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for AnchorType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///`EndpointAnchorSpec`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "endpoint_uuid",
    ///    "peer_uuid",
    ///    "type"
    ///  ],
    ///  "properties": {
    ///    "endpoint_uuid": {
    ///      "type": "string"
    ///    },
    ///    "peer_uuid": {
    ///      "type": "string"
    ///    },
    ///    "type": {
    ///      "$ref": "#/components/schemas/AnchorType"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug, PartialEq)]
    pub struct EndpointAnchorSpec {
        pub endpoint_uuid: ::std::string::String,
        pub peer_uuid: ::std::string::String,
        #[serde(rename = "type")]
        pub type_: AnchorType,
    }
    impl ::std::convert::From<&EndpointAnchorSpec> for EndpointAnchorSpec {
        fn from(value: &EndpointAnchorSpec) -> Self {
            value.clone()
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
    ///`EndpointDriver`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
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
    pub enum EndpointDriver {
        #[serde(rename = "pipewire")]
        Pipewire,
    }
    impl ::std::convert::From<&Self> for EndpointDriver {
        fn from(value: &EndpointDriver) -> Self {
            value.clone()
        }
    }
    impl ::std::fmt::Display for EndpointDriver {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Pipewire => write!(f, "pipewire"),
            }
        }
    }
    impl ::std::str::FromStr for EndpointDriver {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "pipewire" => Ok(Self::Pipewire),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for EndpointDriver {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for EndpointDriver {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for EndpointDriver {
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
    ///    "endpoint_uuid",
    ///    "stream_direction",
    ///    "system_name"
    ///  ],
    ///  "properties": {
    ///    "display_name": {
    ///      "description": "Human-readable name.",
    ///      "type": "string"
    ///    },
    ///    "driver": {
    ///      "$ref": "#/components/schemas/EndpointDriver"
    ///    },
    ///    "endpoint_type": {
    ///      "$ref": "#/components/schemas/EndpointType"
    ///    },
    ///    "endpoint_uuid": {
    ///      "description": "Globally unique endpoint identifier.",
    ///      "type": "string"
    ///    },
    ///    "stream_direction": {
    ///      "$ref": "#/components/schemas/EndpointDir"
    ///    },
    ///    "system_name": {
    ///      "description": "OS name (if any).",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug, PartialEq)]
    pub struct EndpointSpec {
        ///Human-readable name.
        pub display_name: ::std::string::String,
        pub driver: EndpointDriver,
        pub endpoint_type: EndpointType,
        ///Globally unique endpoint identifier.
        pub endpoint_uuid: ::std::string::String,
        pub stream_direction: EndpointDir,
        ///OS name (if any).
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
    ///`StreamSpec`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "destinations",
    ///    "sources",
    ///    "stream_uuid"
    ///  ],
    ///  "properties": {
    ///    "destinations": {
    ///      "description": "To where this stream writes audio.",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/AnchorSpec"
    ///      }
    ///    },
    ///    "sources": {
    ///      "description": "From where this stream reads audio.",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/AnchorSpec"
    ///      }
    ///    },
    ///    "stream_uuid": {
    ///      "description": "Globally unique stream identifier.",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug, PartialEq)]
    pub struct StreamSpec {
        ///To where this stream writes audio.
        pub destinations: ::std::vec::Vec<AnchorSpec>,
        ///From where this stream reads audio.
        pub sources: ::std::vec::Vec<AnchorSpec>,
        ///Globally unique stream identifier.
        pub stream_uuid: ::std::string::String,
    }
    impl ::std::convert::From<&StreamSpec> for StreamSpec {
        fn from(value: &StreamSpec) -> Self {
            value.clone()
        }
    }
}
#[derive(Clone, Debug)]
/**Client for rocd REST API

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
    /**Sends a `GET` request to `/peers/self/endpoints`

*/
    pub async fn list_endpoints<'a>(
        &'a self,
    ) -> Result<ResponseValue<::std::vec::Vec<types::EndpointSpec>>, Error<()>> {
        let url = format!("{}/peers/self/endpoints", self.baseurl,);
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
    /**Sends a `GET` request to `/peers/self/endpoints/{uid}`

Arguments:
- `uid`: Get parameter `uid` from request url path.
*/
    pub async fn read_endpoint<'a>(
        &'a self,
        uid: &'a str,
    ) -> Result<ResponseValue<types::EndpointSpec>, Error<()>> {
        let url = format!(
            "{}/peers/self/endpoints/{}", self.baseurl, encode_path(& uid.to_string()),
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
    /**Sends a `PUT` request to `/peers/self/endpoints/{uid}`

Arguments:
- `uid`: Get parameter `uid` from request url path.
*/
    pub async fn update_endpoint<'a>(
        &'a self,
        uid: &'a str,
    ) -> Result<ResponseValue<types::EndpointSpec>, Error<()>> {
        let url = format!(
            "{}/peers/self/endpoints/{}", self.baseurl, encode_path(& uid.to_string()),
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
    /**Sends a `GET` request to `/streams/{uid}`

Arguments:
- `uid`: Get parameter `uid` from request url path.
*/
    pub async fn read_stream<'a>(
        &'a self,
        uid: &'a str,
    ) -> Result<ResponseValue<types::StreamSpec>, Error<()>> {
        let url = format!(
            "{}/streams/{}", self.baseurl, encode_path(& uid.to_string()),
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
    /**Sends a `PUT` request to `/streams/{uid}`

Arguments:
- `uid`: Get parameter `uid` from request url path.
*/
    pub async fn update_stream<'a>(
        &'a self,
        uid: &'a str,
    ) -> Result<ResponseValue<types::StreamSpec>, Error<()>> {
        let url = format!(
            "{}/streams/{}", self.baseurl, encode_path(& uid.to_string()),
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
