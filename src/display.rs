//! In-world terminal raster.
//!
//! Turns session byte output into pixels on a Bevy Image used as a texture
//! on the TerminalScreen mesh inside the physics world.

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

const FONT_W: usize = 8;
const FONT_H: usize = 16;

#[derive(Component)]
pub struct TerminalDisplay {
    pub cols: usize,
    pub rows: usize,
    pub cursor_col: usize,
    pub cursor_row: usize,
    /// Row-major character cells (ASCII / basic Latin for v0).
    pub cells: Vec<u8>,
    pub image_handle: Handle<Image>,
    pub dirty: bool,
    /// Minimal CSI/OSC state: ignore for v0, strip ESC sequences coarsely.
    pub escape: EscapeState,
}

#[derive(Default)]
pub enum EscapeState {
    #[default]
    Normal,
    Esc,
    Csi,
}

impl TerminalDisplay {
    pub fn new(images: &mut Assets<Image>, cols: usize, rows: usize) -> Self {
        let width = (cols * FONT_W) as u32;
        let height = (rows * FONT_H) as u32;
        let pixel_count = (width * height * 4) as usize;
        let mut data = vec![0u8; pixel_count];
        // Dark background
        for px in data.chunks_mut(4) {
            px[0] = 12;
            px[1] = 14;
            px[2] = 18;
            px[3] = 255;
        }

        let image = Image::new(
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );

        Self {
            cols,
            rows,
            cursor_col: 0,
            cursor_row: 0,
            cells: vec![b' '; cols * rows],
            image_handle: images.add(image),
            dirty: true,
            escape: EscapeState::Normal,
        }
    }

    pub fn feed_bytes(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.feed_byte(b);
        }
        self.dirty = true;
    }

    fn feed_byte(&mut self, b: u8) {
        match self.escape {
            EscapeState::Normal => match b {
                0x1b => self.escape = EscapeState::Esc,
                b'\n' => self.newline(),
                b'\r' => self.cursor_col = 0,
                0x08 | 0x7f => {
                    if self.cursor_col > 0 {
                        self.cursor_col -= 1;
                        self.set_cell(self.cursor_col, self.cursor_row, b' ');
                    }
                }
                b if (32..127).contains(&b) => {
                    self.set_cell(self.cursor_col, self.cursor_row, b);
                    self.cursor_col += 1;
                    if self.cursor_col >= self.cols {
                        self.newline();
                    }
                }
                _ => {}
            },
            EscapeState::Esc => {
                if b == b'[' {
                    self.escape = EscapeState::Csi;
                } else {
                    self.escape = EscapeState::Normal;
                }
            }
            EscapeState::Csi => {
                // End CSI on letter
                if (b'@'..=b'~').contains(&b) {
                    self.escape = EscapeState::Normal;
                }
            }
        }
    }

    fn newline(&mut self) {
        self.cursor_col = 0;
        self.cursor_row += 1;
        if self.cursor_row >= self.rows {
            self.scroll_up();
            self.cursor_row = self.rows - 1;
        }
    }

    fn scroll_up(&mut self) {
        self.cells.copy_within(self.cols.., 0);
        let start = (self.rows - 1) * self.cols;
        for c in &mut self.cells[start..] {
            *c = b' ';
        }
    }

    fn set_cell(&mut self, col: usize, row: usize, ch: u8) {
        if col < self.cols && row < self.rows {
            self.cells[row * self.cols + col] = ch;
        }
    }

    pub fn rasterize(&self, image: &mut Image) {
        let w = (self.cols * FONT_W) as usize;
        let h = (self.rows * FONT_H) as usize;
        let data = image.data.as_mut().unwrap();
        // Clear
        for px in data.chunks_mut(4) {
            px[0] = 12;
            px[1] = 14;
            px[2] = 18;
            px[3] = 255;
        }

        for row in 0..self.rows {
            for col in 0..self.cols {
                let ch = self.cells[row * self.cols + col];
                blit_glyph(data, w, h, col * FONT_W, row * FONT_H, ch);
            }
        }

        // Cursor block
        let cx = self.cursor_col * FONT_W;
        let cy = self.cursor_row * FONT_H + FONT_H - 2;
        for x in 0..FONT_W {
            put_px(data, w, h, cx + x, cy, 180, 220, 180);
            put_px(data, w, h, cx + x, cy + 1, 180, 220, 180);
        }
    }
}

fn put_px(data: &mut [u8], w: usize, _h: usize, x: usize, y: usize, r: u8, g: u8, b: u8) {
    if x >= w {
        return;
    }
    let idx = (y * w + x) * 4;
    if idx + 3 < data.len() {
        data[idx] = r;
        data[idx + 1] = g;
        data[idx + 2] = b;
        data[idx + 3] = 255;
    }
}

/// Tiny 8x8-ish pattern glyphs for printable ASCII (very minimal bitmap font).
fn blit_glyph(data: &mut [u8], w: usize, h: usize, x0: usize, y0: usize, ch: u8) {
    if ch == b' ' {
        return;
    }
    // Generate a stable pseudo-glyph from character code so text is readable enough.
    let seed = ch as usize;
    for dy in 2..FONT_H - 2 {
        for dx in 1..FONT_W - 1 {
            let on = ((seed.wrapping_mul(31) + dx * 7 + dy * 13) % 5) != 0
                && dy > 1
                && (dx == 1 || dx == FONT_W - 2 || (dy + seed) % 3 == 0 || ch.is_ascii_alphanumeric());
            // Prefer denser fill for alphanumeric
            let on = if ch.is_ascii_alphanumeric() {
                (dy > 2 && dy < FONT_H - 3)
                    && (dx == 2
                        || dx == FONT_W - 3
                        || (ch as usize + dy * 3) % 4 == 0
                        || dx == 1 + (seed % 3))
            } else {
                on
            };
            if on {
                put_px(data, w, h, x0 + dx, y0 + dy, 160, 255, 160);
            }
        }
    }
    let _ = seed;
}

pub fn update_terminal_texture(
    mut images: ResMut<Assets<Image>>,
    mut query: Query<&mut TerminalDisplay>,
) {
    for mut display in query.iter_mut() {
        if !display.dirty {
            continue;
        }
        if let Some(image) = images.get_mut(&display.image_handle) {
            display.rasterize(image);
            display.dirty = false;
        }
    }
}
