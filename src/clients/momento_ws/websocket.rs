#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SocketRequest {
    /// Local to the socket. You can start these at 0 and increment one by one.
    #[prost(uint64, tag = "1")]
    pub request_id: u64,
    #[prost(oneof = "socket_request::Kind", tags = "2")]
    pub kind: ::core::option::Option<socket_request::Kind>,
}
/// Nested message and enum types in `SocketRequest`.
pub mod socket_request {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct CacheRequest {
        #[prost(oneof = "cache_request::Kind", tags = "4, 5")]
        pub kind: ::core::option::Option<cache_request::Kind>,
    }
    /// Nested message and enum types in `CacheRequest`.
    pub mod cache_request {
        #[allow(clippy::derive_partial_eq_without_eq)]
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        pub enum Kind {
            #[prost(message, tag = "4")]
            Get(super::super::Get),
            #[prost(message, tag = "5")]
            Set(super::super::Set),
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Kind {
        #[prost(message, tag = "2")]
        Cache(CacheRequest),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SocketResponse {
    /// Local to the socket. You can start these at 0 and increment one by one.
    #[prost(uint64, tag = "1")]
    pub request_id: u64,
    #[prost(oneof = "socket_response::Kind", tags = "2, 3")]
    pub kind: ::core::option::Option<socket_response::Kind>,
}
/// Nested message and enum types in `SocketResponse`.
pub mod socket_response {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct CacheResponse {
        #[prost(oneof = "cache_response::Kind", tags = "1, 2")]
        pub kind: ::core::option::Option<cache_response::Kind>,
    }
    /// Nested message and enum types in `CacheResponse`.
    pub mod cache_response {
        #[allow(clippy::derive_partial_eq_without_eq)]
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        pub enum Kind {
            #[prost(message, tag = "1")]
            Get(super::super::GetResponse),
            #[prost(message, tag = "2")]
            Set(super::super::SetResponse),
        }
    }
    /// The `Status` type defines a logical error model that is suitable for
    /// different programming environments, including REST APIs and RPC APIs. It is
    /// used by [gRPC](<https://github.com/grpc>). Each `Status` message contains
    /// three pieces of data: error code, error message, and error details.
    ///
    /// You can find out more about this error model and how to work with it in the
    /// [API Design Guide](<https://cloud.google.com/apis/design/errors>).
    ///
    /// This is copied from Google's repository, and stripped down a little. We don't
    /// include a details list, so we'll let that be. Codes in Status are int32 in their
    /// source, so I've left it this way. You'll need to convert to what those codes mean
    /// per their documentation: <https://grpc.github.io/grpc/core/md_doc_statuscodes.html>
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Status {
        /// The status code, which should be an enum value of [google.rpc.Code][google.rpc.Code].
        #[prost(int32, tag = "1")]
        pub code: i32,
        /// A developer-facing error message, which should be in English. Any
        /// user-facing error message should be localized and sent in the
        /// [google.rpc.Status.details][google.rpc.Status.details] field, or localized by the client.
        #[prost(string, tag = "2")]
        pub message: ::prost::alloc::string::String,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Kind {
        #[prost(message, tag = "2")]
        Error(Status),
        #[prost(message, tag = "3")]
        Cache(CacheResponse),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Get {
    #[prost(bytes = "vec", tag = "1")]
    pub cache_key: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Set {
    #[prost(bytes = "vec", tag = "1")]
    pub cache_key: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub cache_body: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "3")]
    pub ttl_milliseconds: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetResponse {
    #[prost(oneof = "get_response::Kind", tags = "1, 2")]
    pub kind: ::core::option::Option<get_response::Kind>,
}
/// Nested message and enum types in `GetResponse`.
pub mod get_response {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Kind {
        #[prost(bytes, tag = "1")]
        Hit(::prost::alloc::vec::Vec<u8>),
        #[prost(bool, tag = "2")]
        Miss(bool),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SetResponse {}
