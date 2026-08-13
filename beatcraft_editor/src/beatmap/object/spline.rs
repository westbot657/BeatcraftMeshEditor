use glam::Vec3;


pub struct BezierPath {
    points: Vec<Vec3>,
}

impl BezierPath {
    /// Returns None if no points were given.
    pub fn new(points: Vec<Vec3>) -> Option<Self> {
        if points.is_empty() {
            None
        } else {
            Some(Self { points })
        }
    }

    fn binomial_coefficient(n: i32, k: i32) -> i32 {
        if k < 0 || k > n {
            return 0;
        }
        if k == 0 || k == n {
            return 1;
        }
        Self::binomial_coefficient(n - 1, k - 1) + Self::binomial_coefficient(n - 1, k)
    }

    pub fn position(&self, t: f32) -> Vec3 {
        let n = self.points.len() as i32 - 1;

        let mut pt = Vec3::ZERO;

        for i in 0i32..=n {
            let coeff = Self::binomial_coefficient(n, i) as f32 * f32::powi(1. - t, n - i) * f32::powi(t, i);
            pt += coeff * unsafe { self.points.get_unchecked(i as usize) };
        }
        pt
    }

    pub fn derivative(&self, t: f32) -> Vec3 {
        let n = self.points.len() as i32 - 1;
        if n == 0 { return Vec3::ZERO }

        let mut pt = Vec3::ZERO;

        for i in 0i32..n {
            let coeff = (n * Self::binomial_coefficient(n - 1, i)) as f32
                * f32::powi(1. - t, (n - 1) - i)
                * f32::powi(t, i);
            pt += coeff * unsafe { self.points.get_unchecked((i + 1) as usize) - self.points.get_unchecked(i as usize) };
        }
        pt
    }

    pub fn points(&self) -> &[Vec3] {
        self.points.as_slice()
    }

}

pub struct BezierCurve {
    p0: Vec3,
    p1: Vec3,
    p2: Vec3,
}
impl BezierCurve {
    pub fn new(p0: Vec3, p1: Vec3, p2: Vec3) -> Self {
        Self { p0, p1, p2 }
    }
    pub fn position(&self, t: f32) -> Vec3 {
        let n = 1. - t;
        let x = n * n * self.p0.x + 2. * n * t * self.p1.x + t * t * self.p2.x;
        let y = n * n * self.p0.y + 2. * n * t * self.p1.y + t * t * self.p2.y;
        let z = n * n * self.p0.z + 2. * n * t * self.p1.z + t * t * self.p2.z;
        Vec3::new(x, y, z)
    }
    pub fn derivative(&self, t: f32) -> Vec3 {
        let n = 1. - t;
        let x = 2. * n * (self.p1.x - self.p0.x) + 2. * t * (self.p2.x - self.p1.x);
        let y = 2. * n * (self.p1.y - self.p0.y) + 2. * t * (self.p2.y - self.p1.y);
        let z = 2. * n * (self.p1.z - self.p0.z) + 2. * t * (self.p2.z - self.p1.z);
        Vec3::new(x, y, z)
    }
}

pub trait Spline {
    fn position(&self, t: f32) -> Vec3;
    fn derivative(&self, t: f32) -> Vec3;
}

impl Spline for BezierPath {
    fn position(&self, t: f32) -> Vec3 {
        Self::position(self, t)
    }

    fn derivative(&self, t: f32) -> Vec3 {
        Self::derivative(self, t)
    }
}

impl Spline for BezierCurve {
    fn position(&self, t: f32) -> Vec3 {
        Self::position(self, t)
    }

    fn derivative(&self, t: f32) -> Vec3 {
        Self::derivative(self, t)
    }
}

