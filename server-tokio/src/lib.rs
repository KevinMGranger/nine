pub use nine::p2000::tagged::{Tattach, Tversion, Tauth, Twalk, Tstat, Topen, Tread};
pub use nine::p2000::untagged::{Rversion, Rattach, Rauth, Rwalk, Rstat, Ropen, Rread};
use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncWrite};
use nine::de::*;
use nine::p2000::*;
use nine::ser::*;
use std::collections::{HashMap, HashSet};
use std::env::{args, var};

mod errors;

pub use errors::*;

struct MessagePump<R, W> {
    reader: R,
    writer: W
}

impl <R: AsyncRead, W: AsyncWrite> MessagePump<R, W> {
    async fn read_a<'de, T: Deserialize<'de>>(&self) -> Result<T, DeError> {
        unimplemented!()        
    }
}

enum TagStatus {
    /// The tag has either been responded to or hasn't been recieved
    Unused,
    /// A message for this tag has been received, and we're waiting for the response to be generated
    ProcessingMessage,
    /// The response has been generated and we're sending it right now
    SendingResponse,
    /// A cancellation request has been received
    CancellationRequested
}


macro_rules! match_message {
    {
        ($self:ident, $serializer:ident, $received_discriminant:ident)
        $($message_type:ty => $func:ident),*
    } => {
        match $received_discriminant {
            $(
            <$message_type as MessageTypeId>::MSG_TYPE_ID => {
                let msg: $message_type = $self.read_a().unwrap();
                let tag = msg.tag;
                println!("<{:?}", &msg);
                let type_id = match $self.$func(msg) {
                    Ok(response) => {
                        println!(">{:?}", &response);
                        response.serialize(&mut $serializer).unwrap();
                        response.msg_type_id()
                    },
                    Err(ServerError::NonFatal { msg: response, .. }) => {
                        let response = Rerror { tag: tag, ename: format!("{}", response).into() };
                        println!(">{:?}", &response);
                        response.serialize(&mut $serializer).unwrap();
                        response.msg_type_id()
                    },
                    e => { e.unwrap(); 0}
                };

                let buf = $serializer.into_writer().into_inner();

                $self.write_msg(type_id, buf).unwrap();
            },
            )*
            _ => {
                eprintln!("unknown msg type {}", $received_discriminant);
            }
        }
    }
}


pub struct ServerContext;

#[async_trait]
pub trait NineP2000Server {
    async fn version(&mut self, ctx: ServerContext, msg: Tversion) -> NineResult<Rversion>;
    async fn attach(&mut self, ctx: ServerContext, msg: Tattach) -> NineResult<Rattach>;
    async fn auth(&mut self, ctx: ServerContext, msg: Tauth) -> NineResult<Rauth>;
    async fn walk(&mut self, ctx: ServerContext, msg: Twalk) -> NineResult<Rwalk>;
    async fn stat(&mut self, ctx: ServerContext, msg: Tstat) -> NineResult<Rstat>;
    async fn open(&mut self, ctx: ServerContext, msg: Topen) -> NineResult<Ropen>;
    async fn read(&mut self, ctx: ServerContext, msg: Tread) -> NineResult<Rread>;
}
