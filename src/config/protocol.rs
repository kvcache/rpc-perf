use super::*;

#[derive(Clone, Copy, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    Blabber,
    Http1,
    Http2,
    Memcache,
    Momento,
    MomentoWs,
    Ping,
    Resp,
    Kafka,
}
