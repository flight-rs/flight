use std::path::Path;
use std::time::Instant;
use gfx::{self, Factory};
use gfx::traits::FactoryExt;
use nalgebra::{self as na, UnitQuaternion, Similarity3, Translation3, Point2, Vector3};

use lib::{Texture, UberMesh, Error};
use lib::mesh::*;
use lib::load;
use lib::draw::{DrawParams, Painter, SolidStyle, UberStyle, UberMaterial};
use lib::vr::{primary, secondary, VrMoment, MappedController, Trackable};

pub const NEAR_PLANE: f64 = 0.1;
pub const FAR_PLANE: f64 = 1000.;
pub const BACKGROUND: [f32; 4] = [0.529, 0.808, 0.980, 1.0];
const PI: f32 = ::std::f32::consts::PI;
const PI2: f32 = 2. * PI;
const DEG: f32 = PI2 / 360.;

pub struct App<R: gfx::Resources> {
    solid: Painter<R, SolidStyle<R>>,
    uber: Painter<R, UberStyle<R>>,
    grid: Mesh<R, VertC, ()>,
    controller_grid: Mesh<R, VertC, ()>,
    arrow: Mesh<R, VertC, ()>,
    controller: UberMesh<R>,
    teapot: UberMesh<R>,
    start_time: Instant,
    primary: MappedController,
    secondary: MappedController,
}

fn grid_lines(count: u32, size: f32) -> MeshSource<VertC, ()> {
    let mut lines = Vec::new();
    let base_color = [0.2, 0.2, 0.2];
    let light_color = [0.8, 0.8, 0.8];
    let mid = count / 2;
    let rad = size / 2.;
    let mult = size / count as f32;
    for a in 0..(count + 1) {
        for b in 0..(count + 1) {
            let line_color = if a == mid && b == mid {
                [[1., 0., 0.],
                 [0., 1., 0.],
                 [0., 0., 1.]]
            } else if a % 2 == 0 && b % 2 == 0 { [base_color; 3] } else { [light_color; 3] };
            let a = a as f32 * mult - rad;
            let b = b as f32 * mult - rad;
            lines.push(VertC { pos: [-rad, a, b], color: line_color[0] });
            lines.push(VertC { pos: [rad, a, b], color: line_color[0] });
            lines.push(VertC { pos: [a, -rad, b], color: line_color[1] });
            lines.push(VertC { pos: [a, rad, b], color: line_color[1] });
            lines.push(VertC { pos: [a, b, -rad], color: line_color[2] });
            lines.push(VertC { pos: [a, b, rad], color: line_color[2] });
        }
    }

    let frac = 0.125 * mult;
    lines.push(VertC { pos: [rad - frac, 0., -frac], color: [1., 0., 0.] });
    lines.push(VertC { pos: [rad - frac, 0.,  frac], color: [1., 0., 0.] });
    lines.push(VertC { pos: [rad - frac, -frac, 0.], color: [1., 0., 0.] });
    lines.push(VertC { pos: [rad - frac,  frac, 0.], color: [1., 0., 0.] });
    lines.push(VertC { pos: [0., rad - frac, -frac], color: [0., 1., 0.] });
    lines.push(VertC { pos: [0., rad - frac,  frac], color: [0., 1., 0.] });
    lines.push(VertC { pos: [-frac, rad - frac, 0.], color: [0., 1., 0.] });
    lines.push(VertC { pos: [ frac, rad - frac, 0.], color: [0., 1., 0.] });
    lines.push(VertC { pos: [-frac, 0., rad - frac], color: [0., 0., 1.] });
    lines.push(VertC { pos: [ frac, 0., rad - frac], color: [0., 0., 1.] });
    lines.push(VertC { pos: [0., -frac, rad - frac], color: [0., 0., 1.] });
    lines.push(VertC { pos: [0.,  frac, rad - frac], color: [0., 0., 1.] });

    MeshSource {
        verts: lines,
        inds: Indexing::All,
        prim: Primitive::LineList,
        mat: (),
    }
}

fn arrow() -> MeshSource<VertC, ()> {
    MeshSource {
        verts: vec![
            VertC { pos: [0., 0., 0.], color: [0., 0., 0.] },
            VertC { pos: [0., 0., 1.], color: [0., 0., 0.] },
            VertC { pos: [0.1, 0., 0.9], color: [0., 0., 0.] },
            VertC { pos: [0., 0., 1.], color: [0., 0., 0.] },
            VertC { pos: [-0.1, 0., 0.9], color: [0., 0., 0.] },
            VertC { pos: [0., 0., 1.], color: [0., 0., 0.] },
        ],
        inds: Indexing::All,
        prim: Primitive::LineList,
        mat: (),
    }
}

fn load_my_simple_object<P, R, F>(
    f: &mut F,
    path: P,
    albedo: [f32; 3],
    metalness: f32,
    roughness: f32,
    flatness: f32
)
    -> Result<Mesh<R, VertNTT, UberMaterial<R>>, Error>
    where P: AsRef<Path>, R: gfx::Resources, F: gfx::Factory<R>
{
    fn f2unorm(v: f32) -> u8 {
        (v * 256.).round().min(255.).max(0.) as u8
    }

    let albedo = [f2unorm(albedo[0]), f2unorm(albedo[1]), f2unorm(albedo[2]), 255];
    let knobs = [f2unorm(metalness), f2unorm(roughness), f2unorm(flatness), 0];
    use gfx::format::*;
    Ok(load::open_wavefront(path)?.compute_tan().with_material(UberMaterial {
        albedo: Texture::<_, (R8_G8_B8_A8, Srgb)>::uniform_value(f, albedo)?,
        normal: Texture::<_, (R8_G8_B8_A8, Unorm)>::uniform_value(f, [0x80, 0x80, 0xFF, 0xFF])?,
        knobs: Texture::<_, (R8_G8_B8_A8, Unorm)>::uniform_value(f, knobs)?,
    }).upload(f))
}

impl<R: gfx::Resources> App<R> {
    pub fn new<F: Factory<R> + FactoryExt<R>>(factory: &mut F) -> Result<Self, Error> {
        // Setup Painters
        let mut solid = Painter::new(factory)?;
        solid.setup(factory, Primitive::LineList)?;
        solid.setup(factory, Primitive::TriangleList)?;

        let mut uber: Painter<_, UberStyle<_>> = Painter::new(factory)?;
        uber.setup(factory, Primitive::TriangleList)?;

        // Construct App
        Ok(App {
            solid: solid,
            uber: uber,
            grid: grid_lines(8, 8.).upload(factory),
            controller_grid: grid_lines(2, 0.2).upload(factory),
            arrow: arrow().upload(factory),
            controller: load_my_simple_object(
                factory,
                "assets/controller.obj",
                [0.7, 0.7, 0.7],
                0.,
                0.4,
                0.)?,
            teapot: load::open_uber_mesh(
                factory, 
                "assets/teapot_wood/model.obj",
                "assets/teapot_wood/albedo.png",
                "assets/teapot_wood/normal.png",
                "assets/teapot_wood/knobs.png")?,
            start_time: Instant::now(),
            primary: MappedController {
                is: primary(),
                pad: Point2::new(0., 1.),
                .. Default::default()
            },
            secondary: MappedController {
                is: secondary(),
                .. Default::default()
            },
        })
    }

    pub fn draw<C: gfx::CommandBuffer<R>>(
        &mut self,
        ctx: &mut DrawParams<R, C>,
        vrm: &VrMoment,
    ) {
        let elapsed = self.start_time.elapsed();
        let t = elapsed.as_secs() as f32 + (elapsed.subsec_nanos() as f32 * 1e-9);

        match (self.primary.update(vrm), self.secondary.update(vrm)) {
            (Ok(_), Ok(_)) => (),
            _ => warn!("A not vive-like controller is connected"),
        }

        // Clear targets
        ctx.encoder.clear_depth(&ctx.depth, FAR_PLANE as f32);
        ctx.encoder.clear(&ctx.color, [BACKGROUND[0].powf(1. / 2.2), BACKGROUND[1].powf(1. / 2.2), BACKGROUND[2].powf(1. / 2.2), BACKGROUND[3]]);

        // Draw grid
        self.solid.draw(ctx, na::one(), &self.grid);

        // Draw teapot
        let tearot =
            UnitQuaternion::from_axis_angle(&Vector3::x_axis(), (t * 0.7).sin() * 10. * DEG)
            * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), (t * 0.8).cos() * 15. * DEG)
            * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), t * 60. * DEG);

        let teamat = if self.primary.connected {
            na::convert(self.primary.pose * Similarity3::from_parts(
                Translation3::new(0., 0., -0.25),
                tearot,
                (0.15 * self.primary.pad_theta().abs() as f32 / PI).max(0.001),
            ))
        } else {
            Similarity3::from_parts(
                Translation3::new(1., 0., 1.),
                tearot,
                1.,
            )
        };
        self.uber.draw(ctx, na::convert(teamat), &self.teapot);

        // Draw controllers
        for cont in vrm.controllers() {
            self.solid.draw(ctx, na::convert(cont.pose), &self.controller_grid);
            self.uber.draw(ctx, na::convert(cont.pose), &self.controller);
        }

        for cont in &[&self.primary, &self.secondary] {
            let scale = na::norm(&cont.lin_vel);
            if scale > ::std::f32::EPSILON {
                let mat = Similarity3::new_observer_frame(
                    &cont.origin(),
                    &(cont.origin() + cont.lin_vel),
                    &Vector3::y(),
                    scale,
                );
                self.solid.draw(ctx, na::convert(mat), &self.arrow);
            }
            let scale = na::norm(&cont.ang_vel) / 6.28;
            if scale > ::std::f32::EPSILON {
                let mat = Similarity3::new_observer_frame(
                    &cont.origin(),
                    &(cont.origin() + cont.ang_vel),
                    &Vector3::y(),
                    scale,
                );
                self.solid.draw(ctx, na::convert(mat), &self.arrow);
            }
        }
    }
}
