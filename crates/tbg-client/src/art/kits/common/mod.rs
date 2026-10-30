//! Shared building blocks for kit champions: the rig-v2 body and dress
//! (`body`), weapons and props (`gear`), and archetype motion blocks
//! (`moves`), so every kit shares one style and only adds its identity.

pub mod body;
pub mod gear;
pub mod moves;

/// Which Evolution pieces a form has.
#[derive(Clone, Copy, Debug)]
pub struct Form {
    pub a: bool,
    pub b: bool,
    pub t2: bool,
}

impl Form {
    pub fn of(tier: u8, branch: u8) -> Form {
        Form {
            a: tier >= 1 && branch == 0,
            b: tier >= 1 && branch == 1,
            t2: tier >= 2,
        }
    }
}
