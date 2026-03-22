//! Collision world parser — reads .bcd collision data files.
//!
//! Faithfully ported from `deimos-reference/src/collision.py`.
//!
//! Parses binary collision data from Wizard101 zone files into a structured
//! `CollisionWorld` containing proxy geometry objects (boxes, spheres,
//! cylinders, tubes, planes, meshes) with collision flags.
#![allow(dead_code, unused_imports)]

use std::io::{Cursor, Read};
use wizwalker::types::XYZ;

// ── Helper: binary reader ───────────────────────────────────────────

struct StructReader<'a> {
    cursor: Cursor<&'a [u8]>,
}

impl<'a> StructReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            cursor: Cursor::new(data),
        }
    }

    fn read_i32(&mut self) -> i32 {
        let mut buf = [0u8; 4];
        self.cursor.read_exact(&mut buf).unwrap_or_default();
        i32::from_le_bytes(buf)
    }

    fn read_u32(&mut self) -> u32 {
        let mut buf = [0u8; 4];
        self.cursor.read_exact(&mut buf).unwrap_or_default();
        u32::from_le_bytes(buf)
    }

    fn read_f32(&mut self) -> f32 {
        let mut buf = [0u8; 4];
        self.cursor.read_exact(&mut buf).unwrap_or_default();
        f32::from_le_bytes(buf)
    }

    fn read_string(&mut self) -> String {
        let length = self.read_i32();
        if length <= 0 {
            return String::new();
        }
        let mut buf = vec![0u8; length as usize];
        self.cursor.read_exact(&mut buf).unwrap_or_default();
        String::from_utf8_lossy(&buf).to_string()
    }

    fn read_xyz(&mut self) -> (f32, f32, f32) {
        (self.read_f32(), self.read_f32(), self.read_f32())
    }

    fn read_matrix3x3(&mut self) -> [f32; 9] {
        let mut mat = [0.0f32; 9];
        for m in &mut mat {
            *m = self.read_f32();
        }
        mat
    }
}

// ── Proxy types ─────────────────────────────────────────────────────

/// Proxy geometry type.
///
/// Python: `ProxyType` enum — collision.py:54
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProxyType {
    Box = 0,
    Ray = 1,
    Sphere = 2,
    Cylinder = 3,
    Tube = 4,
    Plane = 5,
    Mesh = 6,
    Invalid = 7,
}

impl ProxyType {
    fn from_i32(v: i32) -> Self {
        match v {
            0 => Self::Box,
            1 => Self::Ray,
            2 => Self::Sphere,
            3 => Self::Cylinder,
            4 => Self::Tube,
            5 => Self::Plane,
            6 => Self::Mesh,
            _ => Self::Invalid,
        }
    }
}

// ── Collision flags ─────────────────────────────────────────────────

bitflags::bitflags! {
    /// Collision flag bits.
    ///
    /// Python: `CollisionFlag` — collision.py:69
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct CollisionFlag: u32 {
        const OBJECT       = 1 << 0;
        const WALKABLE     = 1 << 1;
        const HITSCAN      = 1 << 3;
        const LOCAL_PLAYER = 1 << 4;
        const WATER        = 1 << 6;
        const CLIENT_OBJECT = 1 << 7;
        const TRIGGER      = 1 << 8;
        const FOG          = 1 << 9;
        const GOO          = 1 << 10;
        const FISH         = 1 << 11;
    }
}

// ── Geometry parameters ─────────────────────────────────────────────

/// Geometry-specific parameters for each proxy type.
///
/// Python: `GeomParams` and subclasses — collision.py:107-241
#[derive(Debug, Clone)]
pub enum GeomParams {
    Box {
        length: f32,
        width: f32,
        depth: f32,
    },
    Ray {
        position: f32,
        direction: f32,
        length: f32,
    },
    Sphere {
        radius: f32,
    },
    Cylinder {
        radius: f32,
        length: f32,
    },
    Tube {
        radius: f32,
        length: f32,
    },
    Plane {
        normal: (f32, f32, f32),
        distance: f32,
    },
    Mesh,
}

// ── Proxy geometry ──────────────────────────────────────────────────

/// A single collision proxy object.
///
/// Python: `ProxyGeometry` dataclass — collision.py:244
#[derive(Debug, Clone)]
pub struct ProxyGeometry {
    pub category_flags: CollisionFlag,
    pub collide_flag: CollisionFlag,
    pub name: String,
    pub rotation: [f32; 9],
    pub location: (f32, f32, f32),
    pub scale: f32,
    pub material: String,
    pub proxy_type: ProxyType,
    pub params: GeomParams,
    /// Only populated for Mesh proxy types.
    pub vertices: Vec<(f32, f32, f32)>,
    pub faces: Vec<(i32, i32, i32)>,
    pub normals: Vec<(f32, f32, f32)>,
}

impl ProxyGeometry {
    fn load(reader: &mut StructReader, category: CollisionFlag, collide: CollisionFlag, proxy_type: ProxyType, is_mesh: bool) -> Self {
        let mut vertices = Vec::new();
        let mut faces = Vec::new();
        let mut normals = Vec::new();

        // Mesh types read vertex/face data first
        if is_mesh {
            let vertex_count = reader.read_i32();
            let face_count = reader.read_i32();
            for _ in 0..vertex_count {
                vertices.push(reader.read_xyz());
            }
            for _ in 0..face_count {
                let a = reader.read_i32();
                let b = reader.read_i32();
                let c = reader.read_i32();
                faces.push((a, b, c));
                normals.push(reader.read_xyz());
            }
        }

        // Common fields
        let name = reader.read_string();
        let rotation = reader.read_matrix3x3();
        let location = reader.read_xyz();
        let scale = reader.read_f32();
        let material = reader.read_string();
        let _proxy_type_check = ProxyType::from_i32(reader.read_i32());

        // Type-specific params
        let params = match proxy_type {
            ProxyType::Box => GeomParams::Box {
                length: reader.read_f32(),
                width: reader.read_f32(),
                depth: reader.read_f32(),
            },
            ProxyType::Ray => GeomParams::Ray {
                position: reader.read_f32(),
                direction: reader.read_f32(),
                length: reader.read_f32(),
            },
            ProxyType::Sphere => GeomParams::Sphere {
                radius: reader.read_f32(),
            },
            ProxyType::Cylinder => GeomParams::Cylinder {
                radius: reader.read_f32(),
                length: reader.read_f32(),
            },
            ProxyType::Tube => GeomParams::Tube {
                radius: reader.read_f32(),
                length: reader.read_f32(),
            },
            ProxyType::Plane => {
                let nx = reader.read_f32();
                let ny = reader.read_f32();
                let nz = reader.read_f32();
                let d = reader.read_f32();
                GeomParams::Plane {
                    normal: (nx, ny, nz),
                    distance: d,
                }
            }
            ProxyType::Mesh | ProxyType::Invalid => GeomParams::Mesh,
        };

        Self {
            category_flags: category,
            collide_flag: collide,
            name,
            rotation,
            location,
            scale,
            material,
            proxy_type,
            params,
            vertices,
            faces,
            normals,
        }
    }

    /// Get XYZ location as a wizwalker XYZ.
    pub fn xyz(&self) -> XYZ {
        XYZ {
            x: self.location.0,
            y: self.location.1,
            z: self.location.2,
        }
    }
}

// ── Collision world ─────────────────────────────────────────────────

/// The full collision world — a list of proxy geometry objects.
///
/// Python: `CollisionWorld` dataclass — collision.py:360
#[derive(Debug, Clone, Default)]
pub struct CollisionWorld {
    pub objects: Vec<ProxyGeometry>,
}

impl CollisionWorld {
    /// Parse collision data from raw .bcd bytes.
    ///
    /// Python: `CollisionWorld.load(raw_data)` — collision.py:364
    pub fn load(raw_data: &[u8]) -> Self {
        let mut reader = StructReader::new(raw_data);
        let geometry_count = reader.read_i32();
        let mut objects = Vec::with_capacity(geometry_count.max(0) as usize);

        for _ in 0..geometry_count {
            let geometry_type = reader.read_i32();
            let category_bits = reader.read_u32();
            let collide_bits = reader.read_u32();

            let proxy_type = ProxyType::from_i32(geometry_type);
            let category = CollisionFlag::from_bits_truncate(category_bits);
            let collide = CollisionFlag::from_bits_truncate(collide_bits);

            let is_mesh = proxy_type == ProxyType::Mesh;
            let geometry = ProxyGeometry::load(&mut reader, category, collide, proxy_type, is_mesh);
            objects.push(geometry);
        }

        Self { objects }
    }

    /// Get all walkable geometry objects.
    pub fn walkable(&self) -> Vec<&ProxyGeometry> {
        self.objects
            .iter()
            .filter(|o| o.category_flags.contains(CollisionFlag::WALKABLE))
            .collect()
    }

    /// Get all object collision geometry.
    pub fn colliders(&self) -> Vec<&ProxyGeometry> {
        self.objects
            .iter()
            .filter(|o| o.category_flags.contains(CollisionFlag::OBJECT))
            .collect()
    }
}
