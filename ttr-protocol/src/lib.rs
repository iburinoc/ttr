use derive_more::From;
use protobuf::error::ProtobufError;
use thiserror::Error;

mod protos;

pub use protos::*;

macro_rules! define_msg {
    ($($t:ident => $k:expr ,)*) => {
        #[derive(Debug, Clone, From)]
        pub enum Message {
            $( $t($t), )*
            Unrecognized { kind: u32, data: Vec<u8> },
        }

        impl Message {
            pub fn kind(&self) -> u32 {
                match self {
                    $(
                        Message::$t(_) => $k,
                    )*
                    Message::Unrecognized { kind, .. } => *kind,
                }
            }

            pub fn parse(kind: u32, data: &[u8]) -> Result<Message, ProtobufError> {
                match kind {
                    $(
                        $k => Ok(Message::$t(protobuf::parse_from_bytes::<$t>(data)?)),
                    )*
                    _ => Ok(Message::Unrecognized { kind, data: data.into() }),
                }
            }

            pub fn serialize(&self) -> Vec<u8> {
                let mut msg = Vec::new();
                msg.extend_from_slice(&self.kind().to_be_bytes());
                msg.extend_from_slice(&[0u8; 4]);
                let len = match self {
                    $(
                        Message::$t(m) => {
                            use protobuf::Message;

                            m.write_to_vec(&mut msg).unwrap();
                            msg.len() - 8
                        }
                    )*
                    Message::Unrecognized { data, .. } => {msg.extend_from_slice(data); data.len() }
                };
                msg[4..8].copy_from_slice(&(len as u32).to_be_bytes());
                msg
            }
        }
    };
}

define_msg! {
    Heartbeat => 2,
    Connect => 3,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct Header {
    kind: u32,
    mlen: u32,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Wrong number of bytes provided: {0} (expected {1})")]
    WrongByteCount(usize, usize),
    #[error("Error processing protobuf: {0:?}")]
    ProtobufError(#[from] ProtobufError),
}

impl Message {
    pub fn parse_header(data: &[u8]) -> Result<Header, ParseError> {
        Header::parse(data)
    }
}

impl Header {
    pub fn bytes_required(&self) -> usize {
        self.mlen as usize
    }

    pub fn parse(data: &[u8]) -> Result<Header, ParseError> {
        use std::convert::TryInto;

        let len = data.len();
        if len == 8 {
            let kind = u32::from_be_bytes(data[0..4].try_into().unwrap());
            let mlen = u32::from_be_bytes(data[4..8].try_into().unwrap());

            Ok(Header { kind, mlen })
        } else {
            Err(ParseError::WrongByteCount(len, 8))
        }
    }

    pub fn parse_message(&self, data: &[u8]) -> Result<Message, ParseError> {
        let len = data.len();
        if len == self.mlen as _ {
            Ok(Message::parse(self.kind, data)?)
        } else {
            Err(ParseError::WrongByteCount(len, self.mlen as _))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_hex(s: &str) -> Result<Message, ParseError> {
        let data = hex::decode(s).unwrap();
        let header = Header::parse(&data[0..8])?;
        header.parse_message(&data[8..])
    }

    macro_rules! parse_test {
        ($hex:expr, $msg:expr) => {
            let result = parse_hex($hex);
            let s = format!("{:?}", result);
            assert_eq!(s, $msg);
        };
    }

    #[test]
    fn test_heartbeat() {
        parse_test!("0000000200000000", "Ok(Heartbeat())");
    }

    #[test]
    fn test_connect() {
        parse_test!(
            "000000030000001c12047365616e18e1f49eb79a85efadfd0122096d79436f6e74657874",
            "Ok(Connect(name: \"sean\" peerId: 18256392401556322913 ctx: \"myContext\"))"
        );
    }
}
