use std::fs::File;
use std::path::Path;
use std::io::prelude::*;

macro_rules! shader {
    ($name:ident { $($x:tt)+ }) => (pub fn $name<R: gfx::Resources, F: gfx::Factory<R>>(factory: &mut F) 
        -> Result<gfx::ShaderSet<R>, gfx::shade::core::CreateShaderError> {
        Ok(shader_set!(factory, $($x)+))
    })
}

macro_rules! shader_set {
    ($f:ident, vertex: $v:expr, fragment: $p:expr $(,)*) => ({
        let v = $v.build().into_bytes();
        let p = $p.build().into_bytes();
        gfx::ShaderSet::Simple(
            $f.create_shader_vertex(&v)?,
            $f.create_shader_pixel(&p)?,
        )
    });
    ($f:ident, vertex: $v:expr, geometry: $g:expr, fragment: $p:expr $(,)*) => ({
        let v = $v.build().into_bytes();
        let g = $g.build().into_bytes();
        let p = $p.build().into_bytes();
        gfx::ShaderSet::Geometry(
            $f.create_shader_vertex(&v)?,
            $f.create_shader_geometry(&v)?,
            $f.create_shader_pixel(&p)?,
        )
    });
    ($f:ident, vertex: $v:expr, tessellation_control: $h:expr, tessellation_evaluation: $d:expr, fragment: $p:expr $(,)*) => ({
        let v = $v.build().into_bytes();
        let h = $h.build().into_bytes();
        let d = $d.build().into_bytes();
        let p = $p.build().into_bytes();
        gfx::ShaderSet::Tessellated(
            $f.create_shader_vertex(&v)?,
            $f.create_shader_hull(&h)?,
            $f.create_shader_domain(&d)?,
            $f.create_shader_pixel(&p)?,
        )
    });
}

pub struct BuildShader {
    prefix: String,
    source: String,
    name: Option<String>,
}

pub fn file(fname: &str) -> BuildShader {
    let path = Path::new(fname);
    let mut build = BuildShader {
        prefix: String::new(),
        source: String::new(),
        name: match path.file_name() {
            Some(v) => v.to_str().map(|v| v.to_owned()),
            None => None,
        },
    };
    File::open(path)
        .expect(&("Shader \"".to_owned() + fname + "\" not found"))
        .read_to_string(&mut build.source)
        .unwrap();
    build
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

    pub fn vals<'a, M>(mut self, vals: M) -> BuildShader
        where M: IntoIterator<Item = &'a (&'static str, Option<String>)>
    {
        for &(ref n, ref v) in vals {
            self = match *v {
                Some(ref v) => self.define_to(n, v),
                None => self.define(n),
            };
        }
        self
    }

    pub fn build(self) -> String {
        if self.source.starts_with("#version") {
            let (ver, src) = self.source.split_at(self.source.find('\n').unwrap_or(self.source.len()));
            format!("{}\n{}#line 2\n{}", ver, self.prefix, src)
        } else {
            format!("{}#line 1\n{}", self.prefix, self.source)
        }
    }
}