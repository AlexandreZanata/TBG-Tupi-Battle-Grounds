//! PNG header validation before allocating a runtime custom emoji image.
pub fn dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() > 262144
        || bytes.len() < 24
        || bytes.get(..8) != Some(b"\x89PNG\r\n\x1a\n")
        || bytes.get(12..16) != Some(b"IHDR")
    {
        return None;
    }
    let w = u32::from_be_bytes(bytes.get(16..20)?.try_into().ok()?);
    let h = u32::from_be_bytes(bytes.get(20..24)?.try_into().ok()?);
    (w > 0 && h > 0 && w <= 768 && h <= 512).then_some((w, h))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn p75_header_rejects_overflow_before_decode() {
        let mut b = vec![0; 24];
        b[..8].copy_from_slice(b"\x89PNG\r\n\x1a\n");
        b[12..16].copy_from_slice(b"IHDR");
        b[16..20].copy_from_slice(&769u32.to_be_bytes());
        b[20..24].copy_from_slice(&128u32.to_be_bytes());
        assert!(dimensions(&b).is_none());
        b[16..20].copy_from_slice(&768u32.to_be_bytes());
        assert_eq!(dimensions(&b), Some((768, 128)));
    }
}
