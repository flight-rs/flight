// Crates
extern crate rust_webvr;
extern crate clap;

// Libraries
use rust_webvr::VRServiceManager;
use clap::{Arg, App, SubCommand};

fn main() {
    // check for mock
    let matches = App::new("VR")
        .arg(Arg::with_name("mock")
            .short("m")
            .long("mock")
            .help("Use mock vr api"))
        .get_matches();
    
    // Init
    let mut vrsm = VRServiceManager::new();
    if matches.is_present("mock") {
        vrsm.register_mock();
    } else {
        vrsm.register_defaults();
    }

    println!("{}", vrsm.get_displays().len());

}
