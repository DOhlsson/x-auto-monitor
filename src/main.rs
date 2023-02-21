extern crate nix;
extern crate udev;

use std::os::unix::prelude::AsRawFd;

use nix::poll::{poll, PollFd, PollFlags};
use udev::MonitorBuilder;
use udev::MonitorSocket;
use x11rb::connection::Connection;
use x11rb::protocol::randr::ConnectionExt;
use x11rb::rust_connection::ConnectionError;
use x11rb::rust_connection::RustConnection;

struct Udev {
    pollfd: PollFd,
    socket: MonitorSocket,
}

struct Randr {
    conn: RustConnection,
    root: u32,
}

impl Randr {
    fn new() -> Randr {
        let (conn, screen_num) = x11rb::connect(None).unwrap();

        let setup = &conn.setup();
        let screen = &setup.roots[screen_num];
        //println!("

        println!("screen_num {screen_num}");
        println!("screen.root {}", screen.root);

        let version = conn.randr_query_version(1, 6).unwrap().reply().unwrap();
        println!("{version:#?}");

        Randr {
            root: screen.root,
            conn,
        }
    }

    fn get(&self) -> Result<Vec<(u32, u8)>, Box<dyn std::error::Error>> {
        let resource = self.conn.randr_get_screen_resources(self.root)?.reply()?;
        //println!("outputs: {:?}", resource.outputs);

        let mut outputs: Vec<(u32, u8)> = Vec::new();

        for o in resource.outputs.iter() {
            let res = self
                .conn
                .randr_get_output_info(*o, resource.config_timestamp)?
                .reply()?;
            //println!("Output {o}: {res:#?}");

            outputs.push((*o, res.connection.into()));
        }

        Ok(outputs)
    }
}

impl Udev {
    fn new() -> Udev {
        let builder = MonitorBuilder::new().unwrap();
        let builder = builder.match_subsystem("drm").unwrap();
        let socket = builder.listen().unwrap();

        let fd = socket.as_raw_fd();
        let events = PollFlags::POLLIN;
        let pollfd = PollFd::new(fd, events);

        Udev { pollfd, socket }
    }

    fn wait(&self) -> () {
        poll(&mut [self.pollfd], -1).unwrap();
        let mut iter = self.socket.iter();

        let _res = iter.next().unwrap();
        //println!("udev {:#?}", _res);
    }
}

fn main() {
    println!("Hello, world!");

    let randr = Randr::new();
    let udev = Udev::new();

    let res = randr.get().unwrap();
    println!("displays {res:#?}");

    loop {
        udev.wait();
        println!("UDEV!");
        randr.get().unwrap();
        println!("displays {res:#?}");
    }
}
