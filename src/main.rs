// Crates
#[macro_use]
extern crate log;

extern crate rust_webvr;
extern crate clap;
extern crate simplelog;
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glutin;

// Libraries
use rust_webvr::{VRServiceManager, VRLayer, VRFramebufferAttributes};
use simplelog::{Config, TermLogger, LogLevelFilter};
use clap::{Arg, App};
use gfx::{handle, Factory, texture, Encoder};
use gfx::format::*;
use gfx_device_gl::{NewTexture, Resources};
use gfx::memory::Typed;
use glutin::GlContext;

fn main() {
    // logging setup
    TermLogger::init(LogLevelFilter::Trace, Config::default()).unwrap();

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

    let mut events_loop = glutin::EventsLoop::new();
    let window_builder = glutin::WindowBuilder::new();
    let context = glutin::ContextBuilder::new();
    let (window, mut device, mut factory, rtv, stv) = 
        gfx_window_glutin::init::<Rgba8, DepthStencil>(window_builder, context, &events_loop);

    // Get the display
    let displays = vrsm.get_displays();
    let display = displays.get(0).unwrap();

    let display_data = display.borrow().data();
    trace!("VRDisplay: {:?}", display_data);

    let render_width = display_data.left_eye_parameters.render_width as u16;
    let render_height = display_data.left_eye_parameters.render_height as u16;

    let mut manager = handle::Manager::new();
    let mut encoder: Encoder<Resources, _> = factory.create_command_buffer().into();

    // create a texture
    let (tex, texture_id) = {
        let desc = texture::Info {
            kind: texture::Kind::D2(render_width * 2, render_height, texture::AaMode::Single),
            levels: 1,
            format: R8_G8_B8_A8::get_surface_type(),
            bind: gfx::RENDER_TARGET | gfx::SHADER_RESOURCE,
            usage: gfx::memory::Usage::Data,
        };

        let raw = factory.create_texture_raw(desc, Some(ChannelType::Unorm), None).unwrap();
        let texture_id = match *manager.ref_texture(&raw) {
            NewTexture::Texture(t) => t as u32,
            _ => panic!("Something went wrong here"),
        };
        (Typed::new(raw), texture_id)
    };

    info!("{}", texture_id);

    let surface = factory.view_texture_as_render_target::<(R8_G8_B8_A8, Unorm)>(&tex, 0, None).unwrap();

    // Render to HMD
    let layer = VRLayer {
        texture_id: texture_id,
        .. Default::default()
    };

    // Configure VR presentation parameters
    let attributes = VRFramebufferAttributes {
        multiview: false,
        depth: false,
        multisampling: false,
    };
    display.borrow_mut().start_present(Some(attributes));

    loop {
        let mut d = display.borrow_mut();
        d.sync_poses();

        encoder.clear(&surface, [1.0, 0.0, 0.0, 1.0]);
        encoder.clear(&rtv, [0.0, 0.0, 1.0, 1.0]);
        encoder.flush(&mut device);

        d.render_layer(&layer);
        d.submit_frame();

        window.swap_buffers().unwrap();

        trace!("{:?}", d.synced_frame_data(0.1, 100.0).pose);

        // Window Events
        events_loop.poll_events(|event| {
            match event {
                // process events here
                _ => ()
            }
        });
    }
}
