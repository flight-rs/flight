// Crates
#[macro_use]
extern crate log;
extern crate clap;
extern crate simplelog;
extern crate flight as lib;
extern crate gfx;
extern crate nalgebra;
extern crate glutin;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;

use simplelog::{Config, TermLogger, LogLevelFilter};
use clap::{Arg, App};
use gfx::{handle, Factory, texture, Device};
use gfx::format::*;
use gfx_device_gl::{NewTexture};
use gfx::memory::Typed;
use glutin::GlContext;

mod app;

use lib::draw;
use lib::vr::*;

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
    let mock = matches.is_present("mock");

    // VR init
    let mut vrctx = match if mock { VrContext::mock() } else { VrContext::new() } {
        Some(v) => v,
        None => {
            error!("Could not create VrContext, exiting");
            return
        },
    };

    // Set clipping planes
    vrctx.near = app::NEAR_PLANE;
    vrctx.far = app::FAR_PLANE;

    // Get some frame sizeing information
    let (render_width, render_height) = vrctx.retrieve_size();

    // Window manager stuff
    let mut events_loop = glutin::EventsLoop::new();
    let window_builder = glutin::WindowBuilder::new()
        .with_visibility(false)
        .with_dimensions(render_width, render_height)
        .with_title("Mock OpenVR Display");
    let context = glutin::ContextBuilder::new();
    // Fuuny thing I found here: changing `_window` to `_` (ignoring it) makes everything explode because of early drop.
    let (window, mut device, mut factory, wcolor, wdepth) =
        gfx_window_glutin::init::<Rgba8, DepthStencil>(window_builder, context, &events_loop);

    // Create texture to render to
    let (tex, texture_id) = {
        let desc = texture::Info {
            kind: texture::Kind::D2(render_width as u16, render_height as u16, texture::AaMode::Single),
            levels: 1,
            format: R8_G8_B8_A8::get_surface_type(),
            bind: gfx::RENDER_TARGET | gfx::SHADER_RESOURCE,
            usage: gfx::memory::Usage::Data,
        };

        let raw = factory.create_texture_raw(desc, Some(ChannelType::Unorm), None).unwrap();
        let mut manager = handle::Manager::new();
        let texture_id = match *manager.ref_texture(&raw) {
            NewTexture::Texture(t) => t as u32,
            _ => panic!("Something went wrong here"),
        };
        (Typed::new(raw), texture_id)
    };
    vrctx.set_texture(texture_id);

    // Create depth buffer
    let (.., depth) = factory.create_depth_stencil(render_width as u16, render_height as u16).unwrap();

    let surface = factory.view_texture_as_render_target::<(R8_G8_B8_A8, Unorm)>(&tex, 0, None).unwrap();
    let mut application = match app::App::new(&mut factory) {
        Ok(a) => a,
        Err(e) => {
            error!("Could not start application: {}", e);
            return
        },
    };

    // setup context
    let mut ctx = draw::DrawParams {
        encoder: factory.create_command_buffer().into(),
        color: if mock { wcolor } else { surface },
        depth: if mock { wdepth } else { depth },
        left: Default::default(),
        right: Default::default(),
    };

    if mock { window.show() }

    // Main loop
    vrctx.start();
    let mut running = true;
    while running {
        let vrm = vrctx.sync();
        let hmd = match vrm.hmd() {
            Some(h) => h.clone(),
            None => continue,
        };

        // Update context
        running = !vrm.exit;
        ctx.left = hmd.left;
        ctx.right = hmd.right;

        // Draw frame
        application.draw(&mut ctx, &vrm);

        // Send instructions to OpenGL
        // TODO: Move flush to separate thread
        ctx.encoder.flush(&mut device);

        // Send resulting texture to VR device
        vrm.submit(&mut vrctx);
        if mock { window.swap_buffers().unwrap() }

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
    vrctx.stop();
}
