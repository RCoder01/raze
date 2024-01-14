use std::io::Write;

use crate::img::{Color, Image};

use super::ImageWriter;

const QOI_OP_RUN: u8 = 0b11_000000;
const QOI_OP_INDEX: u8 = 0b00_000000;
const QOI_OP_DIFF: u8 = 0b01_000000;
const QOI_OP_LUMA: u8 = 0b10_000000;
const QOI_OP_RGB: u8 = 0b11_111110;
const QOI_OP_RGBA: u8 = 0b11_111111;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct QOIColor {
    r: u8,
    g: u8,
    b: u8,
}

impl From<&Color> for QOIColor {
    fn from(value: &Color) -> Self {
        let [r, g, b] = value.to_rgb_bytes();
        Self { r, g, b }
    }
}

#[derive(Debug, Clone, Copy)]
enum ColorDiff {
    Small(u8),
    Medium([u8; 2]),
    Large,
}

impl QOIColor {
    fn hash(self) -> u8 {
        self.r
            .wrapping_mul(3)
            .wrapping_add(self.g.wrapping_mul(5))
            .wrapping_add(self.b.wrapping_mul(7))
            .wrapping_add(255u8.wrapping_mul(11))
            % 64
    }

    fn v(self) -> u32 {
        u32::from_le_bytes([self.r, self.g, self.b, 255])
    }

    fn difference(self, other: Self) -> ColorDiff {
        let dr = self.r as i16 - other.r as i16;
        let dg = self.g as i16 - other.g as i16;
        let db = self.b as i16 - other.b as i16;
        if (-2..=1).contains(&dr) && (-2..=1).contains(&dg) && (-2..=1).contains(&db) {
            let byte = QOI_OP_DIFF
                | ((dr + 2) as u8 & 0b11) << 4
                | ((dg + 2) as u8 & 0b11) << 2
                | ((db + 2) as u8 & 0b11);
            return ColorDiff::Small(byte);
        }
        let drdg = dr - dg;
        let dbdg = db - dg;
        if (-32..32).contains(&dg) && (-8..7).contains(&drdg) && (-8..7).contains(&dbdg) {
            let first_byte = QOI_OP_LUMA | ((dg + 32) as u8 & 0b111111);
            let second_byte = ((drdg + 8) as u8 & 0b1111) << 4 | ((dbdg + 8) as u8 & 0b1111);
            return ColorDiff::Medium([first_byte, second_byte]);
        }
        ColorDiff::Large
    }
}

#[derive(Debug)]
pub struct QOIWriter<'a>(&'a Image);

impl<'a> From<&'a Image> for QOIWriter<'a> {
    fn from(value: &'a Image) -> Self {
        Self(value)
    }
}

impl<'a> ImageWriter for QOIWriter<'a> {
    fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(b"qoif")?;
        writer.write_all(&(self.0.width() as u32).to_be_bytes())?;
        writer.write_all(&(self.0.height() as u32).to_be_bytes())?;
        writer.write_all(&[3, 0])?;
        let mut index = [QOIColor::default(); 64];
        let mut prev_color = QOIColor::default();
        let mut run = 0;
        for px in self.0.data() {
            let color = QOIColor::from(px);
            if color == prev_color {
                run += 1;
                if run == 62 {
                    // println!("Run of {} ({:02X})", run, QOI_OP_RUN | (run - 1));
                    writer.write_all(&[QOI_OP_RUN | (run - 1)])?;
                    run = 0;
                }
                continue;
            }
            if run != 0 {
                // println!("Run of {} ({:02X})", run, QOI_OP_RUN | (run - 1));
                writer.write_all(&[QOI_OP_RUN | (run - 1)])?;
                run = 0;
            }
            if index[color.hash() as usize] == color {
                // println!("Hash of {} ({:02X})", color.hash(), QOI_OP_INDEX | color.hash());
                writer.write_all(&[QOI_OP_INDEX | color.hash()])?;
                prev_color = color;
                continue;
            }
            index[color.hash() as usize] = color;
            match color.difference(prev_color) {
                ColorDiff::Small(byte) => {
                    // println!("small diff {:02X}", byte);
                    writer.write_all(&[byte])?;
                }
                ColorDiff::Medium([b1, b2]) => {
                    // println!("medium diff {:02X} {:02X}", b1, b2);
                    writer.write_all(&[b1, b2])?;
                }
                ColorDiff::Large => {
                    // println!("large diff {:?}", color);
                    writer.write_all(&[QOI_OP_RGB, color.r, color.g, color.b])?;
                }
            }
            prev_color = color;
        }
        if run != 0 {
            // println!("Run of {} ({:02X})", run, QOI_OP_RUN | (run - 1));
            writer.write_all(&[QOI_OP_RUN | (run - 1)])?;
        }
        writer.write_all(&0x0000_0000_0000_0001u64.to_be_bytes())?;
        Ok(())
    }

    fn extension(&self) -> Option<String> {
        Some("qoi".into())
    }
}
