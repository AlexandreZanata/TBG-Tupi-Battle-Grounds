//! Walk parameters per body: stride, foot lift, hip bob and knee sink, and
//! whether footfalls stomp dust. Cadence itself comes from the distance a
//! unit travels (`anim.rs`), so these only shape the step.

use tbg_core::UnitKind;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gait {
    /// Ground covered per full cycle (two steps), in leg lengths; capped by
    /// `reach`.
    pub stride: f32,
    /// Peak swing-foot lift, in leg lengths.
    pub lift: f32,
    /// Hip bob amplitude, in leg lengths (twice per cycle).
    pub bob: f32,
    /// How far the hips drop so the knees flex while walking, in leg lengths.
    pub sink: f32,
    /// Heavy footfalls raise dust at every contact (Large bodies).
    pub stomp: bool,
}

impl Gait {
    pub const LIGHT: Gait = Gait {
        stride: 1.15,
        lift: 0.22,
        bob: 0.02,
        sink: 0.09,
        stomp: false,
    };
    pub const HEAVY: Gait = Gait {
        stride: 1.1,
        lift: 0.16,
        bob: 0.035,
        sink: 0.1,
        stomp: true,
    };

    /// The kit's gait, else light or heavy by body plan.
    pub fn of(kind: UnitKind) -> Gait {
        super::kit(kind).map_or_else(
            || {
                if crate::art::roster::large(kind) {
                    Gait::HEAVY
                } else {
                    Gait::LIGHT
                }
            },
            |k| k.gait,
        )
    }

    /// Longest stride (leg lengths per cycle) the legs reach with the hips
    /// at the top of their bob (`feet.rs`: sink - bob / 2 below rest).
    pub fn reach(&self) -> f32 {
        let drop = self.sink - self.bob * 0.5;
        4.0 * (2.0 * drop - drop * drop).max(0.0).sqrt()
    }

    /// Ground one full cycle covers for a leg of `leg` (figure units).
    pub fn cycle(&self, leg: f32) -> f32 {
        leg * self.stride.min(self.reach())
    }
}
