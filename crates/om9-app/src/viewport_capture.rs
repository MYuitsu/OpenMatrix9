// SPDX-License-Identifier: LGPL-2.1-or-later
//! ViewCaptureToFile command state, scripted settings and atomic bitmap export.
use om9_core::{Error, Result};
use std::{
    fs,
    io::{BufWriter, Write},
    path::Path,
};

use om9_core::identity::DocumentId;
use om9_scene::workspace::{ViewId, ViewState};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub(crate) struct Settings {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub scale: u32,
    pub grid: bool,
    pub world_axes: bool,
    pub cplane_axes: bool,
    pub transparent: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            scale: 1,
            grid: false,
            world_axes: false,
            cplane_axes: false,
            transparent: false,
        }
    }
}
impl Settings {
    pub fn size(&self, native: [u32; 2], scripted: bool) -> Result<[u32; 2]> {
        let [w, h] = if scripted {
            [
                self.width.unwrap_or(native[0]),
                self.height.unwrap_or(native[1]),
            ]
        } else {
            native
        };
        let scale = self.scale;
        Ok([
            w.checked_mul(scale)
                .filter(|&v| v > 0)
                .ok_or(Error::Invalid("capture width"))?,
            h.checked_mul(scale)
                .filter(|&v| v > 0)
                .ok_or(Error::Invalid("capture height"))?,
        ])
    }
    fn set(&mut self, name: &str, value: &str) -> Result<()> {
        let number = || {
            value
                .parse::<u32>()
                .ok()
                .filter(|&x| x > 0)
                .ok_or(Error::Invalid("capture size must be a positive integer"))
        };
        let flag = || match value.trim_start_matches('_').to_ascii_lowercase().as_str() {
            "yes" => Ok(true),
            "no" => Ok(false),
            _ => Err(Error::Invalid("choose Yes or No")),
        };
        match name.trim_start_matches('_').to_ascii_lowercase().as_str() {
            "width" => self.width = Some(number()?),
            "height" => self.height = Some(number()?),
            "scale" => self.scale = number()?,
            "drawgrid" => self.grid = flag()?,
            "drawworldaxes" => self.world_axes = flag()?,
            "drawcplaneaxes" => self.cplane_axes = flag()?,
            "transparentbackground" => self.transparent = flag()?,
            _ => return Err(Error::Invalid("unknown View Capture setting")),
        }
        Ok(())
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stage {
    Dialog,
    Path,
    Settings,
    Render,
}
#[derive(Clone)]
pub(crate) struct Session {
    pub id: uuid::Uuid,
    pub cancel: om9_core::transaction::Cancellation,
    pub stage: Stage,
    pub view_id: ViewId,
    pub view: ViewState,
    pub stamp: (DocumentId, u64),
    pub selection: u64,
    pub settings: Settings,
    pub path: Option<PathBuf>,
    pub scripted: bool,
}
impl Session {
    pub fn prompt(&self) -> String {
        match self.stage {
            Stage::Dialog => "Save file name: ".into(),
            Stage::Path => "Save file name ( Browse ): ".into(),
            Stage::Settings => {
                let s = &self.settings;
                let flag = |b| if b { "Yes" } else { "No" };
                let dim =
                    |n: Option<u32>| n.map_or_else(|| "Viewport".to_owned(), |n| n.to_string());
                format!(
                    "View Capture Settings ( Width={} Height={} Scale={} DrawGrid={} DrawWorldAxes={} DrawCPlaneAxes={} TransparentBackground={} ): ",
                    dim(s.width),
                    dim(s.height),
                    s.scale,
                    flag(s.grid),
                    flag(s.world_axes),
                    flag(s.cplane_axes),
                    flag(s.transparent)
                )
            }
            Stage::Render => "Saving viewport image…".into(),
        }
    }
    pub fn accept_path(&mut self, path: PathBuf) -> Result<()> {
        format(&path)?;
        self.path = Some(path);
        self.stage = if self.scripted {
            Stage::Settings
        } else {
            Stage::Render
        };
        Ok(())
    }
    fn input(&mut self, text: &str) -> Result<()> {
        let text = text.trim();
        if self.stage == Stage::Path && !text.starts_with('"') {
            if text.trim_start_matches('_').eq_ignore_ascii_case("browse") {
                self.stage = Stage::Dialog;
                return Ok(());
            }
            // A path submitted at its own prompt may contain unquoted spaces.
            return self.accept_path(PathBuf::from(text));
        }
        if text.is_empty() && self.stage == Stage::Settings {
            self.stage = Stage::Render;
            return Ok(());
        }
        let mut args = split_arguments(text)?;
        while let Some(word) = args.next() {
            match self.stage {
                Stage::Path => self.accept_path(word.into())?,
                Stage::Settings => {
                    if word.trim_start_matches('_').eq_ignore_ascii_case("enter") {
                        self.stage = Stage::Render;
                    } else if let Some((key, value)) = word.split_once('=') {
                        self.settings.set(key, value)?;
                    } else {
                        self.settings.set(
                            &word,
                            &args.next().ok_or(Error::Missing("capture option value"))?,
                        )?;
                    }
                }
                _ => return Err(Error::Busy),
            }
        }
        Ok(())
    }
}
fn split_arguments(text: &str) -> Result<std::vec::IntoIter<String>> {
    let mut args = Vec::new();
    let mut word = String::new();
    let mut quote = false;
    for ch in text.chars() {
        if ch == '"' {
            quote = !quote;
        } else if ch.is_whitespace() && !quote {
            if !word.is_empty() {
                args.push(std::mem::take(&mut word));
            }
        } else {
            word.push(ch);
        }
    }
    if quote {
        return Err(Error::Invalid("unclosed file name quote"));
    }
    if !word.is_empty() {
        args.push(word);
    }
    Ok(args.into_iter())
}
impl crate::session::Controller {
    pub(crate) fn start_view_capture(&mut self, scripted: bool) -> Result<()> {
        if self.is_running() {
            return Err(Error::Busy);
        }
        let view_id = self.workspace.active;
        let view = self
            .workspace
            .views
            .get(&view_id)
            .ok_or(Error::Missing("active viewport"))?
            .clone();
        self.view_capture = Some(Session {
            id: uuid::Uuid::new_v4(),
            cancel: Default::default(),
            stage: if scripted { Stage::Path } else { Stage::Dialog },
            view_id,
            view,
            stamp: (self.document().id, self.document().revision),
            selection: self.selection_state().revision,
            settings: self.capture_settings.clone(),
            path: None,
            scripted,
        });
        Ok(())
    }
    /// Apply the owned native picker result to the command that requested it.
    pub fn accept_view_capture_file(
        &mut self,
        path: Option<PathBuf>,
        transparent: bool,
    ) -> Result<()> {
        let capture = self
            .view_capture
            .as_mut()
            .filter(|s| s.stage == Stage::Dialog)
            .ok_or(Error::Missing("viewport image dialog"))?;
        if let Some(path) = path {
            capture.accept_path(path)?;
            capture.settings.transparent = transparent;
        } else {
            self.cancel();
        }
        Ok(())
    }

    pub(crate) fn capture_input(&mut self, text: &str) -> Result<()> {
        if text.trim().eq_ignore_ascii_case("esc") || text.trim().eq_ignore_ascii_case("cancel") {
            self.cancel();
            return Ok(());
        }
        self.view_capture
            .as_mut()
            .ok_or(Error::Missing("capture command"))?
            .input(text)
    }
    pub(crate) fn start_capture_macro(&mut self, tail: &str) -> Result<()> {
        self.start_view_capture(true)?;
        let result = (|| {
            let mut words = split_arguments(tail)?;
            let Some(path) = words.next() else {
                return Ok(());
            };
            let session = self.view_capture.as_mut().unwrap();
            if path.trim_start_matches('_').eq_ignore_ascii_case("browse") {
                session.stage = Stage::Dialog;
                return Ok(());
            }
            session.accept_path(path.into())?;
            let rest = words.collect::<Vec<_>>().join(" ");
            if rest.is_empty() {
                Ok(())
            } else {
                session.input(&rest)
            }
        })();
        if result.is_err() {
            self.view_capture = None;
        }
        result
    }
}

pub(crate) fn format(path: &Path) -> Result<&'static str> {
    match path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "jpg" | "jpeg" => Ok("jpg"),
        "bmp" => Ok("bmp"),
        "pcx" => Ok("pcx"),
        "png" => Ok("png"),
        "tif" | "tiff" => Ok("tiff"),
        "tga" => Ok("tga"),
        _ => Err(Error::Invalid(
            "choose a BMP, JPG, PCX, PNG, TIFF or TGA image",
        )),
    }
}

pub(crate) fn save(
    path: &Path,
    size: [u32; 2],
    mut pixels: Vec<u8>,
    transparent: bool,
    cancel: &om9_core::transaction::Cancellation,
) -> Result<()> {
    if cancel.is_canceled() {
        return Err(Error::Canceled);
    }
    let kind = format(path)?;
    let [w, h] = size;
    if w == 0
        || h == 0
        || u64::from(w) * u64::from(h) != pixels.len() as u64 / 4
        || pixels.len() % 4 != 0
    {
        return Err(Error::Invalid("viewport capture pixels"));
    }
    // MSAA resolves coverage against transparent black. PNG/TGA/TIFF expect
    // straight RGB, while opaque formats keep the black viewport background.
    let alpha = transparent && matches!(kind, "png" | "tga" | "tiff");
    if alpha {
        for p in pixels.chunks_exact_mut(4) {
            let a = u32::from(p[3]);
            if (1..255).contains(&a) {
                for c in &mut p[..3] {
                    *c = ((u32::from(*c) * 255 + a / 2) / a).min(255) as u8;
                }
            }
        }
    }
    let rgba =
        image::RgbaImage::from_raw(w, h, pixels).ok_or(Error::Invalid("capture dimensions"))?;
    let image = image::DynamicImage::ImageRgba8(rgba);
    let stage = path.with_file_name(format!(".om9-view-{}.tmp", uuid::Uuid::new_v4()));
    let file = fs::File::create_new(&stage).map_err(|e| Error::Io(e.to_string()))?;
    let result = (|| {
        let mut writer = BufWriter::new(file);
        if kind == "pcx" {
            write_pcx(&mut writer, &image.to_rgb8())?;
        } else {
            let encoding =
                image::ImageFormat::from_extension(kind).ok_or(Error::Invalid("capture format"))?;
            let image = if alpha {
                image
            } else {
                image::DynamicImage::ImageRgb8(image.into_rgb8())
            };
            image
                .write_to(&mut writer, encoding)
                .map_err(|e| Error::Io(e.to_string()))?;
        }
        writer.flush().map_err(|e| Error::Io(e.to_string()))?;
        writer
            .get_ref()
            .sync_all()
            .map_err(|e| Error::Io(e.to_string()))?;
        drop(writer);
        if cancel.is_canceled() {
            return Err(Error::Canceled);
        }
        fs::rename(&stage, path).map_err(|e| Error::Io(e.to_string()))
    })();
    if result.is_err() {
        let _ = fs::remove_file(stage);
    }
    result
}

fn write_pcx(out: &mut impl Write, image: &image::RgbImage) -> Result<()> {
    let width = u16::try_from(image.width()).map_err(|_| Error::Invalid("PCX width"))?;
    let height = u16::try_from(image.height()).map_err(|_| Error::Invalid("PCX height"))?;
    let stride = width
        .checked_add(width % 2)
        .ok_or(Error::Invalid("PCX scanline"))?;
    let mut header = [0u8; 128];
    header[..4].copy_from_slice(&[10, 5, 1, 8]);
    for (i, value) in [
        (8, width - 1),
        (10, height - 1),
        (12, 96),
        (14, 96),
        (66, stride),
        (68, 1),
    ] {
        header[i..i + 2].copy_from_slice(&value.to_le_bytes());
    }
    header[65] = 3;
    let mut emit = || -> std::io::Result<()> {
        out.write_all(&header)?;
        for row in image.rows() {
            for component in 0..3 {
                let mut samples = row
                    .clone()
                    .map(|p| p[component])
                    .chain((stride > width).then_some(0))
                    .peekable();
                while let Some(value) = samples.next() {
                    let mut count = 1u8;
                    while count < 63 && samples.next_if_eq(&value).is_some() {
                        count += 1;
                    }
                    if count != 1 || value >= 192 {
                        out.write_all(&[192 | count])?;
                    }
                    out.write_all(&[value])?;
                }
            }
        }
        Ok(())
    };
    emit().map_err(|e| Error::Io(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn six_capture_formats_keep_dimensions_channels_and_existing_destination_on_cancel() {
        let dir = std::env::temp_dir().join(format!("om9-view-formats-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&dir).unwrap();
        let mut pixels = vec![0u8; 65 * 3 * 4];
        for p in pixels.chunks_exact_mut(4) {
            p.copy_from_slice(&[193, 20, 77, 255]);
        }
        pixels[..4].copy_from_slice(&[64, 32, 0, 128]);
        for ext in ["png", "bmp", "tga", "tif", "jpg", "pcx"] {
            let path = dir.join(format!("view.{ext}"));
            save(&path, [65, 3], pixels.clone(), true, &Default::default()).unwrap();
            if ext == "pcx" {
                let bytes = fs::read(&path).unwrap();
                assert_eq!(&bytes[..4], &[10, 5, 1, 8]);
                assert_eq!(bytes[65], 3);
                assert_eq!(u16::from_le_bytes(bytes[66..68].try_into().unwrap()), 66);
                let mut unpacked = Vec::new();
                let mut data = bytes[128..].iter().copied();
                while let Some(v) = data.next() {
                    if v >= 192 {
                        unpacked
                            .extend(std::iter::repeat_n(data.next().unwrap(), (v & 63) as usize));
                    } else {
                        unpacked.push(v);
                    }
                }
                assert_eq!(unpacked.len(), 66 * 3 * 3);
                assert_eq!(&unpacked[..3], &[64, 193, 193]);
                assert_eq!(unpacked[65], 0);
                assert_eq!(unpacked[66], 32);
            } else {
                let output = image::open(&path).unwrap().to_rgba8();
                assert_eq!(output.dimensions(), (65, 3));
                if matches!(ext, "png" | "tif" | "tga") {
                    assert_eq!(output.get_pixel(0, 0).0, [128, 64, 0, 128]);
                } else {
                    assert_eq!(output.get_pixel(0, 0)[3], 255);
                }
            }
            let prior = fs::read(&path).unwrap();
            let token = om9_core::transaction::Cancellation::default();
            token.cancel();
            assert!(save(&path, [65, 3], pixels.clone(), true, &token).is_err());
            assert_eq!(fs::read(&path).unwrap(), prior);
            fs::remove_file(path).unwrap();
        }
        assert!(format(&dir.join("invalid.om9")).is_err());
        fs::remove_dir(dir).unwrap();
    }
}
