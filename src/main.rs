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
extern crate image;

use webvr::{VRServiceManager, VRLayer, VRFramebufferAttributes};
use simplelog::{Config, TermLogger, LogLevelFilter};
use clap::{Arg, App};
use gfx::{handle, Factory, texture, Device};
use gfx::format::*;
use gfx_device_gl::{NewTexture};
use gfx::memory::Typed;
use glutin::GlContext;
use cgmath::prelude::*;
use cgmath::Matrix4;

mod app;
pub mod lib;

use lib::context;

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
    let mut vrsm = VRServiceManager::new();
    if mock {
        vrsm.register_mock();
    } else {
        vrsm.register_defaults();
    }

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

    // Window manager stuff
    let mut events_loop = glutin::EventsLoop::new();
    let window_builder = glutin::WindowBuilder::new()
        .with_visibility(false)
        .with_dimensions(render_width as u32, render_height as u32)
        .with_title("Mock OpenVR Display");
    let context = glutin::ContextBuilder::new();
    // Fuuny thing I found here: changing `_window` to `_` (ignoring it) makes everything explode because of early drop.
    let (window, mut device, mut factory, wcolor, wdepth) =
        gfx_window_glutin::init::<Rgba8, DepthStencil>(window_builder, context, &events_loop);

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
        let mut manager = handle::Manager::new();
        let texture_id = match *manager.ref_texture(&raw) {
            NewTexture::Texture(t) => t as u32,
            _ => panic!("Something went wrong here"),
        };
        (Typed::new(raw), texture_id)
    };

    // Create depth buffer
    let (.., depth) = factory.create_depth_stencil(render_width * 2, render_height).unwrap();

    let surface = factory.view_texture_as_render_target::<(R8_G8_B8_A8, Unorm)>(&tex, 0, None).unwrap();
    let mut application = match app::App::new(&mut factory) {
        Ok(a) => a,
        Err(e) => {
            error!("Could not start application: {}", e);
            return
        },
    };
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

    // setup context
    let mut ctx = context::DrawContext {
        encoder: factory.create_command_buffer().into(),
        color: if mock { wcolor } else { surface },
        depth: if mock { wdepth } else { depth },
        left: context::EyeContext {
            view: Matrix4::identity(),
            proj: Matrix4::identity(),
            xoffset: -0.5,
            clip: left_clip,
        },
        right: context::EyeContext {
            view: Matrix4::identity(),
            proj: Matrix4::identity(),
            xoffset: 0.5,
            clip: right_clip,
        },
    };

    if mock { window.show() }

    // Main loop
    let mut running = true;
    while running {
        let (data, frame) = {
            let mut d = display.borrow_mut();
            d.sync_poses();
            (d.data(), d.synced_frame_data(app::NEAR_PLANE, app::FAR_PLANE))
        };

        // Update context
        ctx.left.view.clone_from((&frame.left_view_matrix).into());
        ctx.left.proj.clone_from((&frame.left_projection_matrix).into());
        ctx.right.view.clone_from((&frame.right_view_matrix).into());
        ctx.right.proj.clone_from((&frame.right_projection_matrix).into());

        // Update clipping if mock
        if mock {
            let (w, h, ..) = ctx.color.get_dimensions();
            ctx.left.clip = gfx::Rect { x: 0, y: 0, w: w / 2, h: h };
            ctx.right.clip = gfx::Rect { x: w / 2, y: 0, w: w / 2, h: h };
        }

        // Draw frame
        application.draw(&mut ctx, &data);

        // Send instructions to OpenGL
        // TODO: Move flush to separate thread
        ctx.encoder.flush(&mut device);

        // Send resulting texture to VR device
        {
            let mut d = display.borrow_mut();
            d.render_layer(&layer);
            d.submit_frame();
        }
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

    display.borrow_mut().stop_present();
}
