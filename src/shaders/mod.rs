#![allow(dead_code)]

#[macro_use]
mod util;

use gfx;
use self::util::file;

use gfx::ShaderSet;
use gfx::shade::core::CreateShaderError;
pub type ShaderResult<R> = Result<ShaderSet<R>, CreateShaderError>;

// Setup shaders
shader!(simple {
    vertex: file("shaders/transform.v.glsl")
        .define("COLOR"),
    fragment: file("shaders/simple.f.glsl")
        .define_to("I_POS", "v_pos")
        .define_to("I_COLOR", "v_color")
});

shader!(unishade {
    vertex: file("shaders/transform.v.glsl")
        .define("NORM"),
    fragment: file("shaders/unishade.f.glsl")
        .define_to("I_POS", "v_pos")
        .define_to("I_NORM", "v_norm")
});