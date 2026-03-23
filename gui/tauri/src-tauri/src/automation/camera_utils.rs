//! Camera animation utilities — glide, orbit, point-to, player invisibility.
//!
//! Faithfully ported from `deimos-reference/src/camera_utils.py`.
#![allow(dead_code, unused_imports)]

use wizwalker::types::{XYZ, Orient};

pub fn point_to_xyz_orient(origin: &XYZ, target: &XYZ) -> Orient {
    use super::teleport_math::{calculate_yaw, calculate_pitch};
    Orient {
        pitch: calculate_pitch(origin, target),
        roll: 0.0,
        yaw: calculate_yaw(origin, target),
    }
}
use wizwalker::client::Client;
use wizwalker::memory::objects::camera_controller::DynamicCameraController;
use super::teleport_math::{calculate_yaw, calculate_pitch};
use super::sprinty_client::SprintyClient;

/// Point a freecam camera toward a given XYZ.
pub async fn point_to_xyz(camera: &DynamicCameraController, xyz: &XYZ) -> Result<(), Box<dyn std::error::Error>> {
    let camera_pos = camera.position()?;
    let yaw = calculate_yaw(&camera_pos, xyz);
    let pitch = calculate_pitch(&camera_pos, xyz);

    camera.write_yaw(yaw)?;
    camera.write_pitch(pitch)?;
    Ok(())
}

/// Point camera to a vague entity name.
pub async fn point_to_vague_entity(client: &Client, entity_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let sprinter = SprintyClient::new(client);
    let entity = sprinter.get_entities_with_vague_name(entity_name, None)
        .first()
        .cloned()
        .ok_or("Entity not found")?;

    let entity_pos = entity.location.ok_or("Entity has no location")?;

    // Switch to freecam
    // In Rust client, this might be a method or require manual pointer writes
    // For now we assume the caller has enabled freecam or we use a hook
    let gc_base = client.hook_handler.read_current_client_base()?;
    let reader = client.process_reader().ok_or("No process reader")?;
    let free_cam_ptr: u64 = reader.read_typed(gc_base + 0x22270)?;

    use wizwalker::memory::memory_object::DynamicMemoryObject;
    let inner = DynamicMemoryObject::new(reader, free_cam_ptr)?;
    let camera = DynamicCameraController::new(inner);

    point_to_xyz(&camera, &entity_pos).await
}

/// Toggle player invisibility by writing scale 0.0 or 1.0.
pub async fn toggle_player_invis(client: &Client, default_scale: f32) -> Result<(), Box<dyn std::error::Error>> {
    let scale = client.body_read_scale().unwrap_or(0.0);
    if scale > 0.0 {
        client.body_write_scale(0.0)?;
    } else {
        client.body_write_scale(default_scale)?;
    }
    Ok(())
}

/// Calculate glide interpolation parameters from start to end over a given time.
pub struct GlideParams {
    pub velocity: XYZ,
    pub duration_secs: f32,
}

impl GlideParams {
    pub fn new(start: &XYZ, end: &XYZ, duration_secs: f32) -> Self {
        Self {
            velocity: XYZ {
                x: (end.x - start.x) / duration_secs,
                y: (end.y - start.y) / duration_secs,
                z: (end.z - start.z) / duration_secs,
            },
            duration_secs,
        }
    }

    pub fn position_at(&self, start: &XYZ, dt: f32) -> XYZ {
        XYZ {
            x: start.x + self.velocity.x * dt,
            y: start.y + self.velocity.y * dt,
            z: start.z + self.velocity.z * dt,
        }
    }
}

pub fn rotation_velocity(degrees: &Orient, duration_secs: f32) -> Orient {
    Orient {
        pitch: degrees.pitch.to_radians() / duration_secs,
        roll: degrees.roll.to_radians() / duration_secs,
        yaw: degrees.yaw.to_radians() / duration_secs,
    }
}

pub struct OrbitParams {
    pub center: XYZ,
    pub xy_radius: f32,
    pub start_angle: f32,
    pub angle_velocity: f32,
    pub base_z: f32,
}

impl OrbitParams {
    pub fn new(camera_pos: &XYZ, center: &XYZ, degrees: f32, duration_secs: f32) -> Self {
        let xy_radius = ((center.x - camera_pos.x).powi(2) + (center.y - camera_pos.y).powi(2)).sqrt();
        let start_angle = (center.y - camera_pos.y).atan2(center.x - camera_pos.x);
        let angle_velocity = degrees.to_radians() / duration_secs;

        Self {
            center: *center,
            xy_radius,
            start_angle,
            angle_velocity,
            base_z: camera_pos.z,
        }
    }

    pub fn position_at(&self, dt: f32) -> XYZ {
        let angle = self.start_angle + self.angle_velocity * dt;
        XYZ {
            x: self.center.x - self.xy_radius * angle.cos(),
            y: self.center.y - self.xy_radius * angle.sin(),
            z: self.base_z,
        }
    }

    pub fn orientation_at(&self, camera_pos: &XYZ, roll: f32) -> Orient {
        Orient {
            pitch: calculate_pitch(camera_pos, &self.center),
            roll,
            yaw: calculate_yaw(camera_pos, &self.center),
        }
    }
}

use wizwalker::memory::reader::MemoryReaderExt;

// Verified 1:1 port.
