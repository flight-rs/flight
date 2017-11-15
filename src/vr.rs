use nalgebra::{self as na, Transform3, Vector3, Point3, Vector2, Point2, Isometry3, Quaternion, Translation3, Unit};
use webvr::*;
use draw::EyeParams;
use fnv::FnvHashMap;
use gfx::{Rect};
use ::NativeRepr;

/// Provides access to VR hardware.
pub struct VrContext {
    vrsm: VRServiceManager,
    disp: VRDisplayPtr,
    /// Set the eyes' near clipping plane
    pub near: f64,
    /// Set the eyes' far clipping plane
    pub far: f64,
    layer: VRLayer,
    exit: bool,
    paused: bool,
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
            exit: false,
            paused: false,
        })
    }

    /// Connect to default hardware devices.
    pub fn new() -> Option<VrContext> {
        let mut vrsm = VRServiceManager::new();
        vrsm.register_defaults();
        VrContext::init(vrsm)
    }

    /// Connect to a mock HMD.
    pub fn mock() -> Option<VrContext> {
        let mut vrsm = VRServiceManager::new();
        vrsm.register_mock();
        VrContext::init(vrsm)
    }

    /// Set the OpenGL texture id to display on the HMD.
    pub fn set_texture(&mut self, texture_id: u32) {
        info!("Attaching texture {} to HMD", texture_id);
        self.layer.texture_id = texture_id;
    }

    /// Start drawing to the HMD.
    pub fn start(&mut self) {
        info!("Starting HMD presentation");
        self.disp.borrow_mut().start_present(Some(VRFramebufferAttributes {
            multiview: false,
            depth: false,
            multisampling: false,
        }));
    }

    /// Stop drawing to the HMD.
    pub fn stop(&mut self) {
        info!("Stopping HMD presentation");
        self.disp.borrow_mut().stop_present();
    }

    /// Retrieve the HMD device from the hardware API.
    pub fn retrieve_size(&mut self) -> (u32, u32) {
       size_from_data(&self.disp.borrow().data())
    }

    /// Synchronize with the hardware, returning transient details about the VR
    /// system at the specific moment in time. This data can be used directly or
    /// to update state variables.
    pub fn sync(&mut self) -> VrMoment {
        {
            let mut disp = self.disp.borrow_mut();
            disp.sync_poses();
        }

        let mut new_controllers = Vec::new();
        for event in self.vrsm.poll_events() {
            match event {
                VREvent::Display(VRDisplayEvent::Pause(_)) => self.paused = true,
                VREvent::Display(VRDisplayEvent::Resume(_)) => self.paused = false,
                VREvent::Display(VRDisplayEvent::Exit(_)) => self.exit = true,
                VREvent::Gamepad(VRGamepadEvent::Connect(_, state)) => 
                    new_controllers.push(ControllerRef::Indexed(state.gamepad_id)),
                _ => (),
            }
        }

        let mut moment = VrMoment {
            cont: FnvHashMap::default(),
            hmd: None,
            primary: None,
            secondary: None,
            tertiary: None,
            layer: self.layer.clone(),
            stage: na::one(),
            exit: self.exit,
            paused: self.paused,
            new_controllers: new_controllers,
        };
        {
            let disp = self.disp.borrow();
            let data = disp.data();
            let state = disp.synced_frame_data(self.near, self.far);
            let (w, h) = size_from_data(&data);

            moment.stage = if let Some(ref stage) = data.stage_parameters {
                Transform3::upgrade(stage.sitting_to_standing_transform)
                    .try_inverse().unwrap_or(Transform3::identity())
            } else {
                Transform3::identity()
            };

            let left_view = Transform3::upgrade(state.left_view_matrix);
            let right_view = Transform3::upgrade(state.right_view_matrix);
            let left_projection = Transform3::upgrade(state.left_projection_matrix);
            let right_projection = Transform3::upgrade(state.right_projection_matrix);

            if let (Some(pose), true) = (pose_transform(&state.pose), data.connected) {
                moment.hmd = Some(HmdMoment {
                    name: data.display_name.clone(),
                    size: (w, h),
                    pose: pose,
                    left: EyeParams {
                        eye: left_view.try_inverse().unwrap() * Point3::origin(),
                        view: left_view,
                        proj: left_projection,
                        clip_offset: -0.5,
                        clip: Rect {
                            x: 0,
                            y: 0,
                            w: data.left_eye_parameters.render_width as u16,
                            h: h as u16,
                        },
                    },
                    right: EyeParams {
                        eye: right_view.try_inverse().unwrap() * Point3::origin(),
                        view: right_view,
                        proj: right_projection,
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
            let mut gpiter = gamepads.iter().filter_map(|gp| {
                let gp = gp.borrow();
                if gp.state().connected { Some(gp.id()) } else { None }
            });
            moment.primary = gpiter.next();
            moment.secondary = gpiter.next();
            moment.tertiary = gpiter.next();
        }
        for gp in gamepads {
            let gp = gp.borrow();
            let data = gp.data();
            let state = gp.state();
            if let (Some(pose), true) = (pose_transform(&state.pose), state.connected) {
                moment.cont.insert(state.gamepad_id, ControllerMoment {
                    id: state.gamepad_id,
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

/// Instantaneous information about the VR system retrieved from `VrContext::sync()`.
/// This can be used directly or to update some persistent state.
pub struct VrMoment {
    cont: FnvHashMap<u32, ControllerMoment>,
    hmd: Option<HmdMoment>,
    primary: Option<u32>,
    secondary: Option<u32>,
    tertiary: Option<u32>,
    layer: VRLayer,
    /// The stage transform (moves the origin to the center of the room)
    pub stage: Transform3<f32>,
    /// Has the VR system requested the application to exit
    pub exit: bool,
    /// Has the VR system requested the application to pause movement (should still sync and submit frames)
    pub paused: bool,
    /// References to controllers that have connected since the last sync
    pub new_controllers: Vec<ControllerRef>,
}

impl VrMoment {
    /// Get a controller by reference if such a controller is connected.
    pub fn controller(&self, role: ControllerRef) -> Option<&ControllerMoment> {
        if let Some(ref i) = role.index(self) { self.cont.get(i) } else { None }
    }

    /// Iterate over all connected controllers.
    pub fn controllers<'a>(&'a self) -> ControllerIter<'a> {
        self.cont.values()
    }

    /// Get instantaneous information about the HMD if it is connected.
    pub fn hmd(&self) -> Option<&HmdMoment> {
        self.hmd.as_ref()
    }

    /// Submit the rendered scene. This ends the applicability
    /// of this information, since it only applies to the
    /// state of the VR system at the last sync.
    pub fn submit(self, ctx: &mut VrContext) {
        let mut d = ctx.disp.borrow_mut();
        d.render_layer(&self.layer);
        d.submit_frame();
    }
}

/// Iterator over momentary controller information.
pub type ControllerIter<'a> = ::std::collections::hash_map::Values<'a, u32, ControllerMoment>;

/// Used to persistently identity a controller, either by internal 
/// id or by role. Note that roles can refer to different physical devices 
/// at different times, while the internal id will remain locked 
/// to a particular device.
#[derive(Copy, Clone, Debug)]
pub enum ControllerRef {
    Primary,
    Secondary,
    Tertiary,
    Indexed(u32),
}

impl ControllerRef {
    /// Get the internal id of the controller at a particular moment.
    fn index(&self, moment: &VrMoment) -> Option<u32> {
        use self::ControllerRef::*;
        match *self {
            Primary => moment.primary,
            Secondary => moment.secondary,
            Tertiary => moment.tertiary,
            Indexed(i) => Some(i),
        }
    }

    /// Make thus reference specific to a device (internal id)
    /// rather than dynamically updating (role).
    pub fn fixed(&self, moment: &VrMoment) -> ControllerRef {
        match self.index(moment) {
            Some(i) => ControllerRef::Indexed(i),
            None => *self,
        }
    }
}

/// Create a reference to the primary controller.
pub fn primary() -> ControllerRef {
    ControllerRef::Primary
}

/// Create a reference to the secondary controller.
pub fn secondary() -> ControllerRef {
    ControllerRef::Secondary
}

/// Create a reference to the tertiary controller.
pub fn tertiary() -> ControllerRef {
    ControllerRef::Tertiary
}

/// Instantaneous information about a button.
pub type ButtonMoment = VRGamepadButton;

/// A device that provides instantaneous position and orientation information.
pub trait Trackable {
    /// Get the location and orientation of the device.
    fn pose(&self) -> Isometry3<f32>;

    /// Get the direction of the device's x axis.
    fn x_dir(&self) -> Vector3<f32> { self.pose() * Vector3::x() }
    /// Get the direction of the device's y axis.
    fn y_dir(&self) -> Vector3<f32> { self.pose() * Vector3::y() }
    /// Get the direction of the device's z axis.
    fn z_dir(&self) -> Vector3<f32> { self.pose() * Vector3::z() }
    /// The the location of the device's origin.
    fn origin(&self) -> Point3<f32> { self.pose() * Point3::origin() }
    /// Get the direction the device is pointing.
    fn pointing(&self) -> Vector3<f32> { -self.z_dir() }
}

/// Instantaneous information about the HMD. This can be used directly
/// or to update some persistent state.
#[derive(Clone)]
pub struct HmdMoment {
    /// The textual name of the HMD
    pub name: String,
    /// The resolution of the HMD
    pub size: (u32, u32),
    /// The location and orientation of the HMD
    pub pose: Isometry3<f32>,
    /// The drawing parameters for the left eye
    pub left: EyeParams,
    /// The drawing parameters for the right eye
    pub right: EyeParams,
}

impl Trackable for HmdMoment {
    fn pose(&self) -> Isometry3<f32> {
        self.pose
    }
}

/// Instantaneous information about a controller. This can be used directly
/// or to update some persistent state.
#[derive(Clone, Debug)]
pub struct ControllerMoment {
    id: u32,
    /// The textual name of the controller
    pub name: String,
    /// The location and orientation of the controller
    pub pose: Isometry3<f32>,
    /// The state of the floating point inputs on the controller
    pub axes: Vec<f64>,
    /// The state of the button inputs on the controller
    pub buttons: Vec<ButtonMoment>,
}

impl ControllerMoment {
    /// Create a reference to this particular hardware device (not to its role).
    pub fn reference(&self) -> ControllerRef {
        ControllerRef::Indexed(self.id)
    }
}

impl Trackable for ControllerMoment {
    fn pose(&self) -> Isometry3<f32> {
        self.pose
    }
}

fn pose_transform(ctr: &VRPose) -> Option<Isometry3<f32>> {
    let or = Unit::new_normalize(Quaternion::upgrade(
        match ctr.orientation { Some(o) => o, None => return None }));
    let pos = Translation3::upgrade(
        match ctr.position { Some(o) => o, None => return None });
    Some(Isometry3::from_parts(pos, or))
}

/// A structure for tracking the state of a vive controller.
#[derive(Clone, Debug)]
pub struct ViveController {
    /// The controller that updates this state object
    pub is: ControllerRef,
    /// The controller connection status.
    pub connected: bool,
    /// The pose of the controller
    pub pose: Isometry3<f32>,
    /// The transformation of the controller between the second most and most recent updates
    pub pose_delta: Isometry3<f32>,
    /// How far is the trigger pulled
    pub trigger: f64,
    /// The change in the trigger between the second most and most recent updates
    pub trigger_delta: f64,
    /// The last touched location on the circular pad
    pub pad: Point2<f64>,
    /// The change in touch location on the circular pad between the second most and most recent updates
    pub pad_delta: Vector2<f64>,
    /// Is the circular pad touched
    pub pad_touched: bool,
    /// Is the menu button pressed
    pub menu: bool,
    /// Are the grip buttons pressed
    pub grip: bool,
}

impl Default for ViveController {
    fn default() -> Self {
        ViveController {
            is: primary(),
            connected: false,
            pose: na::one(),
            pose_delta: na::one(),
            trigger: 0.,
            trigger_delta: 0.,
            pad: Point2::origin(),
            pad_delta: na::zero(),
            pad_touched: false,
            menu: false,
            grip: false,
        }
    }
}

impl ViveController {
    /// Create a simple default state that will be updated with data from the given controller.
    pub fn new(reference: ControllerRef) -> ViveController {
        ViveController {
            is: reference,
            .. Default::default()
        }
    }

    /// Update the controller state using the provided instantaneous information.
    pub fn update(&mut self, mom: &VrMoment) -> Result<(), ()> {
        if let Some(cont) = mom.controller(self.is) {
            if cont.axes.len() < 3 || cont.buttons.len() < 2 { return Err(()) }

            self.connected = true;

            self.pose_delta = cont.pose * self.pose.inverse();
            self.pose = cont.pose;

            let (x, y) = (cont.axes[0], cont.axes[1]);
            if x != 0. || y != 0. {
                let pad = Point2::new(x, y);
                if self.pad_touched {
                    self.pad_delta = pad - self.pad;
                } else {
                    self.pad_delta = na::zero();
                }
                self.pad = pad;
                self.pad_touched = true;
            } else { 
                self.pad_touched = false;
                self.pad_delta = na::zero();
            }

            self.trigger_delta = cont.axes[2] - self.trigger;
            self.trigger = cont.axes[2];
            self.menu = cont.buttons[0].pressed;
            self.grip = cont.buttons[1].pressed;
        } else {
            self.pad_touched = false;
            self.menu = false;
            self.grip = false;
            self.trigger = 0.;
            self.connected = false;
        }
        Ok(())
    }

    /// Get the radial location of the last circular pad touch.
    pub fn pad_theta(&self) -> f64 {
        self.pad[1].atan2(self.pad[0])
    }
}

impl Trackable for ViveController {
    fn pose(&self) -> Isometry3<f32> {
        self.pose
    }
}
