//! Contract: generated Augment art metadata mirrors the canonical catalog (BR-AG01–14).

use super::json_lite::parse;
use std::path::Path;
use tbg_core::content::{AUGMENTS, AUGMENT_WINDOW_TICKS, TICK_HZ};

#[test]
fn art_manifest_matches_every_canonical_augment_and_round_timing() {
    let data = parse(include_str!("../../assets/augments/augments.json"))
        .unwrap_or_else(|error| panic!("augments.json: {error}"));
    assert_eq!(data.get("tickHz").int(), Some(i64::from(TICK_HZ)));
    assert_eq!(
        data.get("roundTicks").int(),
        Some(i64::from(AUGMENT_WINDOW_TICKS))
    );
    let augments = data.get("augments").list();
    assert_eq!(augments.len(), AUGMENTS.len());
    for (index, (json, expected)) in augments.iter().zip(AUGMENTS).enumerate() {
        assert_eq!(usize::from(expected.id), index, "catalog index");
        assert_eq!(json.get("id").int(), Some(i64::from(expected.id)));
        assert_eq!(json.get("name").str(), Some(expected.name));
        assert_eq!(json.get("text").str(), Some(expected.text));
        let rarity = format!("{:?}", expected.rarity);
        assert_eq!(json.get("rarity").str(), Some(rarity.as_str()));
    }
}

#[test]
fn all_augment_illustrations_and_rarity_frames_are_shipped() {
    let data = parse(include_str!("../../assets/augments/augments.json"))
        .unwrap_or_else(|error| panic!("augments.json: {error}"));
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/augments");
    let illustrations = data.get("augments").list().iter().map(|a| a.get("asset"));
    let frames = data.get("rarities").list().iter().map(|r| r.get("frame"));
    assert_eq!(data.get("rarities").list().len(), 4);
    for asset in illustrations.chain(frames) {
        for key in ["svg", "display"] {
            let filename = asset.get(key).str().unwrap_or_default();
            assert!(
                !filename.is_empty() && !filename.contains('/') && !filename.contains(".."),
                "unsafe {key} asset: {filename}"
            );
            assert!(root.join(filename).is_file(), "missing {filename}");
        }
    }
}
