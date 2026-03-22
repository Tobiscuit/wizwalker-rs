//! Teleport math utilities — navmap parsing, pathfinding, yaw/pitch, spiral TP.
//!
//! Faithfully ported from `deimos-reference/src/teleport_math.py`.
//!
//! # Functions
//! - Pure math: `calculate_yaw`, `calculate_pitch`, `calc_distance`, `calc_square_distance`,
//!   `calc_point_on_3d_line`, `are_xyzs_within_threshold`, `rotate_point`, `calc_chunks`
//! - Navmap: `parse_nav_data`, `get_neighbors`
//! - Teleport helpers: `teleport_move_adjust`, `navmap_tp`, `auto_adjusting_teleport`,
//!   `fallback_spiral_tp`, `calc_frontal_vector`
#![allow(dead_code, unused_imports)]

use std::io::{Cursor, Read};
use wizwalker::types::XYZ;
use wizwalker::client::Client;
use wizwalker::constants::Keycode;

// ── Pure math functions ─────────────────────────────────────────────

/// Calculate the yaw angle (radians) from `a` looking toward `b`.
///
/// Python: `calculate_yaw(xyz_1, xyz_2)` — teleport_math.py:391
pub fn calculate_yaw(a: &XYZ, b: &XYZ) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    dx.atan2(dy)
}

/// Calculate the pitch angle (radians) from `a` looking toward `b`.
///
/// Python: `calculate_pitch(xyz_1, xyz_2)` — teleport_math.py:399
pub fn calculate_pitch(a: &XYZ, b: &XYZ) -> f32 {
    let x = b.x - a.x;
    let y = b.y - a.y;
    let z = b.z - a.z;
    -(z.atan2((x * x + y * y).sqrt()))
}

/// Squared distance between two points (no sqrt — faster for comparisons).
///
/// Python: `calc_squareDistance(xyz_1, xyz_2)` — teleport_math.py:133
pub fn calc_square_distance(a: &XYZ, b: &XYZ) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    let dz = a.z - b.z;
    dx * dx + dy * dy + dz * dz
}

/// Euclidean distance between two points.
///
/// Python: `calc_Distance(xyz_1, xyz_2)` — teleport_math.py:138
pub fn calc_distance(a: &XYZ, b: &XYZ) -> f32 {
    calc_square_distance(a, b).sqrt()
}

/// Extend a point along the line from `origin` to `target` by `additional_distance`.
///
/// Python: `calc_PointOn3DLine(xyz_1, xyz_2, additional_distance)` — teleport_math.py:115
pub fn calc_point_on_3d_line(origin: &XYZ, target: &XYZ, additional_distance: f32) -> XYZ {
    let distance = calc_distance(origin, target);
    if distance < 1.0 {
        return *origin;
    }
    let n = (distance - additional_distance) / distance;
    XYZ {
        x: (target.x - origin.x) * n + origin.x,
        y: (target.y - origin.y) * n + origin.y,
        z: (target.z - origin.z) * n + origin.z,
    }
}

/// Check if two XYZs are within a rough distance threshold of each other.
///
/// Not actual distance checking, but precision isn't needed — exists to
/// eliminate tiny variations in XYZ when being sent back from a failed port.
///
/// Python: `are_xyzs_within_threshold(xyz_1, xyz_2, threshold)` — teleport_math.py:127
pub fn are_xyzs_within_threshold(a: &XYZ, b: &XYZ, threshold: f32) -> bool {
    (a.x.abs() - b.x.abs()).abs() < threshold
        && (a.y.abs() - b.y.abs()).abs() < threshold
        && (a.z.abs() - b.z.abs()).abs() < threshold
}

/// Rotate `point` about `origin` by `theta` degrees counterclockwise (XY plane only).
///
/// Python: `rotate_point(origin_xyz, point_xyz, theta)` — teleport_math.py:199
pub fn rotate_point(origin: &XYZ, point: &XYZ, theta_degrees: f32) -> XYZ {
    let radians = theta_degrees.to_radians();
    let cos = radians.cos();
    let sin = radians.sin();
    let x_diff = point.x - origin.x;
    let y_diff = point.y - origin.y;
    XYZ {
        x: cos * x_diff - sin * y_diff + origin.x,
        y: sin * x_diff + cos * y_diff + origin.y,
        z: point.z,
    }
}

/// Calculate the frontal vector — a point "in front" of the player using their yaw.
///
/// Python: `calc_FrontalVector(client, xyz, yaw, speed_constant, ...)` — teleport_math.py:143
pub fn calc_frontal_vector(
    xyz: &XYZ,
    yaw: f32,
    speed_multiplier: f32,
    speed_constant: f32,
    length_adjusted: bool,
) -> XYZ {
    let additional_distance = speed_constant * ((speed_multiplier / 100.0) + 1.0);

    let frontal_x = xyz.x - (additional_distance * yaw.sin());
    let frontal_y = xyz.y - (additional_distance * yaw.cos());
    let frontal_xyz = XYZ {
        x: frontal_x,
        y: frontal_y,
        z: xyz.z,
    };

    if length_adjusted {
        let distance = calc_distance(xyz, &frontal_xyz);
        calc_point_on_3d_line(xyz, &frontal_xyz, additional_distance - distance)
    } else {
        frontal_xyz
    }
}

// ── Navmap parsing ──────────────────────────────────────────────────

/// Read a little-endian value from a cursor.
fn read_i16(cursor: &mut Cursor<&[u8]>) -> i16 {
    let mut buf = [0u8; 2];
    cursor.read_exact(&mut buf).unwrap_or_default();
    i16::from_le_bytes(buf)
}

fn read_i32(cursor: &mut Cursor<&[u8]>) -> i32 {
    let mut buf = [0u8; 4];
    cursor.read_exact(&mut buf).unwrap_or_default();
    i32::from_le_bytes(buf)
}

fn read_f32(cursor: &mut Cursor<&[u8]>) -> f32 {
    let mut buf = [0u8; 4];
    cursor.read_exact(&mut buf).unwrap_or_default();
    f32::from_le_bytes(buf)
}

/// Parse navmesh data from a zone.nav file.
///
/// Returns (vertices, edges) where edges are (start_index, end_index) pairs.
///
/// Python: `parse_nav_data(file_data)` — teleport_math.py:69
/// Implemented from <https://github.com/PeechezNCreem/navwiz/>
/// Licensed under the Boost Software License 1.0.
pub fn parse_nav_data(file_data: &[u8]) -> (Vec<XYZ>, Vec<(u16, u16)>) {
    let mut cursor = Cursor::new(file_data);

    let _vertex_count = read_i16(&mut cursor);
    let mut vertex_max = read_i16(&mut cursor);
    let _unknown = read_i16(&mut cursor); // unknown bytes

    let mut vertices = Vec::new();
    let mut idx: i16 = 0;
    while idx <= vertex_max - 1 {
        let x = read_f32(&mut cursor);
        let y = read_f32(&mut cursor);
        let z = read_f32(&mut cursor);
        vertices.push(XYZ { x, y, z });

        let vertex_index = read_i16(&mut cursor);
        if vertex_index != idx {
            vertices.pop();
            vertex_max -= 1;
        } else {
            idx += 1;
        }
    }

    let edge_count = read_i32(&mut cursor);
    let mut edges = Vec::with_capacity(edge_count as usize);
    for _ in 0..edge_count {
        let start = read_i16(&mut cursor) as u16;
        let stop = read_i16(&mut cursor) as u16;
        edges.push((start, stop));
    }

    (vertices, edges)
}

/// Get neighbors of a vertex from the navmesh edge list.
///
/// Python: `get_neighbors(vertex, vertices, edges)` — teleport_math.py:98
pub fn get_neighbors<'a>(
    vertex: &XYZ,
    vertices: &'a [XYZ],
    edges: &[(u16, u16)],
) -> Vec<&'a XYZ> {
    let vert_idx = vertices.iter().position(|v| {
        (v.x - vertex.x).abs() < f32::EPSILON
            && (v.y - vertex.y).abs() < f32::EPSILON
            && (v.z - vertex.z).abs() < f32::EPSILON
    });

    let Some(idx) = vert_idx else {
        return Vec::new();
    };

    edges
        .iter()
        .filter(|(start, _)| *start as usize == idx)
        .filter_map(|(_, stop)| vertices.get(*stop as usize))
        .collect()
}

// ── Teleport helper functions (async) ───────────────────────────────

/// Teleport the client to a given XYZ, then jitter to update position.
///
/// Python: `teleport_move_adjust(client, xyz, delay)` — teleport_math.py:183
pub async fn teleport_move_adjust(
    client: &Client,
    xyz: &XYZ,
    delay_ms: u64,
) {
    if let Err(e) = client.teleport(xyz) {
        tracing::warn!("teleport_move_adjust: teleport failed: {e}");
        return;
    }
    tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
    client.send_key(Keycode::A);
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    client.send_key(Keycode::D);
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}

/// Spiral-pattern teleport fallback — brute forces XYZs in an alternating spiral.
///
/// Python: `auto_adjusting_teleport(client, quest_position)` — teleport_math.py:210
pub async fn auto_adjusting_teleport(
    client: &Client,
    quest_position: &XYZ,
) {
    let original_zone_name = client.zone_name().unwrap_or_default();
    let Some(original_position) = client.body_position() else { return };

    let mut adjusted_position = *quest_position;
    let mut mod_amount = 50.0f32;
    let mut current_angle = 0.0f32;

    // Initial teleport attempt
    teleport_move_adjust(client, quest_position, 700).await;

    loop {
        let Some(current_pos) = client.body_position() else { break };

        // Check if we've moved or zone changed
        if !are_xyzs_within_threshold(&current_pos, &original_position, 50.0)
            || client.zone_name().as_deref() != Some(original_zone_name.as_str())
        {
            break;
        }

        if !are_xyzs_within_threshold(&original_position, quest_position, 1.0) {
            adjusted_position =
                calc_point_on_3d_line(&original_position, quest_position, mod_amount);
            let rotated = rotate_point(quest_position, &adjusted_position, current_angle);
            teleport_move_adjust(client, &rotated, 700).await;

            mod_amount += 100.0;
            current_angle += 92.0;
        } else {
            break;
        }
    }
}

/// Alias for `auto_adjusting_teleport`.
///
/// Python: `fallback_spiral_tp(client, xyz)` — teleport_math.py:239
pub async fn fallback_spiral_tp(client: &Client, xyz: &XYZ) {
    auto_adjusting_teleport(client, xyz).await;
}

/// Navmap-based teleport — tries direct TP, then nav data, then spiral fallback.
///
/// Python: `navmap_tp(client, xyz)` — teleport_math.py:242
pub async fn navmap_tp(client: &Client, xyz: Option<&XYZ>) {
    let starting_zone = client.zone_name().unwrap_or_default();
    let Some(starting_xyz) = client.body_position() else { return };
    let quest_pos_owned;
    let target_xyz = match xyz {
        Some(pos) => pos,
        None => {
            quest_pos_owned = client.quest_position().unwrap_or(XYZ::default());
            &quest_pos_owned
        }
    };

    // Skip if already at target
    if calc_distance(&starting_xyz, target_xyz) <= 5.0 {
        return;
    }

    // Try direct teleport first
    let _ = client.teleport(target_xyz);
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let Some(current_pos) = client.body_position() else { return };
    if calc_distance(&current_pos, &starting_xyz) > 5.0 || client.zone_name().as_deref() != Some(starting_zone.as_str()) {
        return; // Direct TP worked
    }

    // Attempt navmap-based teleport
    // Load WAD and parse nav data
    let nav_result = load_and_parse_nav(starting_zone.as_str());
    match nav_result {
        Some((vertices, edges)) => {
            // Find closest vertex to target
            let mut closest_idx = 0usize;
            let mut lowest_dist = calc_distance(&vertices[0], target_xyz);
            for (i, v) in vertices.iter().enumerate().skip(1) {
                let d = calc_distance(v, target_xyz);
                if d < lowest_dist {
                    closest_idx = i;
                    lowest_dist = d;
                }
            }

            // BFS from closest vertex, max depth 3
            let closest_vertex = &vertices[closest_idx];
            let mut relevant: Vec<XYZ> = Vec::new();
            let mut queue: Vec<Vec<XYZ>> = vec![vec![*closest_vertex]];
            let max_depth = 3;

            while let Some(path) = queue.pop() {
                let v = path.last().unwrap();
                if !relevant.iter().any(|r| calc_distance(r, v) < 0.1) {
                    relevant.push(*v);
                }
                for neighbor in get_neighbors(v, &vertices, &edges) {
                    if path.len() + 1 > max_depth {
                        continue;
                    }
                    if relevant.iter().any(|r| calc_distance(r, neighbor) < 0.1) {
                        continue;
                    }
                    let mut new_path = path.clone();
                    new_path.push(*neighbor);
                    queue.push(new_path);
                }
            }

            if relevant.is_empty() {
                fallback_spiral_tp(client, target_xyz).await;
                return;
            }

            // Average position of relevant vertices
            let count = relevant.len() as f32;
            let avg = XYZ {
                x: relevant.iter().map(|v| v.x).sum::<f32>() / count,
                y: relevant.iter().map(|v| v.y).sum::<f32>() / count,
                z: relevant.iter().map(|v| v.z).sum::<f32>() / count,
            };

            // Vector from average to target, midpoint
            let av = XYZ {
                x: target_xyz.x - avg.x,
                y: target_xyz.y - avg.y,
                z: avg.z - target_xyz.z,
            };
            let midpoint = XYZ {
                x: avg.x + av.x / 2.0,
                y: avg.y + av.y / 2.0,
                z: avg.z + av.z / 2.0,
            };

            // Try midpoint
            let _ = client.teleport(&midpoint);
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            let Some(pos) = client.body_position() else { return };
            if calc_distance(&pos, &starting_xyz) > 5.0 {
                return; // Midpoint worked
            }

            // Try average point
            let _ = client.teleport(&avg);
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            let Some(pos2) = client.body_position() else { return };
            if calc_distance(&pos2, &starting_xyz) > 5.0 {
                return; // Average worked
            }

            // Final fallback
            fallback_spiral_tp(client, target_xyz).await;
        }
        None => {
            // No nav data available — fall back to spiral
            fallback_spiral_tp(client, target_xyz).await;
        }
    }
}

/// Try to load and parse nav data for a zone. Returns None on failure.
///
/// Python: `load_wad(path)` + `wad.get_file("zone.nav")` + `parse_nav_data()`
fn load_and_parse_nav(zone_name: &str) -> Option<(Vec<XYZ>, Vec<(u16, u16)>)> {
    use wizwalker::file_readers::wad::Wad;

    let wad_path = zone_name.replace('/', "-");
    let mut wad = Wad::from_game_data(&wad_path).ok()?;
    let nav_data = wad.get_file("zone.nav").ok()?;
    Some(parse_nav_data(&nav_data))
}

/// Calculate chunk center points for entity scanning.
///
/// Returns a list of center points of "chunks" of the map, defined by input points.
///
/// Python: `calc_chunks(points, entity_distance)` — teleport_math.py:328
pub fn calc_chunks(points: &[XYZ], entity_distance: f32) -> Vec<XYZ> {
    if points.is_empty() {
        return Vec::new();
    }

    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;

    for p in points {
        if p.x < min_x { min_x = p.x; }
        if p.y < min_y { min_y = p.y; } // Note: Python has bug here (uses p.x for min_y)
        if p.x > max_x { max_x = p.x; }
        if p.y > max_y { max_y = p.y; }
    }

    // Inscribed square for chunking
    let side_length = (2.0f32).sqrt() * entity_distance;
    let half = side_length / 2.0;

    min_x += half;
    min_y += half;
    max_x -= half;
    max_y -= half;

    let mut current = XYZ {
        x: min_x - side_length,
        y: min_y,
        z: 0.0,
    };

    let mut chunk_points = Vec::new();

    loop {
        current.x += side_length;
        if current.x - half > max_x {
            current.x = min_x;
            current.y += side_length;
            if current.y - half > max_y {
                break;
            }
        }

        // Check if any points are in this square
        let has_points = points.iter().any(|p| {
            p.x >= current.x - half
                && p.x < current.x + half
                && p.y >= current.y - half
                && p.y < current.y + half
        });

        if has_points {
            chunk_points.push(XYZ {
                x: current.x,
                y: current.y,
                z: 0.0,
            });
        }
    }

    tracing::debug!("chunks: {}", chunk_points.len());
    chunk_points
}
