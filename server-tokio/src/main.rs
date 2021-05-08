use nine_server_tokio::*;
use async_trait::async_trait;

struct Memfs {
    msize: u32,
    
}

#[async_trait]
impl NineP2000Server for Memfs {
    async fn version(&mut self, ctx: ServerContext, msg: Tversion) -> NineResult<Rversion> {
        self.msize = std::cmp::min(msg.msize, self.msize);
        if msg.version != "9p2000" {
            return Ok(Rversion {
                msize: self.msize,
                version: "unknown".into()
            })
        }

        return Ok(Rversion {
            msize: self.msize,
            version: "9p2000".into()
        })
    }
    async fn attach(&mut self, ctx: ServerContext, msg: Tattach) -> NineResult<Rattach> { 
        unimplemented!()
    }
    async fn auth(&mut self, ctx: ServerContext, msg: Tauth) -> NineResult<Rauth> { unimplemented!() }
    async fn walk(&mut self, ctx: ServerContext, msg: Twalk) -> NineResult<Rwalk> { unimplemented!() }
    async fn stat(&mut self, ctx: ServerContext, msg: Tstat) -> NineResult<Rstat> { unimplemented!() }
    async fn open(&mut self, ctx: ServerContext, msg: Topen) -> NineResult<Ropen> { unimplemented!() }
    async fn read(&mut self, ctx: ServerContext, msg: Tread) -> NineResult<Rread> { unimplemented!() }
}

#[tokio::main]
async fn main() {

}