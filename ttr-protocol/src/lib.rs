use derive_more::From;
use protobuf::{error::ProtobufError, Message as ProtoMessage};
use thiserror::Error;

pub mod protos;

macro_rules! define_proto_variant {
    ($ty:ident, $($ctor:ident : $field:ident,)*) => {
        #[derive(Debug, Clone)]
        pub enum $ty {
            $( $ctor(protos::$ctor), )*
            Unrecognized(protos::$ty),
        }

        impl Action for $ty {
            type Proto = protos::$ty;

            fn to_proto(s: Self) -> Self::Proto {
                s.into()
            }

            fn from_proto(p: Self::Proto) -> Self {
                p.into()
            }

            fn get_unrecognized(&self) -> Option<&Self::Proto> {
                match self {
                    $ty::Unrecognized(m) => Some(m),
                    _ => None,
                }
            }
        }

        impl From<protos::$ty> for $ty {
            fn from(mut m: protos::$ty) -> Self {
                $(
                    if let Some(f) = m.$field.take() {
                        $ty::$ctor(f)
                    } else
                )*
                    {
                        $ty::Unrecognized(m)
                    }
            }
        }

        impl From<$ty> for protos::$ty {
            fn from(m: $ty) -> Self {
                match m {
                    $(
                        $ty::$ctor(m) => {
                            let mut msg = protos::$ty::new();
                            msg.$field = protobuf::SingularPtrField::some(m);
                            msg
                        }
                    )*
                        $ty::Unrecognized(msg) => msg,
                }
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Wrong number of bytes provided: {0} (expected {1})")]
    WrongByteCount(usize, usize),
    #[error("Error processing protobuf: {0:?}")]
    ProtobufError(#[from] ProtobufError),
    #[error("Unrecognized message kind: {0}")]
    UnrecognizedKind(u32),
}

#[derive(Debug, Clone, From)]
pub enum Message<A: Action> {
    Action(A),
    Heartbeat(protos::Heartbeat),
    Connect(protos::Connect),
}

pub type ClientMessage = Message<Query>;
pub type ServerMessage = Message<Response>;

impl<A: Action> Message<A> {
    pub fn kind(&self) -> u32 {
        use Message::*;
        match self {
            Action(_) => 1,
            Heartbeat(_) => 2,
            Connect(_) => 3,
        }
    }

    fn from_data(kind: u32, data: &[u8]) -> Result<Message<A>, ParseError> {
        use Message::*;
        match kind {
            1 => {
                let m = protobuf::parse_from_bytes::<A::Proto>(data)?;
                Ok(Action(A::from_proto(m)))
            }
            2 => Ok(Heartbeat(protobuf::parse_from_bytes::<protos::Heartbeat>(
                data,
            )?)),
            3 => Ok(Connect(protobuf::parse_from_bytes::<protos::Connect>(
                data,
            )?)),
            _ => Err(ParseError::UnrecognizedKind(kind)),
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        use Message::*;
        let mut msg = Vec::new();
        msg.extend_from_slice(&self.kind().to_be_bytes());
        msg.extend_from_slice(&[0u8; 4]);
        let len = match self {
            Action(a) => write_and_get_len(&mut msg, &A::to_proto(a.clone())),
            Heartbeat(m) => write_and_get_len(&mut msg, m),
            Connect(m) => write_and_get_len(&mut msg, m),
        };
        msg[4..8].copy_from_slice(&(len as u32).to_be_bytes());
        msg
    }

    pub fn parse_header(data: &[u8]) -> Result<Header, ParseError> {
        Header::parse(data)
    }

    pub fn parse(header: &Header, data: &[u8]) -> Result<Message<A>, ParseError> {
        let len = data.len();
        if len == header.mlen as _ {
            Ok(Message::from_data(header.kind, data)?)
        } else {
            Err(ParseError::WrongByteCount(len, header.mlen as _))
        }
    }
}

fn write_and_get_len<T: ProtoMessage>(v: &mut Vec<u8>, m: &T) -> u32 {
    m.write_to_vec(v).unwrap();
    (v.len() - 8) as u32
}

define_proto_variant! { Query,
    Hello : hello,
    Event : event,
}

define_proto_variant! { Response,
    Welcome : welcome,
    GameStarted : gameStarted,
    ConnectedPlayers : connectedPlayers,
    Event : event,
}

pub trait Action: Clone + Send + 'static {
    type Proto: ProtoMessage;

    fn to_proto(s: Self) -> Self::Proto;
    fn from_proto(p: Self::Proto) -> Self;

    fn get_unrecognized(&self) -> Option<&Self::Proto>;
}

#[derive(Debug, Copy, Clone, Default)]
pub struct Header {
    kind: u32,
    mlen: u32,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_hex(s: &str) -> Result<ServerMessage, ParseError> {
        let data = hex::decode(s).unwrap();
        let header = Header::parse(&data[0..8])?;
        Message::parse(&header, &data[8..])
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
