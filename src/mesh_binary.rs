//! Versioned little-endian rendering format. The f64 mesh remains authoritative.
use crate::geometry::Mesh;
use std::io::{self, Write};

const MAGIC: &[u8; 4] = b"G2M1";

/// Header: magic, vertex count, face count, reserved zero (all 4 bytes).
/// Payload: xyz f32 positions, xyz f32 normals, then u16 triangle indices.
pub fn write(mesh: &Mesh, mut out: impl Write) -> io::Result<()> {
    let invalid = |message| io::Error::new(io::ErrorKind::InvalidData, message);
    let vertices = mesh.positions.len();
    if vertices == 0 || vertices > usize::from(u16::MAX) + 1 || mesh.normals.len() != vertices {
        return Err(invalid("invalid vertex or normal count"));
    }
    let faces = u32::try_from(mesh.triangles.len()).map_err(|_| invalid("too many faces"))?;
    if faces == 0 || mesh.triangles.iter().flatten().any(|&i| i >= vertices) {
        return Err(invalid("invalid triangle indices"));
    }
    if mesh
        .positions
        .iter()
        .chain(&mesh.normals)
        .flatten()
        .any(|&x| !(x as f32).is_finite())
    {
        return Err(invalid("coordinate cannot be represented as finite f32"));
    }
    out.write_all(MAGIC)?;
    out.write_all(&(vertices as u32).to_le_bytes())?;
    out.write_all(&faces.to_le_bytes())?;
    out.write_all(&0u32.to_le_bytes())?;
    for &x in mesh.positions.iter().chain(&mesh.normals).flatten() {
        out.write_all(&(x as f32).to_le_bytes())?;
    }
    for &i in mesh.triangles.iter().flatten() {
        out.write_all(&(i as u16).to_le_bytes())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry;

    #[test]
    fn binary_round_trip_layout() {
        let mesh = geometry::genus_two().unwrap();
        let mut bytes = Vec::new();
        write(&mesh, &mut bytes).unwrap();
        let v = mesh.positions.len();
        let f = mesh.triangles.len();
        assert_eq!(&bytes[..4], MAGIC);
        assert_eq!(
            u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize,
            v
        );
        assert_eq!(
            u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize,
            f
        );
        assert_eq!(&bytes[12..16], &[0; 4]);
        assert_eq!(bytes.len(), 16 + 24 * v + 6 * f);
        for (&value, raw) in mesh
            .positions
            .iter()
            .chain(&mesh.normals)
            .flatten()
            .zip(bytes[16..16 + 24 * v].as_chunks::<4>().0)
        {
            assert_eq!(f32::from_le_bytes(*raw), value as f32);
        }
        for (&index, raw) in mesh
            .triangles
            .iter()
            .flatten()
            .zip(bytes[16 + 24 * v..].as_chunks::<2>().0)
        {
            assert_eq!(usize::from(u16::from_le_bytes(*raw)), index);
        }
    }

    #[test]
    fn rejects_invalid_render_data() {
        let mut mesh = geometry::genus_two().unwrap();
        mesh.triangles[0][0] = mesh.positions.len();
        assert!(write(&mesh, Vec::new()).is_err());
        mesh.triangles[0][0] = 0;
        mesh.positions[0][0] = f64::INFINITY;
        assert!(write(&mesh, Vec::new()).is_err());
    }
}
