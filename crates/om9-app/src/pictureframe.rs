// SPDX-License-Identifier: LGPL-2.1-or-later
//! Picture-frame reference images: immutable raster assets, an aspect-locked quad on a CPlane,
//! opacity/lock policy, known-distance calibration and embedded persistence.
//!
//! This is reference geometry, not a render engine: import validates and decodes the raster once,
//! every later operation is presentation state.
use om9_core::{Error, Result, geometry::Point3};
use om9_scene::camera::CPlane;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const PICTUREFRAME_RESOURCE: &str = "om9.pictureframe.v1";
const MAX_ASSET_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Clone, PartialEq, Eq)]
pub struct RasterAsset {
    pub name: String,
    pub width_px: u32,
    pub height_px: u32,
    pub bytes: Vec<u8>,
    pub sha256: [u8; 32],
}

impl std::fmt::Debug for RasterAsset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RasterAsset")
            .field("name", &self.name)
            .field("width_px", &self.width_px)
            .field("height_px", &self.height_px)
            .field("bytes", &self.bytes.len())
            .finish()
    }
}

impl RasterAsset {
    /// Decode a PNG or JPEG. The format is detected from the encoded bytes; the source bytes
    /// remain immutable so persistence can hash and embed exactly what was imported.
    pub fn from_encoded(name: impl Into<String>, bytes: Vec<u8>) -> Result<Self> {
        if bytes.is_empty() {
            return Err(Error::Missing("picture frame image"));
        }
        if bytes.len() as u64 > MAX_ASSET_BYTES {
            return Err(Error::Budget("picture frame image"));
        }
        let decoded = image::load_from_memory(&bytes)
            .map_err(|_| Error::Invalid("picture frame image decode"))?;
        if decoded.width() == 0 || decoded.height() == 0 {
            return Err(Error::Invalid("picture frame image size"));
        }
        Ok(Self {
            name: name.into(),
            width_px: decoded.width(),
            height_px: decoded.height(),
            sha256: digest(&bytes),
            bytes,
        })
    }

    pub fn from_png(name: impl Into<String>, bytes: Vec<u8>) -> Result<Self> {
        Self::from_encoded(name, bytes)
    }

    pub fn from_jpeg(name: impl Into<String>, bytes: Vec<u8>) -> Result<Self> {
        Self::from_encoded(name, bytes)
    }

    pub fn sha_hex(&self) -> String {
        hex(&self.sha256)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PictureFrame {
    pub asset: RasterAsset,
    pub external_path: Option<String>,
    pub origin: Point3,
    pub x_axis: Point3,
    pub y_axis: Point3,
    pub width_mm: f64,
    pub height_mm: f64,
    pub opacity: f64,
    pub locked: bool,
    pub visible: bool,
}

impl PictureFrame {
    /// Import a decoded asset and build a quad on `plane`; the source aspect ratio is preserved.
    pub fn import_png(
        asset: RasterAsset,
        plane: &CPlane,
        origin: Point3,
        width_mm: f64,
    ) -> Result<Self> {
        if !width_mm.is_finite() || width_mm <= 0. {
            return Err(Error::Invalid("picture frame width"));
        }
        let height_mm = width_mm * f64::from(asset.height_px) / f64::from(asset.width_px);
        if !height_mm.is_finite() || height_mm <= 0. {
            return Err(Error::Invalid("picture frame height"));
        }
        let frame = Self {
            asset,
            external_path: None,
            origin,
            x_axis: plane.x,
            y_axis: plane.y,
            width_mm,
            height_mm,
            opacity: 1.,
            locked: false,
            visible: true,
        };
        frame.validate()?;
        Ok(frame)
    }

    pub fn validate(&self) -> Result<()> {
        if self.asset.width_px == 0 || self.asset.height_px == 0 {
            return Err(Error::Invalid("picture frame asset"));
        }
        for point in [self.origin, self.x_axis, self.y_axis] {
            if !point.iter().all(|value| value.is_finite()) {
                return Err(Error::Invalid("picture frame point"));
            }
        }
        if !self.width_mm.is_finite()
            || !self.height_mm.is_finite()
            || self.width_mm <= 0.
            || self.height_mm <= 0.
        {
            return Err(Error::Invalid("picture frame size"));
        }
        if !self.opacity.is_finite() || !(0. ..=1.).contains(&self.opacity) {
            return Err(Error::Invalid("picture frame opacity"));
        }
        // Aspect ratio is an invariant, not a mutable suggestion.
        let expected =
            self.width_mm * f64::from(self.asset.height_px) / f64::from(self.asset.width_px);
        if (expected - self.height_mm).abs() > 1e-9 * self.height_mm.max(1.) {
            return Err(Error::Invalid("picture frame aspect"));
        }
        Ok(())
    }

    pub fn corners(&self) -> [Point3; 4] {
        let u = scale(self.x_axis, self.width_mm);
        let v = scale(self.y_axis, self.height_mm);
        [
            self.origin,
            add(self.origin, u),
            add(add(self.origin, u), v),
            add(self.origin, v),
        ]
    }

    pub fn bounds(&self) -> (Point3, Point3) {
        let corners = self.corners();
        let mut min = [f64::INFINITY; 3];
        let mut max = [f64::NEG_INFINITY; 3];
        for point in corners {
            for axis in 0..3 {
                min[axis] = min[axis].min(point[axis]);
                max[axis] = max[axis].max(point[axis]);
            }
        }
        (min, max)
    }

    pub fn translate(&mut self, delta: Point3) -> Result<()> {
        if self.locked {
            return Err(Error::Permission("picture frame locked"));
        }
        if !delta.iter().all(|value| value.is_finite()) {
            return Err(Error::Invalid("picture frame translation"));
        }
        let origin = add(self.origin, delta);
        let old = self.origin;
        self.origin = origin;
        if let Err(error) = self.validate() {
            self.origin = old;
            return Err(error);
        }
        Ok(())
    }

    /// Change width while preserving the source aspect ratio.
    pub fn set_width(&mut self, width_mm: f64) -> Result<()> {
        if self.locked {
            return Err(Error::Permission("picture frame locked"));
        }
        if !width_mm.is_finite() || width_mm <= 0. {
            return Err(Error::Invalid("picture frame width"));
        }
        let height = width_mm * f64::from(self.asset.height_px) / f64::from(self.asset.width_px);
        let old = (self.width_mm, self.height_mm);
        self.width_mm = width_mm;
        self.height_mm = height;
        if let Err(error) = self.validate() {
            self.width_mm = old.0;
            self.height_mm = old.1;
            return Err(error);
        }
        Ok(())
    }

    pub fn set_opacity(&mut self, opacity: f64) -> Result<()> {
        if self.locked {
            return Err(Error::Permission("picture frame locked"));
        }
        if !opacity.is_finite() || !(0. ..=1.).contains(&opacity) {
            return Err(Error::Invalid("picture frame opacity"));
        }
        self.opacity = opacity;
        Ok(())
    }

    pub fn set_locked(&mut self, locked: bool) {
        self.locked = locked;
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Locked or hidden references are visible but never selectable.
    pub fn selectable(&self) -> bool {
        self.visible && !self.locked
    }

    pub fn mm_per_pixel(&self) -> f64 {
        self.width_mm / f64::from(self.asset.width_px)
    }

    /// Known-distance calibration: the quad is resized uniformly so the measured pixel span
    /// equals `target_mm`. The asset bytes and hash are untouched.
    pub fn calibrate(&mut self, pixel_distance: f64, target_mm: f64) -> Result<()> {
        if self.locked {
            return Err(Error::Permission("picture frame locked"));
        }
        if !pixel_distance.is_finite() || pixel_distance <= 0. {
            return Err(Error::Invalid("picture frame pixel distance"));
        }
        if !target_mm.is_finite() || target_mm <= 0. {
            return Err(Error::Invalid("picture frame target length"));
        }
        let mm_per_pixel = target_mm / pixel_distance;
        let width = mm_per_pixel * f64::from(self.asset.width_px);
        if !width.is_finite() || width <= 0. {
            return Err(Error::Invalid("picture frame calibrated width"));
        }
        self.set_width(width)
    }

    pub fn to_record(&self, embed_bitmap: bool) -> PictureFrameRecord {
        PictureFrameRecord {
            name: self.asset.name.clone(),
            width_px: self.asset.width_px,
            height_px: self.asset.height_px,
            sha256: self.asset.sha_hex(),
            bitmap_base64: embed_bitmap.then(|| base64_encode(&self.asset.bytes)),
            external_path: self.external_path.clone(),
            origin: self.origin,
            x_axis: self.x_axis,
            y_axis: self.y_axis,
            width_mm: self.width_mm,
            height_mm: self.height_mm,
            opacity: self.opacity,
            locked: self.locked,
            visible: self.visible,
        }
    }

    pub fn from_record(record: &PictureFrameRecord) -> Result<(Self, Vec<String>)> {
        let mut warnings = Vec::new();
        let bytes = match &record.bitmap_base64 {
            Some(encoded) => {
                base64_decode(encoded).ok_or(Error::Invalid("picture frame base64"))?
            }
            None => {
                warnings.push(format!("picture frame image is missing: {}", record.name));
                Vec::new()
            }
        };
        if !bytes.is_empty() {
            let actual = hex(&digest(&bytes));
            if actual != record.sha256 {
                return Err(Error::Invalid("picture frame asset hash"));
            }
        }
        let asset = if bytes.is_empty() {
            RasterAsset {
                name: record.name.clone(),
                width_px: record.width_px,
                height_px: record.height_px,
                bytes,
                sha256: parse_hex(&record.sha256).ok_or(Error::Invalid("picture frame hash"))?,
            }
        } else {
            RasterAsset::from_encoded(record.name.clone(), bytes)?
        };
        let frame = Self {
            asset,
            external_path: record.external_path.clone(),
            origin: record.origin,
            x_axis: record.x_axis,
            y_axis: record.y_axis,
            width_mm: record.width_mm,
            height_mm: record.height_mm,
            opacity: record.opacity,
            locked: record.locked,
            visible: record.visible,
        };
        frame.validate()?;
        Ok((frame, warnings))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PictureFrameRecord {
    pub name: String,
    pub width_px: u32,
    pub height_px: u32,
    pub sha256: String,
    pub bitmap_base64: Option<String>,
    pub external_path: Option<String>,
    pub origin: Point3,
    pub x_axis: Point3,
    pub y_axis: Point3,
    pub width_mm: f64,
    pub height_mm: f64,
    pub opacity: f64,
    pub locked: bool,
    pub visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PictureFrameStore {
    pub version: u32,
    pub frames: Vec<PictureFrameRecord>,
}

impl Default for PictureFrameStore {
    fn default() -> Self {
        Self {
            version: 1,
            frames: Vec::new(),
        }
    }
}

impl PictureFrameStore {
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|_| Error::Invalid("picture frame store json"))
    }

    /// Load a project manifest atomically: a schema or hash failure blocks the whole load;
    /// only a genuinely absent external image becomes a warning and a retained placeholder.
    pub fn from_json(text: &str) -> Result<(Vec<PictureFrame>, Vec<String>)> {
        let store: Self =
            serde_json::from_str(text).map_err(|_| Error::Invalid("picture frame store schema"))?;
        if store.version != 1 {
            return Err(Error::Invalid("picture frame store version"));
        }
        let mut frames = Vec::with_capacity(store.frames.len());
        let mut warnings = Vec::new();
        for record in &store.frames {
            let (frame, mut frame_warnings) = PictureFrame::from_record(record)?;
            warnings.append(&mut frame_warnings);
            frames.push(frame);
        }
        Ok((frames, warnings))
    }
}

fn digest(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(out, "{byte:02x}");
    }
    out
}

fn parse_hex(text: &str) -> Option<[u8; 32]> {
    if text.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    for (index, chunk) in text.as_bytes().as_chunks::<2>().0.iter().enumerate() {
        let high = hex_nibble(chunk[0])?;
        let low = hex_nibble(chunk[1])?;
        out[index] = high << 4 | low;
    }
    Some(out)
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let a = chunk[0];
        let b = chunk.get(1).copied().unwrap_or(0);
        let c = chunk.get(2).copied().unwrap_or(0);
        out.push(BASE64[(a >> 2) as usize] as char);
        out.push(BASE64[((a & 0x03) << 4 | b >> 4) as usize] as char);
        out.push(if chunk.len() > 1 {
            BASE64[((b & 0x0f) << 2 | c >> 6) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            BASE64[(c & 0x3f) as usize] as char
        } else {
            '='
        });
    }
    out
}

fn base64_decode(text: &str) -> Option<Vec<u8>> {
    let bytes = text.as_bytes();
    if !bytes.len().is_multiple_of(4) {
        return None;
    }
    let mut out = Vec::with_capacity(bytes.len() / 4 * 3);
    for chunk in bytes.as_chunks::<4>().0 {
        let value = |byte: u8| -> Option<u8> {
            if byte == b'=' {
                return Some(0);
            }
            BASE64
                .iter()
                .position(|candidate| *candidate == byte)
                .map(|index| index as u8)
        };
        let a = value(chunk[0])?;
        let b = value(chunk[1])?;
        let c = value(chunk[2])?;
        let d = value(chunk[3])?;
        out.push(a << 2 | b >> 4);
        if chunk[2] != b'=' {
            out.push(b << 4 | c >> 2);
        }
        if chunk[3] != b'=' {
            out.push(c << 6 | d);
        }
    }
    Some(out)
}

fn add(a: Point3, b: Point3) -> Point3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale(a: Point3, factor: f64) -> Point3 {
    [a[0] * factor, a[1] * factor, a[2] * factor]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn png(width: u32, height: u32) -> Vec<u8> {
        let image = image::RgbaImage::from_pixel(width, height, image::Rgba([20, 40, 60, 255]));
        let mut cursor = Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(image)
            .write_to(&mut cursor, image::ImageFormat::Png)
            .unwrap();
        cursor.into_inner()
    }

    fn frame(width: f64) -> PictureFrame {
        let asset = RasterAsset::from_png("reference.png", png(100, 100)).unwrap();
        PictureFrame::import_png(asset, &CPlane::world_xy(), [0.; 3], width).unwrap()
    }

    #[test]
    fn a_jpeg_gives_a_twenty_millimetre_quad_and_is_persisted() {
        let image = image::RgbImage::from_pixel(100, 100, image::Rgb([12, 34, 56]));
        let mut cursor = Cursor::new(Vec::new());
        image::DynamicImage::ImageRgb8(image)
            .write_to(&mut cursor, image::ImageFormat::Jpeg)
            .unwrap();
        let asset = RasterAsset::from_jpeg("reference.jpg", cursor.into_inner()).unwrap();
        let mut frame = PictureFrame::import_png(asset, &CPlane::world_xy(), [0.; 3], 20.).unwrap();
        assert_eq!(frame.asset.width_px, 100);
        assert_eq!(frame.asset.height_px, 100);
        frame.set_opacity(0.5).unwrap();
        let store = PictureFrameStore {
            version: 1,
            frames: vec![frame.to_record(true)],
        };
        let (loaded, warnings) = PictureFrameStore::from_json(&store.to_json().unwrap()).unwrap();
        assert!(warnings.is_empty());
        assert_eq!(loaded[0].opacity, 0.5);
        assert_eq!(loaded[0].asset.sha_hex(), frame.asset.sha_hex());
    }

    #[test]
    fn a_png_gives_a_twenty_millimetre_quad_and_rejects_bad_bytes() {
        let frame = frame(20.);
        assert_eq!(frame.width_mm, 20.);
        assert_eq!(frame.height_mm, 20.);
        assert_eq!(frame.asset.width_px, 100);
        assert_eq!(frame.asset.height_px, 100);
        assert_eq!(frame.corners()[2], [20., 20., 0.]);
        assert!(RasterAsset::from_png("bad.png", b"not an image".to_vec()).is_err());
    }

    #[test]
    fn move_and_scale_keep_the_aspect_ratio_and_refuse_bad_sizes() {
        let mut frame = frame(20.);
        frame.translate([5., 6., 0.]).unwrap();
        assert_eq!(frame.origin, [5., 6., 0.]);
        frame.set_width(40.).unwrap();
        assert_eq!(frame.height_mm, 40.);
        assert!((frame.bounds().1[0] - 45.).abs() < 1e-9);
        assert!(frame.set_width(0.).is_err());
        assert!(frame.set_width(-1.).is_err());
        assert_eq!(frame.width_mm, 40.);
    }

    #[test]
    fn opacity_lock_and_visibility_preserve_the_asset() {
        let mut frame = frame(20.);
        let hash = frame.asset.sha_hex();
        frame.set_opacity(0.5).unwrap();
        frame.set_locked(true);
        assert!(!frame.selectable());
        assert!(frame.set_opacity(0.2).is_err());
        frame.set_visible(false);
        assert!(!frame.selectable());
        frame.set_locked(false);
        frame.set_visible(true);
        assert!(frame.selectable());
        assert_eq!(frame.asset.sha_hex(), hash);
    }

    #[test]
    fn known_distance_calibration_maps_100_pixels_to_25_mm() {
        let mut frame = frame(20.);
        frame.calibrate(100., 25.).unwrap();
        assert!((frame.mm_per_pixel() - 0.25).abs() < 1e-12);
        assert!((frame.width_mm - 25.).abs() < 1e-12);
        assert!((frame.height_mm - 25.).abs() < 1e-12);
        assert!(frame.calibrate(0., 25.).is_err());
        assert!(frame.calibrate(100., 0.).is_err());
        assert_eq!(frame.mm_per_pixel(), 0.25);
    }

    #[test]
    fn an_embedded_store_round_trips_and_a_missing_or_mismatched_asset_is_explicit() {
        let frame = frame(20.);
        let store = PictureFrameStore {
            version: 1,
            frames: vec![frame.to_record(true)],
        };
        let (loaded, warnings) = PictureFrameStore::from_json(&store.to_json().unwrap()).unwrap();
        assert!(warnings.is_empty());
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].asset.sha_hex(), frame.asset.sha_hex());

        let mut missing = frame.to_record(false);
        missing.external_path = Some("missing.png".to_owned());
        let store = PictureFrameStore {
            version: 1,
            frames: vec![missing],
        };
        let (loaded, warnings) = PictureFrameStore::from_json(&store.to_json().unwrap()).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("missing"));

        let mut mismatch = frame.to_record(true);
        mismatch.sha256 = "0".repeat(64);
        let store = PictureFrameStore {
            version: 1,
            frames: vec![mismatch],
        };
        assert!(PictureFrameStore::from_json(&store.to_json().unwrap()).is_err());
    }
}
