#![allow(dead_code)]

#[macro_use]
mod util;

use gfx;
use self::util::file;

pub const LIGHT_COUNT: usize = 2;

// Setup shaders
shader!(simple {
    vertex: file("shaders/transform.v.glsl")
        //.define("TEX")
        //.define("NORM")
        .define("COLOR"),
    fragment: file("shaders/simple.f.glsl")
        .define_to("I_POS", "v_pos")
        .define_to("I_COLOR", "v_color")
});