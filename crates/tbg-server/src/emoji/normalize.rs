//! Bounded untrusted-image decoding into a canonical PNG animation sheet; originals are discarded.
use image::{AnimationDecoder, DynamicImage, ImageDecoder, ImageFormat, RgbaImage};
use sha2::{Digest, Sha256};
use std::{
    io::Cursor,
    time::{Duration, Instant},
};
pub struct Sheet {
    pub bytes: Vec<u8>,
    pub hash: [u8; 32],
    pub frames: u8,
    pub columns: u8,
    pub delays: Vec<u16>,
    pub source: &'static str,
    pub source_bytes: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Type,
    Size,
    Dimensions,
    Frames,
    Decode,
    Duration,
    EncodedSize,
}
impl Error {
    pub fn detail(self) -> &'static str {
        match self {
            Self::Type => "type",
            Self::Size => "size",
            Self::Dimensions => "dimensions",
            Self::Frames => "frames",
            Self::Decode => "decode",
            Self::Duration => "duration",
            Self::EncodedSize => "encoded_size",
        }
    }
}
fn limits() -> image::Limits {
    let mut l = image::Limits::default();
    l.max_image_width = Some(1024);
    l.max_image_height = Some(1024);
    l.max_alloc = Some(16 * 1024 * 1024);
    l
}
fn square(image: RgbaImage) -> RgbaImage {
    let thumb = image::imageops::thumbnail(&image, 128, 128);
    let mut square = RgbaImage::new(128, 128);
    image::imageops::overlay(
        &mut square,
        &thumb,
        i64::from((128 - thumb.width()) / 2),
        i64::from((128 - thumb.height()) / 2),
    );
    square
}
pub fn normalize(bytes: &[u8], mime: &str) -> Result<Sheet, Error> {
    if bytes.is_empty() || bytes.len() > 1048576 {
        return Err(Error::Size);
    }
    let (format, source) = match mime {
        "image/png" => (ImageFormat::Png, "png"),
        "image/gif" => (ImageFormat::Gif, "gif"),
        "image/webp" => (ImageFormat::WebP, "webp"),
        _ => return Err(Error::Type),
    };
    if image::guess_format(bytes).map_err(|_| Error::Decode)? != format {
        return Err(Error::Type);
    }
    // Reject trailing payloads: the only served bytes are newly encoded pixels.
    match format {
        ImageFormat::Png => {
            if !bytes.ends_with(b"\0\0\0\0IEND\xaeB`\x82") {
                return Err(Error::Decode);
            }
        }
        ImageFormat::Gif => {
            if bytes.last() != Some(&0x3b) {
                return Err(Error::Decode);
            }
        }
        ImageFormat::WebP => {
            let len = bytes
                .get(4..8)
                .and_then(|v| v.try_into().ok())
                .map(u32::from_le_bytes)
                .ok_or(Error::Decode)?;
            if u64::from(len) + 8 != bytes.len() as u64 {
                return Err(Error::Decode);
            }
        }
        _ => {}
    }
    let mut reader = image::ImageReader::with_format(Cursor::new(bytes), format);
    reader.limits(limits());
    let (w, h) = reader.into_dimensions().map_err(|_| Error::Dimensions)?;
    if w == 0 || h == 0 || w > 1024 || h > 1024 {
        return Err(Error::Dimensions);
    }
    let source_bytes = bytes.len();
    let started = Instant::now();
    let mut frames: Vec<(RgbaImage, u32)> = Vec::new();
    if format == ImageFormat::Gif {
        let mut decoder =
            image::codecs::gif::GifDecoder::new(Cursor::new(bytes)).map_err(|_| Error::Decode)?;
        decoder
            .set_limits(limits())
            .map_err(|_| Error::Dimensions)?;
        for (index, frame) in decoder.into_frames().enumerate() {
            if index >= 120 {
                return Err(Error::Frames);
            }
            if started.elapsed() > Duration::from_secs(2) {
                return Err(Error::Duration);
            }
            let frame = frame.map_err(|_| Error::Decode)?;
            let (n, d) = frame.delay().numer_denom_ms();
            let delay = (n / d.max(1)).clamp(50, 1000);
            frames.push((square(frame.into_buffer()), delay));
        }
    } else {
        if format == ImageFormat::WebP
            && image::codecs::webp::WebPDecoder::new(Cursor::new(bytes))
                .map_err(|_| Error::Decode)?
                .has_animation()
        {
            return Err(Error::Type);
        }
        if format == ImageFormat::Png
            && image::codecs::png::PngDecoder::new(Cursor::new(bytes))
                .map_err(|_| Error::Decode)?
                .is_apng()
                .map_err(|_| Error::Decode)?
        {
            return Err(Error::Type);
        }
        let mut reader = image::ImageReader::with_format(Cursor::new(bytes), format);
        reader.limits(limits());
        frames.push((
            square(reader.decode().map_err(|_| Error::Decode)?.into_rgba8()),
            1000,
        ));
    }
    if frames.is_empty() {
        return Err(Error::Decode);
    }
    let duration = frames.iter().map(|(_, d)| *d).sum::<u32>().min(4000);
    let count = frames.len().min(24).min((duration / 50).max(1) as usize);
    let columns = count.min(6);
    let rows = count.div_ceil(columns);
    let mut sheet = RgbaImage::new(columns as u32 * 128, rows as u32 * 128);
    let mut delays = Vec::with_capacity(count);
    for i in 0..count {
        let time = (i as u32 * duration) / count as u32;
        let next = ((i + 1) as u32 * duration) / count as u32;
        let mut elapsed = 0;
        let mut frame = &frames[0].0;
        for (candidate, delay) in &frames {
            frame = candidate;
            if elapsed + delay > time {
                break;
            }
            elapsed += delay;
        }
        image::imageops::overlay(
            &mut sheet,
            frame,
            (i % columns) as i64 * 128,
            (i / columns) as i64 * 128,
        );
        delays.push((next - time).clamp(50, 1000) as u16);
    }
    let mut output = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(sheet)
        .write_to(&mut output, ImageFormat::Png)
        .map_err(|_| Error::Decode)?;
    let bytes = output.into_inner();
    if bytes.len() > 262144 {
        return Err(Error::EncodedSize);
    }
    let hash = Sha256::digest(&bytes).into();
    Ok(Sheet {
        bytes,
        hash,
        frames: count as u8,
        columns: columns as u8,
        delays,
        source,
        source_bytes,
    })
}
