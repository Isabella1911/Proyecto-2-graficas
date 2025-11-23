use crate::core::math::Vec3;

pub struct DayNight {}

impl DayNight {
    pub fn new() -> Self { Self {} }

    // ---------------------------------------------------------
    // Sol recorriendo el cielo en un arco suave
    // ---------------------------------------------------------
    pub fn sun_direction(&self, t: f64) -> Vec3 {
        let cycle = 140.0; // duración del día
        let phase = (t / cycle) * std::f64::consts::TAU;

        // Movimiento circular simple
        let y = phase.sin();       // altura
        let x = phase.cos();       // azimut
        
        // Límite para evitar y <= 0 demasiado negro
        Vec3::new(x, y.max(0.02), 0.15).normalized()
    }

    // ---------------------------------------------------------
    // Intensidad del sol según la altura (más suave al amanecer/tarde)
    // ---------------------------------------------------------
    pub fn sun_intensity(&self, t: f64) -> f64 {
        let elev = self.sun_direction(t).y.max(0.0);
        // curva más suave
        0.45 * elev.powf(0.8)
    }

    // ---------------------------------------------------------
    // Color del sol: dorado → blanco cálido → dorado
    // ---------------------------------------------------------
    pub fn sun_color(&self, t: f64) -> Vec3 {
        let elev = self.sun_direction(t).y;

        if elev <= 0.0 {
            return Vec3::new(0.0, 0.0, 0.0); // sol debajo del horizonte
        }

        let dawn = Vec3::new(1.00, 0.72, 0.40); // amanecer pastel
        let noon = Vec3::new(1.00, 0.95, 0.88); // mediodía suave

        let k = elev.clamp(0.0, 1.0);
        dawn * (1.0 - k) + noon * k
    }

    // ---------------------------------------------------------
    // Color del cielo procedural pastel
    // ---------------------------------------------------------
    pub fn sky_color(&self, t: f64) -> Vec3 {
        let sun_dir = self.sun_direction(t);
        let elev = sun_dir.y;

        // noche suave
        let night = Vec3::new(0.06, 0.08, 0.12);
        let twilight = Vec3::new(0.68, 0.50, 0.72); // rosa-morado pastel

        let zenith_day = Vec3::new(0.55, 0.75, 1.00);   // azul pastel
        let horizon_day = Vec3::new(0.90, 0.95, 1.00);  // casi blanco

        if elev <= -0.03 {
            // mezcla noche + twilight
            return night * 0.7 + twilight * 0.3;
        }

        let k = elev.clamp(0.0, 1.0);
        let base = zenith_day * 0.55 + horizon_day * 0.45;

        // tint cálido cerca del horizonte
        let warm = Vec3::new(1.00, 0.70, 0.55);
        let horizon_factor = (0.5 - elev).clamp(0.0, 0.5) / 0.5;

        base * (1.0 - 0.15 * horizon_factor) + warm * (0.10 * horizon_factor)
    }

    // ---------------------------------------------------------
    // Ambiente dependiendo de la hora: suave siempre
    // ---------------------------------------------------------
    pub fn ambient_level(&self, t: f64) -> f64 {
        let elev = self.sun_direction(t).y;

        if elev < -0.2 {
            return 0.05
        }
        if elev < 0.0 {
            return 0.05 + ((elev + 0.2) / 0.2) * 0.06;
        }

        0.12 + elev * 0.06
    }
}
