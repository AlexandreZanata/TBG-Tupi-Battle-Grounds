//! Frame selection from the normalized sheet's exact shared delay timeline.
use bevy::prelude::*;
use tbg_core::protocol::EmojiRef;
#[derive(Component)]
pub struct Sprite {
    pub reference: EmojiRef,
    pub fallback: u8,
    pub born: f64,
    pub private_asset: Option<i64>,
}
pub fn rect(reference: &EmojiRef, elapsed: f64) -> Option<Rect> {
    let EmojiRef::Custom {
        frame_px,
        frames,
        columns,
        delays_ms,
        ..
    } = reference
    else {
        return None;
    };
    if !reference.valid() {
        return None;
    }
    let total: u32 = delays_ms.iter().map(|v| u32::from(*v)).sum();
    let mut ms = (elapsed.max(0.) * 1000.) as u64 % u64::from(total.max(1));
    let mut frame = usize::from(*frames) - 1;
    for (i, delay) in delays_ms.iter().enumerate() {
        if ms < u64::from(*delay) {
            frame = i;
            break;
        }
        ms -= u64::from(*delay);
    }
    let px = f32::from(*frame_px);
    let col = frame % usize::from(*columns);
    let row = frame / usize::from(*columns);
    Some(Rect::from_corners(
        Vec2::new(col as f32 * px, row as f32 * px),
        Vec2::new((col + 1) as f32 * px, (row + 1) as f32 * px),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn p75_animation_uses_exact_delay_boundaries() {
        let reference = EmojiRef::Custom {
            id: [1; 16],
            frame_px: 128,
            frames: 2,
            columns: 2,
            delays_ms: vec![100, 200],
        };
        assert_eq!(rect(&reference, 0.).unwrap().min, Vec2::ZERO);
        assert_eq!(rect(&reference, 0.1).unwrap().min, Vec2::new(128., 0.));
        assert_eq!(rect(&reference, 0.3).unwrap().min, Vec2::ZERO);
    }
}
