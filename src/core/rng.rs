pub struct Rng { state: u64 }
impl Rng {
    pub fn new(seed:u64)->Self{ Self{state: seed.max(1)} }
    pub fn next_u32(&mut self)->u32{
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.state >> 32) as u32
    }
    pub fn next_f32(&mut self)->f32{ (self.next_u32() as f32) / (u32::MAX as f32) }
    pub fn next_f64(&mut self)->f64{ (self.next_u32() as f64) / (u32::MAX as f64) }
}
