use cgmath::prelude::*;
use cgmath::*;
use webvr::*;
use volume::Ray;
use context::EyeContext;
use fnv::FnvHashMap;
use gfx::{Rect};

pub struct VrContext {
    vrsm: VRServiceManager,
    disp: VRDisplayPtr,
    pub near: f64,
    pub far: f64,
    layer: VRLayer,
}

fn size_from_data(data: &VRDisplayData) -> (u32, u32) {
    let w = data.left_eye_parameters.render_width + data.right_eye_parameters.render_width;
    let h = data.left_eye_parameters.render_height.max(data.right_eye_parameters.render_height);
    (w, h)
}

impl VrContext {
    pub fn init(mut vrsm: VRServiceManager) -> Option<VrContext> {
        let display = match vrsm.get_displays().get(0) {
            Some(d) => d.clone(),
            None => {
                error!("No VR display present");
                return None
            },
        };
        info!("VR Device: {}", display.borrow().data().display_name);
        Some(VrContext {
            vrsm: vrsm,
            disp: display,
            near: 0.1,
            far: 100.0,
            layer: Default::default(),
        })
    }

    pub fn new() -> Option<VrContext> {
        let mut vrsm = VRServiceManager::new();
        vrsm.register_defaults();
        VrContext::init(vrsm)
    }

    pub fn mock() -> Option<VrContext> {
        let mut vrsm = VRServiceManager::new();
        vrsm.register_mock();
        VrContext::init(vrsm)
    }

    pub fn set_texture(&mut self, texture_id: u32) {
        info!("Attaching texture to HMD: {}", texture_id);
        self.layer.texture_id = texture_id;
    }

    pub fn start(&mut self) {
        info!("Starting HMD presentation");
        self.disp.borrow_mut().start_present(Some(VRFramebufferAttributes {
            multiview: false,
            depth: false,
            multisampling: false,
        }));
    }

    pub fn stop(&mut self) {
        info!("Stopping HMD presentation");
        self.disp.borrow_mut().stop_present();
    }

    pub fn retrieve_size(&mut self) -> (u32, u32) {
       size_from_data(&self.disp.borrow().data())
    }

    pub fn sync(&mut self) -> VrMoment {
        let mut moment = VrMoment {
            cont: FnvHashMap::default(),
            hmd: None,
            primary: None,
            secondary: None,
            tertiary: None,
            layer: self.layer.clone(),
            stage: Matrix4::zero(),
        };
        {
            let mut disp = self.disp.borrow_mut();
            disp.sync_poses();
            let data = disp.data();
            let state = disp.synced_frame_data(self.near, self.far);
            let (w, h) = size_from_data(&data);

            moment.stage = if let Some(ref stage) = data.stage_parameters {
                <&Matrix4<f32>>::from(&stage.sitting_to_standing_transform).inverse_transform().unwrap()
            } else {
                Matrix4::identity()
            };

            if let (Some(pose), true) = (pose_transform(&state.pose), data.connected) {
                moment.hmd = Some(Hmd {
                    name: data.display_name.clone(),
                    size: (w, h),
                    pose: pose,
                    left: EyeContext {
                        view: <&Matrix4<_>>::from(&state.left_view_matrix).clone(),
                        proj: <&Matrix4<_>>::from(&state.left_projection_matrix).clone(),
                        clip_offset: -0.5,
                        clip: Rect { 
                            x: 0,
                            y: 0,
                            w: data.left_eye_parameters.render_width as u16,
                            h: h as u16, 
                        },
                    },
                    right: EyeContext {
                        view: <&Matrix4<_>>::from(&state.right_view_matrix).clone(),
                        proj: <&Matrix4<_>>::from(&state.right_projection_matrix).clone(),
                        clip_offset: 0.5,
                        clip: Rect { 
                            x: data.left_eye_parameters.render_width as u16,
                            y: 0,
                            w: data.right_eye_parameters.render_width as u16,
                            h: h as u16,
                        },
                    },
                });
            }
        }
        let gamepads =  self.vrsm.get_gamepads();
        {
            let mut gpiter = gamepads.iter();
            if let Some(gp) = gpiter.next() {
                moment.primary = Some(gp.borrow().data().display_id);
            }
            if let Some(gp) = gpiter.next() {
                moment.secondary = Some(gp.borrow().data().display_id);
            }
            if let Some(gp) = gpiter.next() {
                moment.tertiary = Some(gp.borrow().data().display_id);
            }
        }
        for gp in gamepads {
            let gp = gp.borrow();
            let data = gp.data();
            let state = gp.state();
            if let Some(pose) = pose_transform(&state.pose) {
                moment.cont.insert(data.display_id, Controller {
                    name: data.name.clone(),
                    pose: pose,
                    axes: state.axes.clone(),
                    buttons: state.buttons.clone(),
                });
            }
        }
        moment
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ControllerRef {
    Primary,
    Secondary,
    Tertiary,
    Indexed(u32),
}

impl ControllerRef {
    pub fn primary() -> ControllerRef {
        ControllerRef::Primary
    }

    pub fn secondary() -> ControllerRef {
        ControllerRef::Secondary
    }

    pub fn tertiary() -> ControllerRef {
        ControllerRef::Tertiary
    }

    fn index(&self, moment: &VrMoment) -> Option<u32> {
        use self::ControllerRef::*;
        match *self {
            Primary => moment.primary,
            Secondary => moment.secondary,
            Tertiary => moment.tertiary,
            Indexed(i) => Some(i),
        }
    }

    /// Make this always reference the current controller 
    /// (will not change when controller role changes).
    pub fn fixed(&self, moment: &VrMoment) -> ControllerRef {
        match self.index(moment) {
            Some(i) => ControllerRef::Indexed(i),
            None => *self,
        }
    }
}

pub type ControllerButton = VRGamepadButton;

pub trait Trackable {
    fn pose(&self) -> Matrix4<f32>;

    fn ray(&self) -> Ray {
        Ray {
            origin: self.origin().cast(),
            direction: self.z_dir().cast(),
        }
    }

    fn x_dir(&self) -> Vector3<f32> { self.pose().x.truncate() }
    fn y_dir(&self) -> Vector3<f32> { self.pose().y.truncate() }
    fn z_dir(&self) -> Vector3<f32> { self.pose().z.truncate() }
    fn origin(&self) -> Point3<f32> { Point3::from_vec(self.pose().w.truncate()) }
}

#[derive(Clone)]
pub struct Hmd {
    pub name: String,
    pub size: (u32, u32),
    pub pose: Matrix4<f32>,
    pub left: EyeContext,
    pub right: EyeContext,
}

impl Trackable for Hmd {
    fn pose(&self) -> Matrix4<f32> {
        self.pose
    }
}

#[derive(Clone, Debug)]
pub struct Controller {
    pub name: String,
    pub pose: Matrix4<f32>,
    pub axes: Vec<f64>,
    pub buttons: Vec<ControllerButton>,
}

impl Trackable for Controller {
    fn pose(&self) -> Matrix4<f32> {
        self.pose
    }
}

pub type ControllerIter<'a> = ::std::collections::hash_map::Values<'a, u32, Controller>;

pub struct VrMoment {
    cont: FnvHashMap<u32, Controller>,
    hmd: Option<Hmd>,
    primary: Option<u32>,
    secondary: Option<u32>,
    tertiary: Option<u32>,
    layer: VRLayer,
    pub stage: Matrix4<f32>,
}

impl VrMoment {
    pub fn controller(&self, role: ControllerRef) -> Option<&Controller> {
        if let Some(ref i) = role.index(self) { self.cont.get(i) } else { None }
    }

    pub fn controllers<'a>(&'a self) -> ControllerIter<'a> {
        self.cont.values()
    }

    pub fn hmd(&self) -> Option<&Hmd> {
        self.hmd.as_ref()
    }

    pub fn submit(self, ctx: &mut VrContext) {
        let mut d = ctx.disp.borrow_mut();
        d.render_layer(&self.layer);
        d.submit_frame();
    }
}

fn pose_transform(ctr: &VRPose) -> Option<Matrix4<f32>> {
    let or = match ctr.orientation { Some(o) => o, None => return None };
    let rot = Quaternion::new(or[3], or[0], or[1], or[2]);
    let pos = Vector3::from(match ctr.position { Some(o) => o, None => return None });
    Some(Matrix4::from(Decomposed {
        scale: 1.,
        rot: rot,
        disp: pos,
    }))
}