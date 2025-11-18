use super::vec3::Vec3;

#[derive(Copy, Clone, Debug)]
pub struct Ray {
    pub o: Vec3,
    pub d: Vec3,
    pub tmin: f64,
    pub tmax: f64,
}
impl Ray {
    pub fn new(o:Vec3,d:Vec3)->Self{ Self{o, d:d.normalized(), tmin:1e-4, tmax:1e9} }
    pub fn at(&self, t:f64)->Vec3{ self.o + self.d*t }
}
