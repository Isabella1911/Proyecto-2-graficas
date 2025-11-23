use crate::core::math::Vec3;

pub struct DayNight {}

impl DayNight {
    pub fn new() -> Self { Self {} }


    pub fn sun_direction(&self, t: f64) -> Vec3 {
        let cycle = 140.0; 
        let phase = (t / cycle) * std::f64::consts::TAU;

        
        let y = phase.sin();       
        let x = phase.cos();       
        
        
        Vec3::new(x, y.max(0.02), 0.15).normalized()
    }

    
    pub fn sun_intensity(&self, t: f64) -> f64 {
        let elev = self.sun_direction(t).y.max(0.0);
        
        1.5 * elev.powf(0.8)
    }

   
    pub fn sun_color(&self, t: f64) -> Vec3 {
        let elev = self.sun_direction(t).y;

        if elev <= 0.0 {
            return Vec3::new(0.0, 0.0, 0.0);
        }

        let dawn = Vec3::new(1.00, 0.72, 0.40); 
        let noon = Vec3::new(1.00, 0.98, 0.97); 

        let k = elev.clamp(0.0, 1.0);
        dawn * (1.0 - k) + noon * k
    }


    pub fn sky_color(&self, t: f64) -> Vec3 {
        let sun_dir = self.sun_direction(t);
        let elev = sun_dir.y;

       
        let night = Vec3::new(0.06, 0.08, 0.12);
        let twilight = Vec3::new(0.68, 0.50, 0.72); 

        let zenith_day = Vec3::new(0.55, 0.75, 1.00);   
        let horizon_day = Vec3::new(0.90, 0.95, 1.00);  

        if elev <= -0.03 {
            
            return night * 0.7 + twilight * 0.3;
        }

        let k = elev.clamp(0.0, 1.0);
        let base = zenith_day * 0.55 + horizon_day * 0.45;

        
        let warm = Vec3::new(1.00, 0.70, 0.55);
        let horizon_factor = (0.5 - elev).clamp(0.0, 0.5) / 0.5;

        base * (1.0 - 0.15 * horizon_factor) + warm * (0.10 * horizon_factor)
    }

    
    pub fn ambient_level(&self, t: f64) -> f64 {
        let elev = self.sun_direction(t).y;

        if elev < -0.2 {
            return 0.05
        }
        if elev < 0.0 {
            return 0.05 + ((elev + 0.2) / 0.2) * 0.06;
        }

        0.25 + elev * 0.25
    }
}
