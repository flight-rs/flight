use cgmath::*;

pub trait Volume {
    type Hit: Intersection;

    fn intersects(&self, ray: Ray) -> Option<Self::Hit>;
}

pub trait Intersection {
    fn distance(&self) -> f64;
    fn location(&self) -> Point3<f64>;
}


#[derive(Debug)]
pub struct Ray {
    pub origin: Point3<f64>,
    pub direction: Vector3<f64>,
}

#[derive(Debug)]
pub struct Sphere {
    pub radius: f64,
    pub center: Point3<f64>,
}

#[derive(Debug)]
pub struct SphereHit {
    distance: f64,
    location: Point3<f64>,
}

impl Intersection for SphereHit {
    fn distance(&self) -> f64 { self.distance }
    fn location(&self) -> Point3<f64> { self.location }
}

impl Volume for Sphere {
    type Hit = SphereHit;
    fn intersects(&self, ray: Ray) -> Option<SphereHit> {
        let r = ray.direction.cross(self.center-ray.origin).magnitude();
        if r < self.radius {
            let d = (r.powi(2) + (self.center-ray.origin).magnitude().powi(2) ).sqrt();
            let s = (r.powi(2) + self.radius.powi(2)).sqrt();
            let intersectDistance = d - s;
            let intersectPoint = ray.origin + (ray.direction * intersectDistance);

            unimplemented!();
        } else { None }
    }
}