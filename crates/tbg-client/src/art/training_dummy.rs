//! Cached wooden/straw target artwork; its identity remains local training metadata.

use super::{
    figure::{part, FigureSpec},
    mesh::Shape::*,
    rig::{BodyPlan, Joint, Node},
};
use bevy::prelude::*;

pub fn spec() -> FigureSpec {
    let root = Node::Body(Joint::Root);
    let wood = Color::srgb_u8(108, 70, 40);
    let straw = Color::srgb_u8(207, 173, 100);
    FigureSpec {
        plan: BodyPlan::Humanoid.proportions(),
        parts: vec![
            part(root, Box(0.62, 0.09, 0.48), wood, [0., 0.045, 0.]),
            part(root, Box(0.14, 1.45, 0.14), wood, [0., 0.76, 0.]),
            part(root, Box(1.02, 0.13, 0.15), wood, [0., 1.03, 0.]),
            part(root, Cyl(0.24, 0.50), straw, [0., 1.00, 0.]),
            part(root, Ball(0.20), straw, [0., 1.43, 0.]),
            part(root, Box(0.43, 0.05, 0.44), wood, [0., 1.10, 0.]),
            part(root, Box(0.07, 0.07, 0.02), wood, [-0.07, 1.46, 0.19]),
            part(root, Box(0.07, 0.07, 0.02), wood, [0.07, 1.46, 0.19]),
        ],
    }
}
