use crate::cell::{Cell, Flags};
use crate::text_buffer::TextBuffer;
use embedded_graphics::{
    mono_font::{
        iso_8859_1::{FONT_9X18 as FONT, FONT_9X18_BOLD as FONT_BOLD},
        MonoTextStyleBuilder,
    },
    pixelcolor::Rgb888,
    prelude::{DrawTargetExt, Drawable, Point, Size},
    text::{Baseline, Text, TextStyle},
};

const CHAR_SIZE: Size = FONT.character_size;

/// A [`TextBuffer`] on top of a frame buffer
///
/// The internal use [`embedded_graphics`] crate to render fonts to pixels.
///
/// The underlying frame buffer needs to implement `DrawTarget` trait
/// to draw pixels in RGB format.
pub struct TextOnGraphic<D>
where
    D: DrawTargetExt,
{
    width: u32,
    height: u32,
    graphic: D,
}

impl<D> TextOnGraphic<D>
where
    D: DrawTargetExt,
{
    /// Create a new text buffer on graphic.
    pub fn new(graphic: D, width: u32, height: u32) -> Self {
        TextOnGraphic {
            width,
            height,
            graphic,
        }
    }
    /// Get the underlying graphic.
    pub fn get_graphic(&mut self) -> &mut D {
        &mut self.graphic
    }

    /// Return the underlying graphic.
    pub fn into_inner(self) -> D {
        self.graphic
    }
}

impl<C, D> TextBuffer for TextOnGraphic<D>
where
    D: DrawTargetExt<Color = C>,
    C: From<Rgb888>,
{
    #[inline]
    fn width(&self) -> usize {
        (self.width / CHAR_SIZE.width) as usize
    }

    #[inline]
    fn height(&self) -> usize {
        (self.height / CHAR_SIZE.height) as usize
    }

    fn read(&self, _row: usize, _col: usize) -> Cell {
        unimplemented!("reading char from graphic is unsupported")
    }

    #[inline]
    fn write(&mut self, row: usize, col: usize, cell: Cell) {
        if row >= self.height() || col >= self.width() {
            return;
        }
        let mut utf8_buf = [0u8; 8];
        let s = cell.c.encode_utf8(&mut utf8_buf);
        let (fg, bg) = if cell.flags.contains(Flags::INVERSE) {
            (cell.bg, cell.fg)
        } else {
            (cell.fg, cell.bg)
        };
        let mut style = MonoTextStyleBuilder::new()
            .text_color(fg.to_rgb())
            .background_color(bg.to_rgb());
        if cell.flags.contains(Flags::BOLD) {
            style = style.font(&FONT_BOLD);
        } else {
            style = style.font(&FONT);
        }
        if cell.flags.contains(Flags::STRIKEOUT) {
            style = style.strikethrough();
        }
        if cell.flags.contains(Flags::UNDERLINE) {
            style = style.underline();
        }
        let text = Text::with_text_style(
            s,
            Point::new(
                col as i32 * CHAR_SIZE.width as i32,
                row as i32 * CHAR_SIZE.height as i32,
            ),
            style.build(),
            TextStyle::with_baseline(Baseline::Top),
        );

        let mut converted = self.graphic.color_converted();
        text.draw(&mut converted).ok();
    }
}
