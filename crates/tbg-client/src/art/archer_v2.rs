//! Default Archer proportions and bow lengths for base, Longbow, and Marksman.

use super::rig::{BodyPlan, Proportions};

/// How far the string travels from rest to the anchor at the cheek (model units).
pub const DRAW: f32 = 0.28;
/// Distance from the grip back to the resting string line.
pub const STRING_LINE: f32 = 0.23;

/// Humanoid anatomy with longer arms: a side-on stance needs the drawing hand
/// to reach the cheek across shoulders that are 0.46 wide.
pub fn plan() -> Proportions {
    let mut plan = BodyPlan::Humanoid.proportions();
    plan.arm = 0.20;
    plan
}

/// Half the string length (tip to tip / 2) for base, Longbow, and Marksman.
pub fn bow_half(tier: u8) -> f32 {
    match tier {
        0 => 0.64,
        1 => 0.74,
        _ => 0.68,
    }
}
