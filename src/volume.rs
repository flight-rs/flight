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
        let vpc = self.center - ray.origin;
        let rad2 = self.radius * self.radius;
        if vpc.dot(ray.direction) < 0. { // when the sphere is behind the origin p
            let mag2 = vpc.magnitude2();
            if mag2 > rad2 { // no intersection
                None
            } else { // occurs when p is inside the sphere
                Some(SphereHit {
                    distance: 0.,
                    location: ray.origin,
                })
            }
        } else {
            let magpc = ray.direction.dot(self.center - ray.origin); // length of ray to closest point to center
            let pc = ray.origin + magpc * ray.direction; // projection of center onto ray
            let closest2 = (self.center - pc).magnitude2();
	        if closest2 > rad2 { // there is no intersection
                None
            } else {
      		    if vpc.magnitude2() > rad2 { // origin is outside sphere
                    let dist = magpc - (rad2 - closest2).sqrt();
                    Some(SphereHit {
                        distance: dist,
                        location: ray.origin + ray.direction * dist,
                    })
                } else { // origin is inside sphere
			        Some(SphereHit {
                        distance: 0.,
                        location: ray.origin,
                    })
                }
            }
        }
    }
}