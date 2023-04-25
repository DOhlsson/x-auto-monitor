extern crate nix;
extern crate udev;

use std::os::unix::prelude::AsRawFd;

use nix::poll::{poll, PollFd, PollFlags};
use udev::MonitorBuilder;
use udev::MonitorSocket;

pub struct Udev {
    pollfd: PollFd,
    socket: MonitorSocket,
}

impl Udev {
    pub fn new() -> Udev {
        let builder = MonitorBuilder::new().unwrap();
        let builder = builder.match_subsystem("drm").unwrap();
        let socket = builder.listen().unwrap();

        let fd = socket.as_raw_fd();
        let events = PollFlags::POLLIN;
        let pollfd = PollFd::new(fd, events);

        Udev { pollfd, socket }
    }

    pub fn wait(&self) -> () {
        poll(&mut [self.pollfd], -1).unwrap();
        let mut iter = self.socket.iter();

        let _res = iter.next().unwrap();
        //println!("udev {:#?}", _res);
    }
}

