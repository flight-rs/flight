// Crates
#[macro_use]
extern crate log;
extern crate rust_webvr as webvr;
extern crate clap;
extern crate simplelog;
#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate cgmath;
extern crate obj as wavefront;
extern crate fnv;

use webvr::{VRServiceManager, VRLayer, VRFramebufferAttributes};
use simplelog::{Config, TermLogger, LogLevelFilter};
use clap::{Arg, App};
use gfx::{handle, Factory, texture, Encoder, Device};
use gfx::format::*;
use gfx_device_gl::{NewTexture, Resources};
use gfx::memory::Typed;

mod shaders;
mod app;
mod defines;
mod object;
mod load;
mod style;

fn main() {
    // Logging setup
    TermLogger::init(LogLevelFilter::Info, Config::default()).unwrap();

    // Command line arguments
    let matches = App::new("VR")
        .arg(Arg::with_name("mock")
            .short("m")
            .long("mock")
            .help("Use mock VR API"))
        .get_matches();

    // VR init
    let mut vrsm = VRServiceManager::new();
    if matches.is_present("mock") {
        vrsm.register_mock();
    } else {
        vrsm.register_defaults();
    }

    // Window manager stuff
    let mut events_loop = glutin::EventsLoop::new();
    let window_builder = glutin::WindowBuilder::new()
        .with_visibility(false);
    let context = glutin::ContextBuilder::new();
    // Fuuny thing I found here: changing `_window` to `_` (ignoring it) makes everything explode because of early drop.
    let (_window, mut device, mut factory, _, _) =
        gfx_window_glutin::init::<Rgba8, DepthStencil>(window_builder, context, &events_loop);

    // Get the display
    let display = match vrsm.get_displays().get(0) {
        Some(d) => d.clone(),
        None => {
            error!("No VR display present, exiting");
            return
        }
    };
    let gamepads = vrsm.get_gamepads();

    let display_data = display.borrow().data();
    info!("VR Device: {}", display_data.display_name);

    // Get some frame sizeing information
    let render_width = display_data.left_eye_parameters.render_width as u16;
    let render_height = display_data.left_eye_parameters.render_height as u16;
    let left_clip = gfx::Rect { x: 0, y: 0, w: render_width, h: render_height };
    let right_clip = gfx::Rect { x: render_width, y: 0, w: render_width, h: render_height };

    // Setup GFX utility stuff
    let mut manager = handle::Manager::new();
    let mut encoder: Encoder<Resources, _> = factory.create_command_buffer().into();

    // Create texture to render to
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

    let surface = factory.view_texture_as_render_target::<(R8_G8_B8_A8, Unorm)>(&tex, 0, None).unwrap();
    let mut application = app::App::new(surface, &mut factory);
    application.set_gamepads(gamepads.clone());

    // HMD (head-mounted display) layer information
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

    // Main loop
    let mut running = true;
    while running {
        let data = {
            let mut d = display.borrow_mut();
            d.sync_poses();
            app::DrawParams {
                frame: d.synced_frame_data(app::NEAR_PLANE, app::FAR_PLANE),
                display: d.data(),
                clip: (left_clip, right_clip),
            }
        };

        // Draw frame
        application.draw(&mut encoder, data);
        // Send instructions to OpenGL
        // TODO: Move flush to separate thread
        encoder.flush(&mut device);

        // Send resulting texture to VR device
        {
            let mut d = display.borrow_mut();
            d.render_layer(&layer);
            d.submit_frame();
        }

        // Cleanup GFX data
        device.cleanup();

        // Window Events
        events_loop.poll_events(|event| {
            match event {
                // process events here
                glutin::Event::WindowEvent { event: glutin::WindowEvent::Closed, .. } => 
                    running = false,
                _ => ()
            }
        });
    }

    display.borrow_mut().stop_present();
}
