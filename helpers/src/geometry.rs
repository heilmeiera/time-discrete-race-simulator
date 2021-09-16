use approx::ulps_eq;
use serde::Deserialize;

// 2D ----------------------------------------------------------------------------------------------
#[derive(Debug, Deserialize, Clone)]
pub struct Point2d {
    pub x: f64,
    pub y: f64,
}

impl Point2d {
    pub fn as_vector2d(&self) -> Vector2d {
        Vector2d {
            dx: self.x,
            dy: self.y,
        }
    }
    pub fn as_point3d(&self) -> Point3d {
        Point3d {
            x: self.x,
            y: self.y,
            z: 0.0,
        }
    }
    pub fn shift(&self, other: &Vector2d) -> Point2d {
        self.as_vector2d().add(other).as_point2d()
    }
}

impl PartialEq for Point2d {
    fn eq(&self, other: &Self) -> bool {
        ulps_eq!(self.x, other.x) && ulps_eq!(self.y, other.y)
    }
}

#[derive(Debug, Clone)]
pub struct Vector2d {
    pub dx: f64,
    pub dy: f64,
}

impl Vector2d {
    pub fn as_point2d(&self) -> Point2d {
        Point2d {
            x: self.dx,
            y: self.dy,
        }
    }
    pub fn as_vector3d(&self) -> Vector3d {
        Vector3d {
            dx: self.dx,
            dy: self.dy,
            dz: 0.0,
        }
    }
    pub fn sub(&self, other: &Self) -> Vector2d {
        Vector2d {
            dx: self.dx - other.dx,
            dy: self.dy - other.dy,
        }
    }
    pub fn add(&self, other: &Self) -> Vector2d {
        Vector2d {
            dx: self.dx + other.dx,
            dy: self.dy + other.dy,
        }
    }
    pub fn mult(&self, k: f64) -> Vector2d {
        Vector2d {
            dx: self.dx * k,
            dy: self.dy * k,
        }
    }
    /// convenience function (strictly speaking, the cross product is not defined in a 2D space)
    pub fn cross(&self, other: &Self) -> f64 {
        self.as_vector3d().cross(&other.as_vector3d()).dz
    }
    pub fn abs(&self) -> f64 {
        (self.dx.powf(2.0) + self.dy.powf(2.0)).sqrt()
    }
    pub fn normal_vector(&self) -> Vector2d {
        Vector2d {
            dx: -self.dy,
            dy: self.dx,
        }
    }
    pub fn normalized(&self) -> Vector2d {
        self.mult(1.0 / self.abs())
    }
}

impl PartialEq for Vector2d {
    fn eq(&self, other: &Self) -> bool {
        ulps_eq!(self.dx, other.dx) && ulps_eq!(self.dy, other.dy)
    }
}

// 3D ----------------------------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct Point3d {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point3d {
    pub fn as_point2d(&self) -> Point2d {
        Point2d {
            x: self.x,
            y: self.y,
        }
    }
    pub fn as_vector3d(&self) -> Vector3d {
        Vector3d {
            dx: self.x,
            dy: self.y,
            dz: self.z,
        }
    }
    pub fn shift(&self, other: &Vector3d) -> Point3d {
        self.as_vector3d().add(other).as_point3d()
    }
}

impl PartialEq for Point3d {
    fn eq(&self, other: &Self) -> bool {
        ulps_eq!(self.x, other.x) && ulps_eq!(self.y, other.y) && ulps_eq!(self.z, other.z)
    }
}

#[derive(Debug, Clone)]
pub struct Vector3d {
    pub dx: f64,
    pub dy: f64,
    pub dz: f64,
}

impl Vector3d {
    pub fn as_vector2d(&self) -> Vector2d {
        Vector2d {
            dx: self.dx,
            dy: self.dy,
        }
    }
    pub fn as_point3d(&self) -> Point3d {
        Point3d {
            x: self.dx,
            y: self.dy,
            z: self.dz,
        }
    }
    pub fn sub(&self, other: &Self) -> Vector3d {
        Vector3d {
            dx: self.dx - other.dx,
            dy: self.dy - other.dy,
            dz: self.dz - other.dz,
        }
    }
    pub fn add(&self, other: &Self) -> Vector3d {
        Vector3d {
            dx: self.dx + other.dx,
            dy: self.dy + other.dy,
            dz: self.dz + other.dz,
        }
    }
    pub fn mult(&self, k: f64) -> Vector3d {
        Vector3d {
            dx: self.dx * k,
            dy: self.dy * k,
            dz: self.dz * k,
        }
    }
    pub fn cross(&self, other: &Self) -> Vector3d {
        Vector3d {
            dx: self.dy * other.dz - self.dz * other.dy,
            dy: self.dz * other.dx - self.dx * other.dz,
            dz: self.dx * other.dy - self.dy * other.dx,
        }
    }
    pub fn abs(&self) -> f64 {
        (self.dx.powf(2.0) + self.dy.powf(2.0) + self.dz.powf(2.0)).sqrt()
    }
    pub fn normalized(&self) -> Vector3d {
        self.mult(1.0 / self.abs())
    }
}

impl PartialEq for Vector3d {
    fn eq(&self, other: &Self) -> bool {
        ulps_eq!(self.dx, other.dx) && ulps_eq!(self.dy, other.dy) && ulps_eq!(self.dz, other.dz)
    }
}
