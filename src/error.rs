use std::fmt;
use gfx::{Primitive, UpdateError, PipelineStateError};
use gfx::CombinedError as GfxCombinedError;
use gfx::shade::core::CreateShaderError;
use gfx::shade::ProgramError;
use image::ImageError;
use std::io::Error as IoError;

macro_rules! error_impl {
    ($e:ident($n:ident), $v:ident($t:path)) => {
        impl From<$t> for $e {
            fn from(e: $t) -> $e {
                Error {
                    context: None,
                    error: $n::$v(e),
                }
            }
        }
    };
    ($e:ident($n:ident), $v:ident(-> $f:ident($($p:ident: $t:path),*))) => {
        impl $e {
            pub fn $f($($p: $t),*) -> $e {
                Error {
                    context: None,
                    error: $n::$v($($p),*),
                }
            }
        }
    };
}

macro_rules! error_var {
    ($t:path) => {$t};
    (-> $f:ident($($p:ident: $t:path),*)) => {$($t),*};
}

macro_rules! error {
    ($(#[$m:meta])*$e:ident($n:ident) {
        $($v:ident($($w:tt)*)),*
    }) => {
        /// Error type enum
        #[derive(Debug)]
        pub enum $n {
            $($v(error_var!($($w)*))),*
        }

        $(#[$m])*
        #[derive(Debug)]
        pub struct $e {
            pub context: Option<String>,
            pub error: $n,
        }

        impl From<$n> for $e {
            fn from(e: $n) -> $e {
                Error {
                    context: None,
                    error: e,
                }
            }
        }

        $(error_impl!($e($n), $v($($w)*));)*
    }
}

error!(
    /// Combines all errors that can occur in the flight API
    Error(ErrorKind) {
        Gfx(GfxCombinedError),
        GfxUpdate(UpdateError<usize>),
        Pipeline(PipelineStateError<String>),
        Shader(CreateShaderError),
        ShaderProgram(ProgramError),
        Image(ImageError),
        Io(IoError),
        InvalidPrimitive(-> invalid_primitive(p: Primitive)),
        CubemapSizeMismatch(-> cube_size(expected: u32))
    }
);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref ctx) = self.context {
            write!(f, "{:?}: {}", self.error, ctx)
        } else {
            write!(f, "{:?}", self.error)
        }
    }
}

impl<T, C> From<(T, C)> for Error
    where C: ::std::fmt::Display, Error: From<T> {
    fn from((e, c): (T, C)) -> Error {
        Error::from(e).context(format!("{}", c))
    }
}

impl Error {
    /// Add some text explaining the context in which the error occurred
    pub fn context(self, ctx: String) -> Error {
        Error {
            context: Some(ctx),
            error: self.error,
        }
    }
}
