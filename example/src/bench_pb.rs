#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Colfer {
    #[prost(int64, tag="1")]
    pub key: i64,
    #[prost(string, tag="2")]
    pub host: ::prost::alloc::string::String,
    #[prost(uint32, tag="4")]
    pub port: u32,
    #[prost(int64, tag="5")]
    pub size: i64,
    #[prost(fixed64, tag="6")]
    pub hash: u64,
    #[prost(double, tag="7")]
    pub ratio: f64,
    #[prost(bool, tag="8")]
    pub route: bool,
}
