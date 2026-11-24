//! A genus-two surface obtained by doubling a polygonal pair of pants.
//! Sample z² = LAMBDA² q0 q1 q2, then apply a smooth waist deformation.
//! See docs/genus-two.md for the embedding argument.

use crate::topology::{self, MeshStats};
use std::{
    collections::HashMap,
    f64::consts::PI,
    io::{self, Write},
};

const A: f64 = 2.65;
const B: f64 = 1.35;
const C: f64 = 1.12;
const R: f64 = 0.53;
const LAMBDA: f64 = 0.43;
const ARC: usize = 64;
const SEAM: usize = 32;
const RADIAL: usize = 16;

type Point = [f64; 3];
type Face = [usize; 3];
type PlanarMesh = (Vec<PlanarVertex>, Vec<Face>);

pub struct Mesh {
    pub positions: Vec<Point>,
    pub normals: Vec<Point>,
    pub triangles: Vec<Face>,
    pub stats: MeshStats,
}

#[derive(Clone, Copy)]
struct PlanarVertex {
    x: f64,
    y: f64,
    boundary: bool,
}

/// Return P and its two partial derivatives by the product rule.
fn polynomial(x: f64, y: f64) -> (f64, f64, f64) {
    let q0 = 1.0 - (x / A).powi(2) - (y / B).powi(2);
    let q1 = (x - C).powi(2) + y * y - R * R;
    let q2 = (x + C).powi(2) + y * y - R * R;
    let px = -2.0 * x / (A * A) * q1 * q2 + q0 * 2.0 * (x - C) * q2 + q0 * q1 * 2.0 * (x + C);
    let py = -2.0 * y / (B * B) * q1 * q2 + q0 * 2.0 * y * (q1 + q2);
    (q0 * q1 * q2, px, py)
}

/// The ray from (C,0) intersects the ellipse in a positive quadratic root.
fn ellipse_radius(dx: f64, dy: f64) -> f64 {
    let alpha = (dx / A).powi(2) + (dy / B).powi(2);
    let beta = 2.0 * C * dx / (A * A);
    let gamma = (C / A).powi(2) - 1.0;
    (-beta + (beta * beta - 4.0 * alpha * gamma).sqrt()) / (2.0 * alpha)
}

fn area2(a: PlanarVertex, b: PlanarVertex, c: PlanarVertex) -> f64 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

/// Check the local prerequisites of the two-graph embedding argument.
/// Global nonintersection of the planar mesh follows from its radial wedges.
fn check_planar(vertices: &[PlanarVertex], faces: &[Face]) -> Result<(), String> {
    let mut edges = HashMap::new();
    for &[a, b, c] in faces {
        let area = area2(vertices[a], vertices[b], vertices[c]);
        if !area.is_finite() || area <= 0.0 {
            return Err("nonpositive projected triangle area".into());
        }
        if [a, b, c].iter().all(|&i| vertices[i].boundary) {
            return Err("all-boundary triangle would collapse the double".into());
        }
        for (v, w) in [(a, b), (b, c), (c, a)] {
            *edges.entry((v.min(w), v.max(w))).or_insert(0) += 1;
        }
    }
    for ((a, b), n) in edges {
        let both_boundary = vertices[a].boundary && vertices[b].boundary;
        if !matches!((n, both_boundary), (1, true) | (2, false)) {
            return Err("planar boundary/edge incidence mismatch".into());
        }
    }
    Ok(())
}

/// Build the right annulus, then reflect it. Seam vertices share indices;
/// no approximate coordinate welding is needed.
fn planar_mesh(arc: usize, seam: usize, radial: usize) -> Result<PlanarMesh, String> {
    if arc < 4 || seam < 2 || radial < 2 {
        return Err("insufficient angular or radial subdivisions".into());
    }
    let n = arc + seam;
    let corner = B.atan2(-C);
    let mut vertices = Vec::new();
    for i in 0..n {
        // Sample both corners exactly; all seam rays are consecutive.
        let theta = if i <= arc {
            -corner + 2.0 * corner * i as f64 / arc as f64
        } else {
            corner + (2.0 * PI - 2.0 * corner) * (i - arc) as f64 / seam as f64
        };
        let (dy, dx) = theta.sin_cos();
        let on_seam = i == 0 || i >= arc;
        let outer = if on_seam {
            -C / dx
        } else {
            ellipse_radius(dx, dy)
        };
        if outer <= R {
            return Err("hole is not strictly inside the half ellipse".into());
        }
        for j in 0..=radial {
            // Cosine spacing samples more densely near the rounded rims.
            let t = (1.0 - (PI * j as f64 / radial as f64).cos()) / 2.0;
            let rho = R + t * (outer - R);
            let mut x = C + rho * dx;
            let mut y = rho * dy;
            if on_seam && j == radial {
                x = 0.0;
                if i == 0 {
                    y = -B;
                }
                if i == arc {
                    y = B;
                }
            }
            let boundary = j == 0 || (j == radial && (i <= arc));
            vertices.push(PlanarVertex { x, y, boundary });
        }
    }
    let right_count = vertices.len();
    let mut mirror = Vec::with_capacity(right_count);
    let mut reflected = Vec::with_capacity(right_count);
    for (i, &v) in vertices.iter().enumerate() {
        let angular = i / (radial + 1);
        let is_seam = i % (radial + 1) == radial && (angular == 0 || angular >= arc);
        if is_seam {
            mirror.push(i);
        } else {
            mirror.push(right_count + reflected.len());
            reflected.push(PlanarVertex { x: -v.x, ..v });
        }
    }
    vertices.extend(reflected);
    let mut faces = Vec::new();
    for i in 0..n {
        let next = (i + 1) % n;
        for j in 0..radial {
            let a = i * (radial + 1) + j;
            let b = a + 1;
            let c = next * (radial + 1) + j + 1;
            let d = c - 1;
            // Increasing radius then increasing angle gives CCW orientation.
            for [u, v, w] in [[a, b, c], [a, c, d]] {
                faces.push([u, v, w]);
                // Reflection reverses orientation; reverse the triangle too.
                faces.push([mirror[u], mirror[w], mirror[v]]);
            }
        }
    }
    check_planar(&vertices, &faces)?;
    Ok((vertices, faces))
}

/// Positive transverse scales and their x derivatives. The x coordinate is
/// unchanged, making the analytic map a global ambient diffeomorphism.
fn waist(x: f64) -> (f64, f64, f64, f64) {
    let width = 0.65_f64;
    let e = (-(x / width).powi(2)).exp();
    let de = -2.0 * x / width.powi(2) * e;
    (
        1.0 - 0.60 * e,
        -0.60 * de,
        1.12 * (1.0 - 0.40 * e),
        -1.12 * 0.40 * de,
    )
}

fn deform([x, y, z]: Point, [nx, ny, nz]: Point) -> Result<(Point, Point), String> {
    let (s, ds, t, dt) = waist(x);
    // A normal is a covector: apply the inverse transpose of the Jacobian.
    let normal = [nx - ds * y * ny / s - dt * z * nz / t, ny / s, nz / t];
    let length = normal.iter().map(|v| v * v).sum::<f64>().sqrt();
    if !length.is_finite() || length <= 0.0 {
        return Err("invalid deformed normal".into());
    }
    Ok(([x, s * y, t * z], normal.map(|v| v / length)))
}

/// Prove the PL planar warp is injective using vertical fibres. Temporarily
/// fill both polygonal holes by fans from their centres. The resulting original
/// domain is a convex polygon. On every nonvertical triangle of that filled
/// disk, positive projected area means the new y is strictly increasing with
/// the old y at fixed x. Continuity extends this to edges and vertical fibres
/// through vertices. The added fan triangles are used only for this check.
fn check_waist(vertices: &[PlanarVertex], faces: &[Face], angular: usize) -> Result<(), String> {
    let warped: Vec<_> = vertices
        .iter()
        .map(|&v| PlanarVertex {
            y: waist(v.x).0 * v.y,
            ..v
        })
        .collect();
    check_planar(&warped, faces)?;
    let mut incidence = HashMap::new();
    for &[a, b, c] in faces {
        for (u, v) in [(a, b), (b, c), (c, a)] {
            let key = (u.min(v), u.max(v));
            let entry = incidence.entry(key).or_insert((0, u, v));
            entry.0 += 1;
        }
    }
    let mut hole_edges = 0;
    for (count, u, v) in incidence.into_values() {
        if count != 1 {
            continue;
        }
        let a = vertices[u];
        let b = vertices[v];
        for x in [-C, C] {
            let on_hole = |p: PlanarVertex| ((p.x - x).powi(2) + p.y * p.y - R * R).abs() < 1e-10;
            if on_hole(a) && on_hole(b) {
                let center = PlanarVertex {
                    x,
                    y: 0.0,
                    boundary: false,
                };
                // Boundary orientation is clockwise on the holes; reverse it
                // for the added disk. Its centre is fixed by the deformation.
                let old_area = area2(b, a, center);
                let new_area = area2(warped[v], warped[u], center);
                if !old_area.is_finite()
                    || old_area <= 0.0
                    || !new_area.is_finite()
                    || new_area <= 0.0
                {
                    return Err("waist deformation inverts a hole-filling triangle".into());
                }
                hole_edges += 1;
            }
        }
    }
    // Each hole has one boundary edge per angular interval, and no other
    // boundary edge lies on either circle. This also guards the fan selection.
    if hole_edges != 2 * angular {
        return Err("missing hole-filling triangles".into());
    }
    Ok(())
}

fn unit_normal(px: f64, py: f64, z: f64) -> Result<Point, String> {
    // The implicit function is F = z² - lambda² P; grad F points outward.
    let n = [-LAMBDA * LAMBDA * px, -LAMBDA * LAMBDA * py, 2.0 * z];
    let length = n.iter().map(|x| x * x).sum::<f64>().sqrt();
    if !length.is_finite() || length <= 0.0 {
        return Err("invalid implicit normal".into());
    }
    Ok(n.map(|x| x / length))
}

fn build(arc: usize, seam: usize, radial: usize) -> Result<Mesh, String> {
    let (planar, faces) = planar_mesh(arc, seam, radial)?;
    check_waist(&planar, &faces, arc + seam)?;
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut upper = Vec::new();
    let mut lower = Vec::new();
    for v in planar {
        let (p, px, py) = polynomial(v.x, v.y);
        // Clamp only known boundary vertices, never conceal an interior error.
        if !p.is_finite() || (v.boundary && p.abs() > 1e-10) || (!v.boundary && p <= 0.0) {
            return Err(format!(
                "unexpected polynomial value {p} at ({}, {})",
                v.x, v.y
            ));
        }
        let z = if v.boundary { 0.0 } else { LAMBDA * p.sqrt() };
        let upper_index = positions.len();
        upper.push(upper_index);
        let (position, normal) = deform([v.x, v.y, z], unit_normal(px, py, z)?)?;
        positions.push(position);
        normals.push(normal);
        if v.boundary {
            lower.push(upper_index);
        } else {
            lower.push(positions.len());
            let (position, normal) = deform([v.x, v.y, -z], unit_normal(px, py, -z)?)?;
            positions.push(position);
            normals.push(normal);
        }
    }
    let mut triangles = Vec::with_capacity(faces.len() * 2);
    for [a, b, c] in faces {
        triangles.push([upper[a], upper[b], upper[c]]);
        triangles.push([lower[a], lower[c], lower[b]]);
    }
    let stats = topology::verify(positions.len(), &triangles)?;
    if stats.euler != -2 || stats.genus != 2 {
        return Err(format!("expected genus two, got {stats:?}"));
    }
    Ok(Mesh {
        positions,
        normals,
        triangles,
        stats,
    })
}

pub fn genus_two() -> Result<Mesh, String> {
    build(ARC, SEAM, RADIAL)
}

impl Mesh {
    /// Compact JSON with round-trip f64 coordinates and zero-based indices.
    /// std-only serialization is straightforward for these fixed-size arrays.
    pub fn write_json(&self, mut out: impl Write) -> io::Result<()> {
        fn triples<T: std::fmt::Display>(
            out: &mut impl Write,
            values: &[[T; 3]],
        ) -> io::Result<()> {
            write!(out, "[")?;
            for (i, [a, b, c]) in values.iter().enumerate() {
                if i != 0 {
                    write!(out, ",")?;
                }
                write!(out, "[{a},{b},{c}]")?;
            }
            write!(out, "]")
        }
        write!(out, "{{\"positions\":")?;
        triples(&mut out, &self.positions)?;
        write!(out, ",\"normals\":")?;
        triples(&mut out, &self.normals)?;
        write!(out, ",\"triangles\":")?;
        triples(&mut out, &self.triangles)?;
        writeln!(
            out,
            ",\"stats\":{{\"vertices\":{},\"edges\":{},\"faces\":{},\"euler\":{},\"genus\":{}}}}}",
            self.stats.vertices,
            self.stats.edges,
            self.stats.faces,
            self.stats.euler,
            self.stats.genus
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_mesh_counts_and_normals() {
        let mesh = genus_two().unwrap();
        assert_eq!(
            mesh.stats,
            MeshStats {
                vertices: 6142,
                edges: 18432,
                faces: 12288,
                euler: -2,
                genus: 2,
            }
        );
        assert_eq!(mesh.positions.len(), mesh.normals.len());
        for (&[x, y, z], n) in mesh.positions.iter().zip(&mesh.normals) {
            let (s, _, t, _) = waist(x);
            let (y, z) = (y / s, z / t);
            let (p, px, py) = polynomial(x, y);
            assert!((z * z - LAMBDA * LAMBDA * p).abs() < 1e-10);
            let (_, expected) = deform([x, y, z], unit_normal(px, py, z).unwrap()).unwrap();
            assert!(n.iter().zip(expected).all(|(&a, b)| (a - b).abs() < 1e-12));
            assert!((n.iter().map(|v| v * v).sum::<f64>() - 1.0).abs() < 1e-12);
        }
    }

    #[test]
    fn different_subdivisions_preserve_genus() {
        for (arc, seam, radial) in [(8, 4, 2), (17, 7, 5), (32, 16, 8)] {
            assert_eq!(build(arc, seam, radial).unwrap().stats.genus, 2);
        }
        assert!(build(4, 2, 1).is_err());
    }

    #[test]
    fn json_contains_geometry_and_certificate() {
        let mesh = build(8, 4, 2).unwrap();
        let mut bytes = Vec::new();
        mesh.write_json(&mut bytes).unwrap();
        let json = String::from_utf8(bytes).unwrap();
        assert!(json.starts_with("{\"positions\":[["));
        assert!(json.contains("\"normals\":"));
        assert!(json.contains("\"triangles\":"));
        assert!(json.ends_with("\"euler\":-2,\"genus\":2}}\n"));
    }
}
