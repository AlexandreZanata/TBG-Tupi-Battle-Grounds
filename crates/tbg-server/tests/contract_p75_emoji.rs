//! BR-CE01–10, IC-11/17 and S08/S11: normalized, approved-only custom emoji ownership.
#[path = "common/p75.rs"]
mod common;
#[path = "common/operator.rs"]
mod operator;
use common::*;
use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::io::Cursor;
fn png() -> Vec<u8> {
    png_variant(20)
}
fn png_variant(color: u8) -> Vec<u8> {
    let mut out = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(RgbaImage::from_pixel(8, 8, Rgba([color, 40, 80, 255])))
        .write_to(&mut out, ImageFormat::Png)
        .unwrap();
    out.into_inner()
}
#[test]
fn br_em01_standard_ids_are_permanent() {
    assert_eq!(
        tbg_core::protocol::STANDARD_EMOJI_ASSETS,
        [
            "1f44b", "1f44d", "1f600", "1f602", "1f60e", "1f914", "1f622", "1f621", "1f525",
            "2764", "1f389", "1f680", "1f480", "1f3c6", "1f64f", "1f4aa"
        ]
    );
}
#[test]
fn s08_hostile_images_are_bounded_and_never_kept_raw() {
    use tbg_server::emoji::normalize::*;
    let original = png();
    let normalized = normalize(&original, "image/png").unwrap();
    assert_ne!(normalized.bytes, original);
    assert_eq!(normalized.source_bytes, original.len());
    let mut polyglot = original.clone();
    polyglot.extend_from_slice(b"<script>alert(1)</script>");
    assert!(normalize(&polyglot, "image/png").is_err());
    assert!(normalize(&original[..original.len() / 2], "image/png").is_err());
    assert!(normalize(&original, "image/gif").is_err());
    assert!(normalize(&vec![0; 1048577], "image/png").is_err());
    let mut huge = original.clone();
    huge[16..20].copy_from_slice(&100000u32.to_be_bytes());
    assert!(normalize(&huge, "image/png").is_err());
    let mut gif = Vec::new();
    {
        let mut encoder = image::codecs::gif::GifEncoder::new(&mut gif);
        encoder
            .encode_frames((0..121).map(|_| {
                image::Frame::from_parts(
                    RgbaImage::from_pixel(1, 1, Rgba([255, 0, 0, 255])),
                    0,
                    0,
                    image::Delay::from_numer_denom_ms(0, 1),
                )
            }))
            .unwrap();
    }
    assert!(
        matches!(normalize(&gif, "image/gif"), Err(Error::Frames)),
        "stop before the 121st decoded frame"
    );
}
#[tokio::test]
async fn br_ce05_only_approved_sheets_are_public_and_hashes_stay_stable() {
    let s = operator::server().await;
    let op = operator::claim(&s).await;
    let g = s.register("Uploader", "upload@example.invalid").await;
    let access = g["access_token"].as_str();
    let code = s.mail("account_verify").await.code.unwrap();
    assert_eq!(
        s.request("POST", "/api/me/email/verify", json!({"code":code}), access)
            .await
            .status,
        200
    );
    assert_eq!(
        op.request(
            "PUT",
            "settings/emoji_uploads_enabled",
            json!({"value":1,"reason":"other"})
        )
        .await
        .status,
        200
    );
    let original = png();
    let uploaded = bytes_request(
        &s.url,
        "POST",
        "/api/me/emoji?label=hello",
        original.clone(),
        "image/png",
        access,
    )
    .await;
    assert_eq!(
        uploaded.status,
        201,
        "{}",
        String::from_utf8_lossy(&uploaded.bytes)
    );
    let id = serde_json::from_slice::<serde_json::Value>(&uploaded.bytes).unwrap()["id"]
        .as_i64()
        .unwrap();
    let preview = bytes_request(
        &s.url,
        "GET",
        &format!("/api/me/emoji/{id}/sheet.png"),
        vec![],
        "application/json",
        access,
    )
    .await;
    assert_eq!(preview.status, 200);
    assert_ne!(preview.bytes, original);
    assert_eq!(
        bytes_request(
            &s.url,
            "GET",
            "/api/emoji/00000000000000000000000000000000.png",
            vec![],
            "application/json",
            None
        )
        .await
        .status,
        404
    );
    assert_eq!(
        op.request(
            "POST",
            &format!("emoji/{id}/approve"),
            json!({"reason":"reviewed"})
        )
        .await
        .status,
        200
    );
    let db = rusqlite::Connection::open(&s.db).unwrap();
    let (public, hash): (String, Vec<u8>) = db
        .query_row(
            "SELECT lower(hex(public_id)),content_hash FROM emoji_assets WHERE id=?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    let served = bytes_request(
        &s.url,
        "GET",
        &format!("/api/emoji/{public}.png"),
        vec![],
        "application/json",
        None,
    )
    .await;
    assert_eq!(served.status, 200);
    assert_eq!(Sha256::digest(&served.bytes).as_slice(), hash);
    let equip = s
        .request(
            "PUT",
            "/api/me/emoji/slots",
            json!({"slots":[{"custom":public},{"standard":1},{"standard":2}]}),
            access,
        )
        .await;
    assert_eq!(equip.status, 200);
    let other = s.register("Other", "other@example.invalid").await;
    assert_eq!(
        s.request(
            "PUT",
            "/api/me/emoji/slots",
            json!({"slots":[{"custom":public},{"standard":1},{"standard":2}]}),
            other["access_token"].as_str()
        )
        .await
        .status,
        422,
        "CE09 owner only"
    );
    assert_eq!(
        op.request(
            "POST",
            &format!("emoji/{id}/revoke"),
            json!({"reason":"other"})
        )
        .await
        .status,
        200
    );
    assert_eq!(
        bytes_request(
            &s.url,
            "GET",
            &format!("/api/emoji/{public}.png"),
            vec![],
            "application/json",
            None
        )
        .await
        .status,
        404,
        "IC-17 even a cache hit is reauthorized"
    );
    assert_eq!(
        s.number("SELECT standard_id FROM account_emoji_slots WHERE account_id=1 AND slot=0"),
        0
    );
}

#[test]
fn s08_ten_thousand_frames_and_zero_delays_have_bounded_work() {
    use tbg_server::emoji::normalize::{normalize, Error};
    let mut bytes = Vec::new();
    {
        let mut encoder = image::codecs::gif::GifEncoder::new(&mut bytes);
        encoder
            .encode_frames((0..10000).map(|_| {
                image::Frame::from_parts(
                    RgbaImage::from_pixel(1, 1, Rgba([0, 0, 0, 255])),
                    0,
                    0,
                    image::Delay::from_numer_denom_ms(0, 1),
                )
            }))
            .unwrap();
    }
    assert!(matches!(normalize(&bytes, "image/gif"), Err(Error::Frames)));
    let mut bytes = Vec::new();
    {
        let mut encoder = image::codecs::gif::GifEncoder::new(&mut bytes);
        encoder
            .encode_frame(image::Frame::from_parts(
                RgbaImage::from_pixel(1, 1, Rgba([0, 0, 0, 255])),
                0,
                0,
                image::Delay::from_numer_denom_ms(0, 1),
            ))
            .unwrap();
    }
    let sheet = normalize(&bytes, "image/gif").unwrap();
    assert!(sheet.delays.iter().all(|ms| *ms >= 50));
    assert!(sheet.bytes.len() <= 262144);
}

#[tokio::test]
async fn s07_s11_s16_bad_labels_and_upload_floods_never_store_raw_input() {
    let s = operator::server().await;
    let op = operator::claim(&s).await;
    let grant = s.register("Uploader", "uploader@example.invalid").await;
    let access = grant["access_token"].as_str();
    let code = s.mail("account_verify").await.code.unwrap();
    assert_eq!(
        s.request("POST", "/api/me/email/verify", json!({"code":code}), access)
            .await
            .status,
        200
    );
    assert_eq!(
        op.request(
            "PUT",
            "settings/emoji_uploads_enabled",
            json!({"value":1,"reason":"other"})
        )
        .await
        .status,
        200
    );
    for label in ["bad%0D%0Aheader", "bad%E2%80%AE", "bad%E2%80%8B", ""] {
        assert_eq!(
            bytes_request(
                &s.url,
                "POST",
                &format!("/api/me/emoji?label={label}"),
                png(),
                "image/png",
                access
            )
            .await
            .status,
            422
        );
    }
    assert_eq!(s.number("SELECT COUNT(*) FROM emoji_assets"), 0);
    for color in 20..23 {
        assert_eq!(
            bytes_request(
                &s.url,
                "POST",
                "/api/me/emoji?label=valid",
                png_variant(color),
                "image/png",
                access
            )
            .await
            .status,
            201
        );
    }
    assert_eq!(
        bytes_request(
            &s.url,
            "POST",
            "/api/me/emoji?label=valid",
            png_variant(24),
            "image/png",
            access
        )
        .await
        .status,
        429
    );
    assert_eq!(s.number("SELECT COUNT(*) FROM emoji_assets"), 3);
}

#[tokio::test]
async fn s12_approval_and_revocation_each_have_one_effect_and_never_reuse_public_ids() {
    let s = operator::server().await;
    let op = operator::claim(&s).await;
    let g = s.register("Uploader", "upload@example.invalid").await;
    let access = g["access_token"].as_str();
    let code = s.mail("account_verify").await.code.unwrap();
    assert_eq!(
        s.request("POST", "/api/me/email/verify", json!({"code":code}), access)
            .await
            .status,
        200
    );
    assert_eq!(
        op.request(
            "PUT",
            "settings/emoji_uploads_enabled",
            json!({"value":1,"reason":"other"})
        )
        .await
        .status,
        200
    );
    assert_eq!(
        bytes_request(
            &s.url,
            "POST",
            "/api/me/emoji?label=valid",
            png(),
            "image/png",
            access
        )
        .await
        .status,
        201
    );
    assert_eq!(
        bytes_request(
            &s.url,
            "POST",
            "/api/me/emoji?label=duplicate",
            png(),
            "image/png",
            access
        )
        .await
        .status,
        422
    );
    let (one, two) = tokio::join!(
        op.request("POST", "emoji/1/approve", json!({"reason":"reviewed"})),
        op.request("POST", "emoji/1/approve", json!({"reason":"reviewed"}))
    );
    let mut status = [one.status, two.status];
    status.sort();
    assert_eq!(status, [200, 409]);
    let db = rusqlite::Connection::open(&s.db).unwrap();
    let public: Vec<u8> = db
        .query_row("SELECT public_id FROM emoji_assets", [], |r| r.get(0))
        .unwrap();
    let (one, two) = tokio::join!(
        op.request("POST", "emoji/1/revoke", json!({"reason":"other"})),
        op.request("POST", "emoji/1/revoke", json!({"reason":"other"}))
    );
    let mut status = [one.status, two.status];
    status.sort();
    assert_eq!(status, [200, 409]);
    assert_eq!(
        op.request("POST", "emoji/1/approve", json!({"reason":"reviewed"}))
            .await
            .status,
        409
    );
    assert_eq!(
        db.query_row("SELECT public_id FROM emoji_assets", [], |r| r
            .get::<_, Vec<u8>>(0))
            .unwrap(),
        public
    );
    assert_eq!(
        s.number("SELECT COUNT(*) FROM admin_audit WHERE target_kind='emoji' AND action='approve'"),
        1
    );
    assert_eq!(
        s.number("SELECT COUNT(*) FROM admin_audit WHERE target_kind='emoji' AND action='revoke'"),
        1
    );
}
