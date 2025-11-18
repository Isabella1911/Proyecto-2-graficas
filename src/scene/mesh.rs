use crate::core::vec3::Vec3;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Clone, Copy)]
pub struct Tri {
    pub v0: Vec3, pub v1: Vec3, pub v2: Vec3,
    pub n:  Vec3, // normal plana
    pub mat_id: usize,
}

impl Tri {
    #[inline]
    pub fn new(v0: Vec3, v1: Vec3, v2: Vec3, n: Vec3, mat_id: usize) -> Self {
        Self { v0, v1, v2, n: n.normalized(), mat_id }
    }
}

#[inline]
fn compute_face_normal(a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    (b - a).cross(c - a).normalized()
}

// Indice OBJ -> índice 0-based en arreglo de posiciones (acepta negativos)
#[inline]
fn fix_idx(len: usize, raw: &str) -> Option<usize> {
    if raw.is_empty() { return None; }
    let i: i32 = raw.parse().ok()?;
    if i > 0 {
        let u = (i as usize).saturating_sub(1);
        if u < len { Some(u) } else { None }
    } else if i < 0 {
        let abs = (-i) as usize;
        if abs == 0 || abs > len { None } else { Some(len - abs) }
    } else {
        None
    }
}

// Triangulación en abanico: v[0], v[k], v[k+1]
#[inline]
fn push_fan(vs: &[Vec3], tris: &mut Vec<Tri>, face_idx: &[usize], mat_id: usize) {
    if face_idx.len() < 3 { return; }
    let v0 = vs[face_idx[0]];
    for k in 1..(face_idx.len() - 1) {
        let v1 = vs[face_idx[k]];
        let v2 = vs[face_idx[k + 1]];
        let e1 = v1 - v0;
        let e2 = v2 - v0;
        let n = e1.cross(e2);
        let len = n.length();
        if len <= 1e-12 { continue; } // descarta degenerados
        let n = n / len;
        tris.push(Tri { v0, v1, v2, n, mat_id });
    }
}

/// Carga triángulos desde un .obj con tolerancia de formato:
/// - Soporta índices positivos y negativos (relativos al final)
/// - Soporta caras con >3 vértices (triangulación en abanico)
/// - Soporta 'f' en formas: i, i/j, i//k, i/j/k
/// - Ignora vt/vn (normales planas por cara)
/// - Aplica `scale` y `translate` a posiciones
/// - Si el archivo no existe, devuelve `Vec::new()` sin fallar
pub fn load_obj_triangles(path: &str, mat_id: usize, scale: f64, translate: Vec3) -> Vec<Tri> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return Vec::new(), // opcional: si no existe, no truena
    };
    let reader = BufReader::new(file);

    let mut vs: Vec<Vec3> = Vec::new();
    let mut tris: Vec<Tri> = Vec::new();

    for line in reader.lines().flatten() {
        let s = line.trim();
        if s.is_empty() || s.starts_with('#') { continue; }

        if s.starts_with("v ") {
            // vértice: v x y z
            let parts: Vec<&str> = s.split_whitespace().collect();
            if parts.len() >= 4 {
                let x: f64 = parts[1].parse().unwrap_or(0.0);
                let y: f64 = parts[2].parse().unwrap_or(0.0);
                let z: f64 = parts[3].parse().unwrap_or(0.0);
                vs.push(Vec3::new(x, y, z) * scale + translate);
            }
        } else if s.starts_with("f ") {
            // Cara: i, i/j, i//k, i/j/k, con N-gons
            let mut face_idx: Vec<usize> = Vec::with_capacity(4);
            for tok in s.split_whitespace().skip(1) {
                // Toma el índice de posición (antes de '/')
                let vi_str = tok.split('/').next().unwrap_or("");
                if let Some(ix) = fix_idx(vs.len(), vi_str) {
                    face_idx.push(ix);
                }
            }
            if face_idx.len() >= 3 {
                push_fan(&vs, &mut tris, &face_idx, mat_id);
            }
        }
        // Ignoramos 'vn', 'vt', 'usemtl', 'mtllib', 'o', 'g' para mantener Tri plano
    }

    tris
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_face_normal_ok() {
        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(1.0, 0.0, 0.0);
        let c = Vec3::new(0.0, 1.0, 0.0);
        let n = compute_face_normal(a, b, c);
        // Debe ser +/- Z
        assert!( (n - Vec3::new(0.0, 0.0, 1.0)).length() < 1e-9
              || (n - Vec3::new(0.0, 0.0, -1.0)).length() < 1e-9 );
    }
}
