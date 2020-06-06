use std::io;

use futures::{
    channel::{mpsc, oneshot},
    future::FutureExt,
    pin_mut, select,
    sink::SinkExt,
    stream::StreamExt,
};
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream, ToSocketAddrs,
    },
    task::JoinHandle,
};

use ttr_protocol::{Message, ParseError};

pub struct Connection {
    read_task: Option<(JoinHandle<Result<(), ConnectionError>>, oneshot::Sender<()>)>,
    write_task: Option<(JoinHandle<Result<(), ConnectionError>>, oneshot::Sender<()>)>,
}

#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error(transparent)]
    ProtocolError(#[from] ParseError),
}

impl Connection {
    pub fn from_stream(
        s: TcpStream,
    ) -> (Connection, mpsc::Receiver<Message>, mpsc::Sender<Message>) {
        let (reader, writer) = s.into_split();

        let (sender, close_write, write_handle) = writer_task(writer);
        let (receiver, close_read, read_handle) = reader_task(reader, sender.clone());

        (
            Connection {
                read_task: Some((read_handle, close_read)),
                write_task: Some((write_handle, close_write)),
            },
            receiver,
            sender,
        )
    }
}

pub async fn connect<A: ToSocketAddrs>(
    addr: A,
) -> io::Result<(Connection, mpsc::Receiver<Message>, mpsc::Sender<Message>)> {
    Ok(Connection::from_stream(TcpStream::connect(addr).await?))
}

fn writer_task(
    mut stream: OwnedWriteHalf,
) -> (
    mpsc::Sender<Message>,
    oneshot::Sender<()>,
    JoinHandle<Result<(), ConnectionError>>,
) {
    let (sender, mut receiver) = mpsc::channel(16);
    let (close_sender, close_receiver) = oneshot::channel();
    let handle = tokio::spawn(async move {
        let close = close_receiver.fuse();
        pin_mut!(close);
        loop {
            select! {
                m = receiver.select_next_some() => {
                    send_message(&mut stream, m).await?
                },
                _ = close => {
                    break;
                },
            }
        }
        Ok(())
    });
    (sender, close_sender, handle)
}

async fn send_message(stream: &mut OwnedWriteHalf, m: Message) -> Result<(), ConnectionError> {
    let data = m.serialize();
    Ok(stream.write_all(&data).await?)
}

fn reader_task(
    mut stream: OwnedReadHalf,
    mut writer: mpsc::Sender<Message>,
) -> (
    mpsc::Receiver<Message>,
    oneshot::Sender<()>,
    JoinHandle<Result<(), ConnectionError>>,
) {
    let (mut sender, receiver) = mpsc::channel(16);
    let (close_sender, close_receiver) = oneshot::channel();
    let handle = tokio::spawn(async move {
        let close = close_receiver.fuse();
        pin_mut!(close);
        loop {
            let message = read_message(&mut stream).fuse();
            pin_mut!(message);

            select! {
                m = message => {
                    use Message::*;
                    let _ = match m? {
                        Heartbeat(h) => writer.send(Heartbeat(h)).await,
                        msg => sender.send(msg).await,
                    };
                },
                _ = close => {
                    break;
                },
            }
        }
        Ok(())
    });
    (receiver, close_sender, handle)
}

async fn read_message(stream: &mut OwnedReadHalf) -> Result<Message, ConnectionError> {
    let mut buf = [0u8; 8];
    stream.read_exact(&mut buf).await?;
    let header = Message::parse_header(&buf)?;

    let mut buf = vec![0u8; header.bytes_required()];
    stream.read_exact(&mut buf).await?;
    Ok(header.parse_message(&buf)?)
}
