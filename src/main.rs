// Crates
#[macro_use]
extern crate log;

extern crate rust_webvr;
extern crate clap;
extern crate simplelog;

// Libraries
use rust_webvr::VRServiceManager;

use simplelog::{Config, TermLogger, LogLevelFilter};
use clap::{Arg, App};

fn main() {
    // logging setup
    TermLogger::init(LogLevelFilter::Warn, Config::default());

    // check for mock
    let matches = App::new("VR")
        .arg(Arg::with_name("mock")
            .short("m")
            .long("mock")
            .help("Use mock VR API"))
        .get_matches();

    // Init
    let mut vrsm = VRServiceManager::new();
    if matches.is_present("mock") {
        vrsm.register_mock();
    } else {
        vrsm.register_defaults();
    }

    let displays = vrsm.get_displays();
    println!("{}", displays.len());

    // Select first display
    let display = displays.get(0).unwrap();

    let display_data = display.borrow().data();
    println!("VRDisplay: {:?}", display_data);

    loop {
        let mut d = display.borrow_mut();
        d.sync_poses();

        println!("{:?}", d.synced_frame_data(0.1, 100.0).pose);
    }
}
