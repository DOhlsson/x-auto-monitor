extern crate nix;
extern crate udev;

use std::os::unix::prelude::AsRawFd;

use nix::poll::{poll, PollFd, PollFlags};
use udev::MonitorBuilder;
use udev::MonitorSocket;
use x11rb::connection::Connection;
use x11rb::protocol::randr;
use x11rb::protocol::randr::ConnectionExt as _;
use x11rb::protocol::xproto::ConnectionExt as _;
use x11rb::rust_connection::RustConnection;

struct Udev {
    pollfd: PollFd,
    socket: MonitorSocket,
}

struct Randr {
    conn: RustConnection,
    root: u32,
}

#[derive(Debug)]
struct Output {
    output: u32,
    name: String,
    connection: randr::Connection,
    mode: Option<u32>,
    edid: Vec<u8>,
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

    fn get(&self) -> Result<Vec<Output>, Box<dyn std::error::Error>> {
        let resource = self.conn.randr_get_screen_resources(self.root)?.reply()?;
        let modes = Vec::from(resource.modes);
        let modes: Vec<u32> = modes.iter().map(|m| m.id).collect();
//        println!("modes: {:?}", modes);

        let mut outputs: Vec<Output> = Vec::new();

        for o in resource.outputs.iter() {
            let output_info = self
                .conn
                .randr_get_output_info(*o, resource.config_timestamp)?
                .reply()?;
            // println!("Output {o}: {output_info:#?}");

            let mut mode = None;
            if output_info.crtc != 0 {
                let crtc_info = self
                    .conn
                    .randr_get_crtc_info(output_info.crtc, resource.config_timestamp)?
                    .reply()?;
                // println!("Crtc {crtc_info:#?}");
                mode = Some(crtc_info.mode);
            }

            let output_properties = self.conn.randr_list_output_properties(*o)?.reply()?;
            let atom_name = self.conn.get_atom_name(69)?.reply()?;
            let atom_name = String::from_utf8(atom_name.name).unwrap();
//             println!("properties: {output_properties:#?}");
//             println!("atom_name: {atom_name}");

            let output_property = self.conn.randr_get_output_property(
                *o,
                69,
                0u32,
                0,
                100,
                false,
                false,
            )?.reply()?;
//            println!("output property: {output_property:02X?}");

            let output: Output = Output {
                output: *o,
                name: String::from_utf8(output_info.name).unwrap(),
                mode,
                edid: output_property.data,
                connection: output_info.connection.into(),
            };

            outputs.push(output);
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
    loop {
        let res = randr.get().unwrap();
        println!("displays {res:#?}");

        udev.wait();
        println!("UDEV!");
    }
}
