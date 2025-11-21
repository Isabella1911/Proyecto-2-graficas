use crate::core::vec3::Vec3;

pub struct DayNight {}

impl DayNight {
    pub fn new() -> Self { Self{} }

   
    pub fn sun_direction(&self, t: f64) -> Vec3 {
        let cycle_duration = 140.0;          // día un poco más largo
        let phase = (t / cycle_duration) * std::f64::consts::TAU;

        let y = phase.sin();                 // elevación
        let x = phase.cos();                 // azimut

        Vec3::new(x, y.max(0.02), 0.20).normalized()
    }


    pub fn sun_intensity(&self, t: f64) -> f64 {
        let elev = self.sun_direction(t).y.max(0.0);
        let base = elev.powf(0.8);

        // antes: 1.0
        // soft summer: cálido y suave
        0.45 * base
    }

  
    pub fn sun_color(&self, t: f64) -> Vec3 {
        let elev = self.sun_direction(t).y;

        if elev <= 0.0 {
            return Vec3::new(0.0, 0.0, 0.0);
        }

        // Amanecer/atardecer más dorado y pastel
        let warm = Vec3::new(1.00, 0.72, 0.40); // dorado suave
        let noon = Vec3::new(1.00, 0.95, 0.88); // blanco cálido

        let k = elev.clamp(0.0, 1.0);
        warm * (1.0 - k) + noon * k
    }

  
    pub fn sky_color(&self, t: f64) -> Vec3 {
        let sun = self.sun_direction(t);
        let elev = sun.y;

        // Paletas pastel
        let zenith_day   = Vec3::new(0.55, 0.75, 1.00);  // Azul suave pastel
        let horizon_day  = Vec3::new(0.90, 0.95, 1.00);  // Azul casi blanco
        let zenith_night = Vec3::new(0.06, 0.08, 0.12);  // Azul marino leve
        let horizon_tw   = Vec3::new(0.68, 0.50, 0.72);  // Rosa-morado suave

        // Noche clara estilo verano
        if elev <= -0.03 {
            return zenith_night * 0.7 + horizon_tw * 0.3;
        }

        // Día
        let f_day = elev.clamp(0.0, 1.0);
        let base = zenith_day * 0.55 + horizon_day * 0.45;

        // Toque cálido de atardecer/amanecer pastel
        let warm_tint = Vec3::new(1.00, 0.70, 0.55);  // rosa-dorado soft
        let horizon_mix = (0.5 - elev).clamp(0.0, 0.5) / 0.5;

        // Mezcla final
        base * (1.0 - 0.15 * horizon_mix) + warm_tint * (0.10 * horizon_mix)
    }

   
    pub fn ambient_level(&self, t: f64) -> f64 {
        let elev = self.sun_direction(t).y;

        if elev < -0.2 {
            0.05  // noche suave de verano
        } else if elev < 0.0 {
            0.05 + ((elev + 0.2) / 0.2) * 0.06
        } else {
            0.12 + elev * 0.06 // más suave que antes
        }
    }
}

