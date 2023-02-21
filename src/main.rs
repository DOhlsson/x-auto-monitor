extern crate nix;
extern crate udev;

use nix::poll::{poll, PollFd, PollFlags};

use std::os::unix::prelude::AsRawFd;
use udev::MonitorBuilder;

fn main() {
    println!("Hello, world!");

    let builder = MonitorBuilder::new().unwrap();
    let builder = builder.match_subsystem("drm").unwrap();
    let socket = builder.listen().unwrap();
    let mut iter = socket.iter();

    let (conn, screen_num) = x11rb::connect(None).unwrap();

    println!("screen {screen_num}");

    loop {
        let fd = socket.as_raw_fd();
        let events = PollFlags::POLLIN;
        let pollfd = PollFd::new(fd, events);
        poll(&mut [pollfd], -1).unwrap();

        let res = iter.next().unwrap();
        println!("res {:#?}", res);

    }
}
