use std::collections::HashSet;

use x11rb::connection::Connection;
use x11rb::protocol::randr;
use x11rb::protocol::randr::ConnectionExt as _;
use x11rb::protocol::randr::Rotation;
use x11rb::protocol::xproto;
use x11rb::protocol::xproto::ConnectionExt as _;
use x11rb::rust_connection::RustConnection;

pub struct Randr {
    conn: RustConnection,
    root: u32,
}

#[derive(Debug)]
struct Output {
    output_num: u32,
    name: String,
    connection: randr::Connection,
    mode: Option<u32>,
    edid: String,
    crtc: u32,
    crtcs: Vec<u32>,
    modes: Vec<u32>,
}

impl Randr {
    pub fn new() -> Randr {
        let (conn, screen_num) = x11rb::connect(None).unwrap();

        let setup = &conn.setup();
        let screen = &setup.roots[screen_num];
        //println!("roots {:#?}", &setup.roots);

        println!("screen_num {screen_num}");
        println!("screen.root {}", screen.root);

        let version = conn.randr_query_version(1, 6).unwrap().reply().unwrap();
        println!("{version:#?}");

        Randr {
            root: screen.root,
            conn,
        }
    }

    fn get_output(
        &self,
        config_timestamp: xproto::Timestamp,
        output_num: &u32,
    ) -> Result<Output, Box<dyn std::error::Error>> {
        let output_info = self
            .conn
            .randr_get_output_info(*output_num, config_timestamp)?
            .reply()?;
        println!("Output {output_num}: {output_info:#?}");

        let mut mode = None;
        if output_info.crtc != 0 {
            let crtc_info = self
                .conn
                .randr_get_crtc_info(output_info.crtc, config_timestamp)?
                .reply()?;
            // println!("Crtc {crtc_info:#?}");
            mode = Some(crtc_info.mode);
        }

        let output_properties = self
            .conn
            .randr_list_output_properties(*output_num)?
            .reply()?;

        let atom_name = self.conn.get_atom_name(69)?.reply()?;
        let atom_name = String::from_utf8(atom_name.name).unwrap();
        //             println!("properties: {output_properties:#?}");
        //             println!("atom_name: {atom_name}");

        let output_property = self
            .conn
            .randr_get_output_property(*output_num, 69, 0u32, 0, 100, false, false)?
            .reply()?;
        //            println!("output property: {output_property:02X?}");

        let edid: Vec<String> = output_property
            .data
            .iter()
            .map(|i| format!("{:02X}", i))
            .collect();

        let edid = edid.join("");

        let output: Output = Output {
            output_num: *output_num,
            name: String::from_utf8(output_info.name).unwrap(),
            mode,
            modes: output_info.modes,
            edid,
            connection: output_info.connection.into(),
            crtc: output_info.crtc,
            crtcs: Vec::from(output_info.crtcs),
        };

        return Ok(output);
    }

    fn get_all(&self) -> Result<Vec<Output>, Box<dyn std::error::Error>> {
        let resource = self.conn.randr_get_screen_resources(self.root)?.reply()?;
        //println!("Resource: {:#?}", resource);

        let modes = Vec::from(resource.modes);
        let modes: Vec<u32> = modes.iter().map(|m| m.id).collect();
        //println!("modes: {:?}", modes);

        let mut outputs: Vec<Output> = Vec::new();

        for o in resource.outputs.iter() {
            let output = self.get_output(resource.timestamp, o).unwrap();

            outputs.push(output);
        }

        Ok(outputs)
    }

    fn set_best_mode_old(&self, output: &Output) -> Result<(), Box<dyn std::error::Error>> {
        let resource = self.conn.randr_get_screen_resources(self.root)?.reply()?;
        let outputs = [output.output_num];

        println!("foo {}", self.root);
        let screen = self.conn.randr_get_screen_size_range(self.root)?.reply()?;
        println!("screen {:#?}", screen);
        self.conn
            .randr_set_screen_size(self.root, 3840, 1080, 818, 286)?;
        println!("bar");

        // TODO if output.crtc is 0, find a free crtc not in use
        let res = self
            .conn
            .randr_set_crtc_config(
                81,
                resource.timestamp,
                resource.config_timestamp,
                1920, // save these
                0,    // this one too
                output.modes[0],
                Rotation::ROTATE0,
                &outputs,
            )?
            .reply()?;

        println!("res {:#?}", res);

        let resource = self.conn.randr_get_screen_resources(self.root)?.reply()?;

        return Ok(());
    }

    pub(crate) fn set_best_mode(&self, output_num: &u32) -> Result<(), Box<dyn std::error::Error>> {
        let resource = self.conn.randr_get_screen_resources(self.root)?.reply()?;

        let output = self.get_output(resource.timestamp, output_num).unwrap();
        let outputs = [*output_num];

        let mut available_crtc: Option<u32> = None;

        for crtc in output.crtcs {
            let crtc_info = self.conn.randr_get_crtc_info(crtc, resource.config_timestamp)?.reply()?;

            if crtc_info.outputs.len() == 0 {
                available_crtc = Some(crtc);
                break;
            }
        }

        //let screen = self.conn.randr_get_screen_size_range(self.root)?.reply()?;
        self.conn.randr_set_screen_size(self.root, 3840, 1080, 818, 286)?;

        let res = self
            .conn
            .randr_set_crtc_config(
                available_crtc.unwrap(),
                resource.timestamp,
                resource.config_timestamp,
                1920, // save these
                0,    // this one too
                output.modes[0],
                Rotation::ROTATE0,
                &outputs,
            )?
            .reply()?;

        return Ok(());
    }

    pub(crate) fn turn_off(&self, output_num: &u32) -> Result<(), Box<dyn std::error::Error>> {
        let resource = self.conn.randr_get_screen_resources(self.root)?.reply()?;

        let output = self.get_output(resource.timestamp, output_num).unwrap();

        //let screen = self.conn.randr_get_screen_size_range(self.root)?.reply()?;
        self.conn.randr_set_screen_size(self.root, 1920, 1080, 818, 286)?;

        let res = self
            .conn
            .randr_set_crtc_config(
                output.crtc,
                resource.timestamp,
                resource.config_timestamp,
                1920, // save these
                0,    // this one too
                0,
                Rotation::ROTATE0,
                &[],
            )?
            .reply()?;

        return Ok(());
    }

    pub(crate) fn get_active(&self) -> Result<HashSet<u32>, Box<dyn std::error::Error>> {
        let resource = self.conn.randr_get_screen_resources(self.root)?.reply()?;

        let mut outputs = Vec::new();

        for output_num in resource.outputs.iter() {
            let output_info = self
                .conn
                .randr_get_output_info(*output_num, resource.config_timestamp)?
                .reply()?;

            if output_info.connection == randr::Connection::CONNECTED {
                outputs.push(*output_num);
            }
        }

        return Ok(HashSet::from_iter(outputs));
    }
}
