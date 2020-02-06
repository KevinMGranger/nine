extern crate byteorder;
extern crate nine;

use byteorder::{LittleEndian, WriteBytesExt};
use nine::{de::*, p2000::*, ser::*};
use std::env::{args, var};
use std::io::{stdin, stdout, BufRead, Read, Write};
use std::os::unix::net::UnixStream; // TODO: abstract so this still works on windows

trait SimpleClient {
    fn version(&mut self);
    fn attach(&mut self, uname: String) -> u32;
    fn walk(&mut self, fid: u32, newfid: u32, wname: Vec<String>) -> Vec<Qid>;
    fn stat(&mut self, fid: u32) -> Stat;
    fn open(&mut self, fid: u32, mode: OpenMode) -> (Qid, u32);
    fn read(&mut self, fid: u32, offset: u64, count: u32) -> Vec<u8>;
    fn write(&mut self, fid: u32, offset: u64, data: Vec<u8>) -> u32;
}

struct Client<Stream>
where
    Stream: Write,
    for<'a> &'a mut Stream: Read,
{
    stream: Stream,
    msg_buf: Vec<u8>,
    msize: u32,
}

impl<Stream: Write + Read> Client<Stream> {
    fn new(stream: Stream) -> Self {
        Client {
            stream,
            msg_buf: Vec::new(),
            msize: u32::max_value(),
        }
    }

    fn send_msg<T: Serialize + MessageTypeId>(&mut self, t: &T) -> Result<(), SerError> {
        self.msg_buf.truncate(0);
        let amt = into_vec(&t, &mut self.msg_buf)?;

        assert!(self.msize >= amt);
        self.stream.write_u32::<LittleEndian>(amt + 5)?;
        self.stream.write_u8(<T as MessageTypeId>::MSG_TYPE_ID)?;
        Ok(self.stream.write_all(&self.msg_buf[0..amt as usize])?)
    }

    fn read_msg<'de, T: Deserialize<'de> + MessageTypeId>(&mut self) -> Result<T, DeError> {
        let _size: u32 = self.read_a()?;
        let mtype: u8 = self.read_a()?;
        assert_eq!(mtype, <T as MessageTypeId>::MSG_TYPE_ID);

        self.read_a()
    }

    fn read_a<'de, T: Deserialize<'de>>(&mut self) -> Result<T, DeError> {
        from_reader(&mut self.stream)
    }
}

impl<Stream: Write + Read> SimpleClient for Client<Stream> {
    fn version(&mut self) {
        let tversion = Tversion {
            tag: 0,
            msize: self.msize,
            version: "9P2000".into(),
        };

        self.send_msg(&tversion).unwrap();
        let rversion: Rversion = self.read_msg().unwrap();

        assert_eq!(rversion.version, "9P2000");
        if rversion.msize > self.msize {
            self.msize = rversion.msize;
        }
    }

    fn attach(&mut self, uname: String) -> u32 {
        let attach = Tattach {
            tag: 0,
            fid: 0,
            afid: !0,
            uname: uname.into(),
            aname: "".into(),
        };

        self.send_msg(&attach).unwrap();
        self.read_msg::<Rattach>().unwrap();

        0
    }
    fn walk(&mut self, fid: u32, newfid: u32, wname: Vec<String>) -> Vec<Qid> {
        let walk = Twalk {
            tag: 0,
            fid,
            newfid,
            wname,
        };
        self.send_msg(&walk).unwrap();

        let rwalk: Rwalk = self.read_msg().unwrap();

        rwalk.wqid
    }
    fn stat(&mut self, fid: u32) -> Stat {
        let stat = Tstat { tag: 0, fid };
        self.send_msg(&stat).unwrap();
        let stat: Rstat = self.read_msg().unwrap();

        stat.stat
    }
    fn open(&mut self, fid: u32, mode: OpenMode) -> (Qid, u32) {
        let open = Topen { tag: 0, fid, mode };
        self.send_msg(&open).unwrap();
        let ropen: Ropen = self.read_msg().unwrap();

        (ropen.qid, ropen.iounit)
    }
    fn read(&mut self, fid: u32, offset: u64, count: u32) -> Vec<u8> {
        let read = Tread {
            tag: 0,
            fid,
            offset,
            count,
        };
        self.send_msg(&read).unwrap();
        let read: Rread = self.read_msg().unwrap();

        read.data
    }
    fn write(&mut self, fid: u32, offset: u64, data: Vec<u8>) -> u32 {
        let twrite = Twrite {
            tag: 0,
            fid,
            offset,
            data,
        };
        self.send_msg(&twrite).unwrap();
        let rwrite: Rwrite = self.read_msg().unwrap();

        rwrite.count
    }
}

fn main() {
    let mut dial = None;
    let mut auth = true;
    let mut subcommand = None;
    let mut path = None;
    let mut args = args().skip(1);
    loop {
        match args.next() {
            Some(x) => match x.as_ref() {
                "-a" => dial = Some(args.next().unwrap()),
                "-n" => auth = false,
                "-A" => unimplemented!(),
                other => {
                    subcommand = Some(other.to_string());
                    path = Some(args.next().unwrap());

                    if args.len() != 0 {
                        panic!("remaining items");
                    }
                    break;
                }
            },
            None => break,
        }
    }

    let path: Vec<String> = path
        .unwrap()
        .split('/')
        .filter(|&x| x != "")
        .map(|x| x.to_owned())
        .collect();

    if auth {
        unimplemented!();
    }

    // TODO: handle optional dial, although that requires -A support

    let mut client = Client::new(UnixStream::connect(dial.unwrap()).unwrap());

    let whoami = var("USER").unwrap();

    client.version();
    let root = client.attach(whoami);

    let mut stdout = stdout();
    match subcommand.unwrap().as_ref() {
        "read" => {
            let len = path.len();
            let fid = 1;
            let qids = client.walk(root, fid, path);
            assert_eq!(len, qids.len());

            let stat = client.stat(fid);
            client.open(fid, OpenMode::READ);

            let mut file_size = stat.length;
            let mut offset = 0;
            while offset < file_size {
                let bytes = client.read(fid, 0, u32::max_value().min(file_size as u32));
                offset += bytes.len() as u64;
                stdout.write(bytes.as_ref()).unwrap();
            }
        }
        "write" => {
            let len = path.len();
            let fid = 1;
            let qids = client.walk(root, fid, path);
            assert_eq!(len, qids.len());

            client.open(fid, OpenMode::WRITE);

            // TODO: -l for line by line only
            let stdin = stdin();
            let mut offset = 0;
            for mut line in stdin.lock().lines().map(Result::unwrap) {
                line.push_str("\n");
                offset += client.write(fid, offset, line.into_bytes()) as u64;
            }
        }
        "stat" => {
            // TODO: copy p9p format
            let len = path.len();
            let fid = 1;
            let qids = client.walk(root, fid, path);
            assert_eq!(len, qids.len());

            let stat = client.stat(fid);
            println!("{:?}", stat);
        },
        "rdwr" => {
            // TODO: how do you know when you have a 'line' from the file?
            unimplemented!()
        },
        "ls" => unimplemented!(),
        "mkdir" => unimplemented!(),
        "touch" => unimplemented!(),
        "chmod" => unimplemented!(),
        "rm" => unimplemented!(),
        "rmdir" => unimplemented!(),
        _ => panic!(),
    }
}
