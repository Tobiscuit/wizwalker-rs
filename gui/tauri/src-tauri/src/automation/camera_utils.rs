//! Camera animation utilities — glide, orbit, point-to, player invisibility.
//!
//! Faithfully ported from `deimos-reference/src/camera_utils.py`.
#![allow(dead_code, unused_imports)]

use wizwalker::types::{XYZ, Orient};
use super::teleport_math::{calculate_yaw, calculate_pitch};

// Note: These functions take raw offset addresses and a reader because
// the CameraController struct methods aren't fully wired yet.
// When CameraController is complete, these will call its methods directly.

/// Point a freecam camera toward a given XYZ.
///
/// Python: `point_to_xyz(camera, xyz)` — camera_utils.py:11
pub fn point_to_xyz_orient(camera_pos: &XYZ, target: &XYZ) -> Orient {
    let yaw = calculate_yaw(camera_pos, target);
    let pitch = calculate_pitch(camera_pos, target);
    Orient {
        pitch,
        roll: 0.0,
        yaw,
    }
}

/// Calculate glide interpolation parameters from start to end over a given time.
///
/// Returns (velocity_xyz, orient_at_time) for use in a loop.
///
/// Python: `glide_to(camera, xyz_1, xyz_2, orientation, time, focus_xyz)` — camera_utils.py:40
pub struct GlideParams {
    pub velocity: XYZ,
    pub duration_secs: f32,
}

impl GlideParams {
    /// Create glide parameters from start/end over a given duration.
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

    /// Compute position at a given time offset.
    pub fn position_at(&self, start: &XYZ, dt: f32) -> XYZ {
        XYZ {
            x: start.x + self.velocity.x * dt,
            y: start.y + self.velocity.y * dt,
            z: start.z + self.velocity.z * dt,
        }
    }
}

/// Calculate rotation velocity for a rotating glide.
///
/// Python: `rotating_glide_to(camera, xyz_1, xyz_2, time, degrees)` — camera_utils.py:78
pub fn rotation_velocity(degrees: &Orient, duration_secs: f32) -> Orient {
    Orient {
        pitch: degrees.pitch.to_radians() / duration_secs,
        roll: degrees.roll.to_radians() / duration_secs,
        yaw: degrees.yaw.to_radians() / duration_secs,
    }
}

/// Calculate orbit position at a given angle around a center point.
///
/// Python: `orbit(camera, xyz_1, xyz_2, degrees, time)` — camera_utils.py:119
pub struct OrbitParams {
    pub center: XYZ,
    pub xy_radius: f32,
    pub start_angle: f32,
    pub angle_velocity: f32,
    pub base_z: f32,
}

impl OrbitParams {
    /// Create orbit parameters.
    ///
    /// - `camera_pos` — initial camera position
    /// - `center` — point to orbit around
    /// - `degrees` — total degrees to orbit
    /// - `duration_secs` — time to complete the orbit
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

    /// Calculate camera position at a given time offset.
    pub fn position_at(&self, dt: f32) -> XYZ {
        let angle = self.start_angle + self.angle_velocity * dt;
        XYZ {
            x: self.center.x - self.xy_radius * angle.cos(),
            y: self.center.y - self.xy_radius * angle.sin(),
            z: self.base_z,
        }
    }

    /// Calculate orientation to look at the center from a given position.
    pub fn orientation_at(&self, camera_pos: &XYZ, roll: f32) -> Orient {
        Orient {
            pitch: calculate_pitch(camera_pos, &self.center),
            roll,
            yaw: calculate_yaw(camera_pos, &self.center),
        }
    }
}
