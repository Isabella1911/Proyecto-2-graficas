use crate::core::vec3::Vec3;

pub struct DayNight {}

impl DayNight {
    pub fn new() -> Self { Self{} }

    /// Dirección del sol (ciclo rápido para demo: 120 s)
    pub fn sun_direction(&self, t: f64) -> Vec3 {
        let cycle_duration = 120.0;
        let phase = (t / cycle_duration) * std::f64::consts::TAU;
        // El sol gira en el plano X–Y (Z levemente inclinado para que no sea totalmente plano)
        let y = phase.sin();      // elevación
        let x = phase.cos();      // azimut
        Vec3::new(x, y.max(0.02), 0.25).normalized()
    }

    /// Intensidad del sol según elevación (menos quemado)
    pub fn sun_intensity(&self, t: f64) -> f64 {
        let elev = self.sun_direction(t).y.max(0.0);
        // Curva suave: cero en el horizonte, máximo al cenit
        let k = elev.powf(0.7);
        // Antes estaba muy alto; bajamos el techo
        1.0 * k
    }

    /// Color del sol (cálido al amanecer/atardecer)
    pub fn sun_color(&self, t: f64) -> Vec3 {
        let elev = self.sun_direction(t).y;
        if elev <= 0.0 {
            // Debajo del horizonte: no contribuye
            return Vec3::new(0.0, 0.0, 0.0);
        }
        // Interpolar entre naranja (bajo) y blanco cálido (alto)
        let warm = Vec3::new(1.00, 0.62, 0.22); // amanecer/atardecer
        let noon = Vec3::new(1.00, 0.96, 0.90); // mediodía cálido (no blanco puro)
        let t = (elev / 1.0).clamp(0.0, 1.0);
        warm * (1.0 - t) + noon * t
    }

    /// Color base del cielo (zenit/horizonte + tinte según hora)
    pub fn sky_color(&self, t: f64) -> Vec3 {
        let sun = self.sun_direction(t);
        let elev = sun.y; // ~[-1,1]
        // Paletas
        let zenith_day   = Vec3::new(0.20, 0.45, 0.95); // azul
        let horizon_day  = Vec3::new(0.65, 0.80, 1.00); // más claro
        let zenith_night = Vec3::new(0.02, 0.03, 0.08); // noche
        let horizon_tw   = Vec3::new(0.35, 0.20, 0.40); // crepúsculo morado

        if elev <= -0.05 {
            // Noche: gradiente suave
            return zenith_night * 0.8 + horizon_tw * 0.2;
        }

        // Día/tarde: mezclamos cielo diurno y tinte cálido cerca del horizonte
        // f: factor de "día" según elevación del sol
        let f_day = elev.clamp(0.0, 1.0);
        let base = zenith_day * 0.6 + horizon_day * 0.4;

        // Tinte cálido cerca del horizonte (amanecer/atardecer)
        let warm_tint = Vec3::new(1.0, 0.55, 0.25);
        let horizon_mix = (0.6 - elev).clamp(0.0, 0.6) / 0.6; // más fuerte con el sol bajo

        base * (1.0 - 0.25 * horizon_mix) + warm_tint * (0.15 * horizon_mix) * f_day
    }

    /// Luz ambiental (hemisférica) más moderada
    pub fn ambient_level(&self, t: f64) -> f64 {
        let elev = self.sun_direction(t).y; // [-1,1]
        if elev < -0.2 {
            0.06  // noche
        } else if elev < 0.0 {
            0.06 + ((elev + 0.2) / 0.2) * 0.10 // transición
        } else {
            0.16 + elev * 0.12 // día
        }
    }
}
