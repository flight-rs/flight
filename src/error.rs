use gfx::Primitive;

#[derive(Fail, Debug, Clone)]
pub enum FlightError {
    #[fail(display = "The {:?} primitive is not supported for this operation", given)]
    InvalidPrimitive {
        given: Primitive,
    },
    #[fail(display = "The given cubemap is not {0:} by {0:}", expected)]
    CubemapSizeMismatch {
        expected: u32,
    },
}
