use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use ::Error;

macro_rules! shader {
    ($name:ident { $($x:tt)+ }) => (pub fn $name<R: gfx::Resources, F: gfx::Factory<R>>(factory: &mut F) 
        -> Result<gfx::ShaderSet<R>, Error> {
        Ok(shader_set!(factory, $($x)+))
    })
}

macro_rules! single_shader {
    ($f:ident, $c:ident, $s:expr) => ({
        let name = $s.name.clone();
        $f.$c(&$s.build().into_bytes())
            .map_err(move |e| (e, name))?
    })
}

macro_rules! shader_set {
    ($f:ident, vertex: $v:expr, fragment: $p:expr $(,)*) => ({
        let v = $v;
        let p = $p;
        gfx::ShaderSet::Simple(
            single_shader!($f, create_shader_vertex, v),
            single_shader!($f, create_shader_pixel, p),
        )
    });
    ($f:ident, vertex: $v:expr, geometry: $g:expr, fragment: $p:expr $(,)*) => ({
        let v = $v;
        let g = $g;
        let p = $p;
        gfx::ShaderSet::Geometry(
            single_shader!($f, create_shader_vertex, v),
            single_shader!($f, create_shader_geometry, g),
            single_shader!($f, create_shader_pixel, p),
        )
    });
    ($f:ident, vertex: $v:expr, tessellation_control: $h:expr, tessellation_evaluation: $d:expr, fragment: $p:expr $(,)*) => ({
        let v = $v;
        let h = $h;
        let d = $d;
        let p = $p;
        gfx::ShaderSet::Tessellated(
            single_shader!($f, create_shader_vertex, v),
            single_shader!($f, create_shader_hull, h),
            single_shader!($f, create_shader_domain, d),
            single_shader!($f, create_shader_pixel, p),
        )
    });
}

pub struct BuildShader {
    prefix: String,
    source: String,
    pub name: String,
}

pub fn file<P: AsRef<Path>>(path: P) -> Result<BuildShader, Error> {
    let path = path.as_ref();
    let mut build = BuildShader {
        prefix: String::new(),
        source: String::new(),
        name: format!("{}", path.display()),
    };
    File::open(path)
        .map_err(|e| (e, path.display()))?
        .read_to_string(&mut build.source)
        .map_err(|e| (e, path.display()))?;
    Ok(build)
}

impl BuildShader {
    pub fn define(mut self, name: &str) -> BuildShader {
        self.prefix += &format!("#define {}\n", name);
        self
    }

    pub fn define_to<S>(mut self, name: &str, val: S) -> BuildShader
        where S: ToString
    {
        self.prefix += &format!("#define {} {}\n", name, val.to_string());
        self
    }

    pub fn build(self) -> String {
        if self.source.starts_with("#version") {
            let (ver, src) = self.source.split_at(self.source.find('\n').unwrap_or(self.source.len()));
            format!("{}\n{}#line 1\n{}", ver, self.prefix, src)
        } else {
            format!("{}#line 1\n{}", self.prefix, self.source)
        }
    }
}