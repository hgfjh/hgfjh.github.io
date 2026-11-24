//! Combinatorial checks for a finite, closed, oriented triangular 2-manifold.
//!
//! These checks do not, by themselves, certify geometric embeddedness.

use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq, Eq)]
pub struct MeshStats {
    pub vertices: usize,
    pub edges: usize,
    pub faces: usize,
    pub euler: isize,
    pub genus: usize,
}

/// Each link edge (a,b) comes from a triangle (v,a,b) incident to v.
fn check_link(edges: &[(usize, usize)]) -> bool {
    let mut adjacency: HashMap<usize, Vec<usize>> = HashMap::new();
    for &(a, b) in edges {
        adjacency.entry(a).or_default().push(b);
        adjacency.entry(b).or_default().push(a);
    }
    // A simplicial circle is a connected simple graph of degree two.
    if adjacency.len() < 3 || adjacency.values().any(|n| n.len() != 2 || n[0] == n[1]) {
        return false;
    }
    let start = *adjacency.keys().next().expect("nonempty link");
    let mut visited = HashSet::new();
    let mut stack = vec![start];
    while let Some(v) = stack.pop() {
        if visited.insert(v) {
            stack.extend(adjacency[&v].iter().copied());
        }
    }
    visited.len() == adjacency.len()
}

/// Verify a connected, closed, consistently oriented simplicial surface.
/// The face order is part of the input: opposite edge directions are required.
pub fn verify(vertex_count: usize, faces: &[[usize; 3]]) -> Result<MeshStats, String> {
    if vertex_count == 0 || faces.is_empty() {
        return Err("empty mesh".into());
    }
    // For an unoriented edge, count total and increasing directed incidences.
    let mut edges: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
    let mut unique = HashSet::new();
    let mut links = vec![Vec::new(); vertex_count];
    let mut neighbors = vec![Vec::new(); vertex_count];
    for &[a, b, c] in faces {
        if [a, b, c].iter().any(|&v| v >= vertex_count) || a == b || b == c || c == a {
            return Err("invalid triangle indices".into());
        }
        let mut key = [a, b, c];
        key.sort_unstable();
        if !unique.insert(key) {
            return Err("duplicate triangle".into());
        }
        for (v, x, y) in [(a, b, c), (b, c, a), (c, a, b)] {
            links[v].push((x, y));
            neighbors[v].push(x);
            neighbors[v].push(y);
            let counts = edges.entry((v.min(x), v.max(x))).or_default();
            counts.0 += 1;
            if v < x {
                counts.1 += 1;
            }
        }
    }
    for (&edge, &counts) in &edges {
        if counts != (2, 1) {
            return Err(format!(
                "edge {edge:?}: expected two opposite incidences, got {counts:?}"
            ));
        }
    }
    for (v, link) in links.iter().enumerate() {
        if !check_link(link) {
            return Err(format!("vertex {v}: link is not a single circle"));
        }
    }
    let mut visited = vec![false; vertex_count];
    let mut stack = vec![0];
    visited[0] = true;
    while let Some(v) = stack.pop() {
        for &w in &neighbors[v] {
            if !visited[w] {
                visited[w] = true;
                stack.push(w);
            }
        }
    }
    if visited.contains(&false) {
        return Err("disconnected mesh".into());
    }
    let euler = vertex_count as isize - edges.len() as isize + faces.len() as isize;
    if euler > 2 || euler % 2 != 0 {
        return Err(format!("invalid orientable Euler characteristic: {euler}"));
    }
    Ok(MeshStats {
        vertices: vertex_count,
        edges: edges.len(),
        faces: faces.len(),
        euler,
        genus: ((2 - euler) / 2) as usize,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const TETRA: [[usize; 3]; 4] = [[0, 2, 1], [0, 1, 3], [1, 2, 3], [2, 0, 3]];

    #[test]
    fn sphere_has_genus_zero() {
        assert_eq!(verify(4, &TETRA).unwrap().genus, 0);
    }

    #[test]
    fn rejects_boundary_bad_orientation_and_duplicate_faces() {
        assert!(verify(4, &TETRA[..3]).is_err());
        let mut reversed = TETRA;
        reversed[0].swap(1, 2);
        assert!(verify(4, &reversed).is_err());
        let mut duplicate = TETRA.to_vec();
        duplicate.push(TETRA[0]);
        assert!(verify(4, &duplicate).is_err());
        assert!(verify(3, &TETRA).is_err());
    }

    #[test]
    fn rejects_two_spheres_touching_at_a_vertex() {
        let mut pinched = TETRA.to_vec();
        pinched.extend(TETRA.map(|f| f.map(|v| if v == 0 { 0 } else { v + 3 })));
        assert!(verify(7, &pinched).unwrap_err().contains("link"));
    }

    #[test]
    fn rejects_disconnected_closed_surfaces() {
        let mut disconnected = TETRA.to_vec();
        disconnected.extend(TETRA.map(|f| f.map(|v| v + 4)));
        assert!(
            verify(8, &disconnected)
                .unwrap_err()
                .contains("disconnected")
        );
    }
}
