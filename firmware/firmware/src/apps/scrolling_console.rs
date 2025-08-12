use crate::platform::cropped_wrapped_draw_target::CroppedWrappedDrawTarget;
use alloc::{
    string::{FromUtf8Error, String},
    vec::Vec,
};
use embedded_graphics::{
    geometry::Dimensions,
    mono_font::{MonoFont, MonoTextStyle},
    prelude::{DrawTarget, Drawable, Point, RgbColor, Size},
    primitives::Rectangle,
    text::{renderer::TextRenderer, Text},
};
use embedded_hal::digital::OutputPin;
use embedded_io_async::{Error, ErrorKind, ErrorType, Write};
use itertools::Itertools;
use mipidsi::{
    interface::{InterfaceAsync, InterfacePixelFormat},
    models::Model,
    DisplayAsync,
};

pub struct ScrollingConsole<'b, DI, MODEL, RST>
where
    DI: InterfaceAsync,
    MODEL: Model,
    MODEL::ColorFormat: InterfacePixelFormat<DI::Word>,
    RST: OutputPin,
    // where
    //     T: DrawTarget<Color: RgbColor> + ScrollableDisplayAsync,
{
    draw_target: CroppedWrappedDrawTarget<DisplayAsync<'b, DI, MODEL, RST>>,
    cur_row: i32,
    scroll_offset: u16,
    reached_bottom: bool,
    font: MonoFont<'static>,
    // area: Rectangle,
}

#[derive(Debug)]
pub enum ConsoleError {
    FormattingError(FromUtf8Error),
}

impl Error for ConsoleError {
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}

impl From<FromUtf8Error> for ConsoleError {
    fn from(value: FromUtf8Error) -> Self {
        ConsoleError::FormattingError(value)
    }
}

impl<'b, DI, MODEL, RST> ErrorType for ScrollingConsole<'b, DI, MODEL, RST>
where
    DI: InterfaceAsync,
    MODEL: Model,
    MODEL::ColorFormat: InterfacePixelFormat<DI::Word>,
    RST: OutputPin,
{
    type Error = ConsoleError;
}

impl<'b, DI, MODEL, RST> Write for ScrollingConsole<'b, DI, MODEL, RST>
where
    DI: InterfaceAsync,
    MODEL: Model,
    MODEL::ColorFormat: InterfacePixelFormat<DI::Word>,
    RST: OutputPin,
{
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.write_line(&String::from_utf8(Vec::from(buf))?).await;
        Ok(buf.len())
    }
}

impl<'b, DI, MODEL, RST> ScrollingConsole<'b, DI, MODEL, RST>
where
    DI: InterfaceAsync,
    MODEL: Model,
    MODEL::ColorFormat: InterfacePixelFormat<DI::Word>,
    RST: OutputPin,
{
    pub async fn new(
        mut draw_target: DisplayAsync<'b, DI, MODEL, RST>,
        font: MonoFont<'static>,
        area: Rectangle,
    ) -> Self {
        draw_target.set_vertical_scroll_region(0, 0).await.unwrap();
        Self {
            draw_target: CroppedWrappedDrawTarget(draw_target, area),
            cur_row: 0,
            scroll_offset: 0,
            reached_bottom: false,
            font,
        }
    }

    #[allow(unused)]
    pub fn get_display(&mut self) -> &mut DisplayAsync<'b, DI, MODEL, RST> {
        &mut (self.draw_target.0)
    }

    pub async fn into_inner(mut self) -> DisplayAsync<'b, DI, MODEL, RST> {
        self.draw_target
            .0
            .set_vertical_scroll_offset(0)
            .await
            .unwrap();
        self.draw_target.0
    }

    pub async fn write_line(&mut self, text: &str) {
        let style = MonoTextStyle::new(&self.font, MODEL::ColorFormat::WHITE);

        // let mut translated = self.draw_target.translated(Point::new(35, 0));
        // let draw_target =
        //     translated.cropped(&Rectangle::new(Point::new(0, 0), Size::new(170, 320)));
        // let draw_target = CroppedWrappedDrawTarget(self.draw_target, self.area);

        let bbox = self.draw_target.bounding_box();
        let bottom_right = bbox.bottom_right().unwrap();
        let width = (bottom_right.x - bbox.top_left.x + 1) as u32;
        let max_cols =
            width / (style.font.character_size.width as u32 + style.font.character_spacing as u32);
        let height = bottom_right.y - bbox.top_left.y + 1;

        // let mut last_line_was_full = false;
        for line in text.replace("\r\n", "\n").split('\n') {
            for chunk in &line.chars().chunks(max_cols as usize) {
                let s: String = chunk.collect();

                // if last_line_was_full {
                //     last_line_was_full = false;

                //     // if last line was full, start a new line
                //     // and the first character of the current is '\n', skip it
                //     if s.starts_with('\n') {
                //         s.remove(0);
                //     }
                //     // if there is nothing to print, skip it
                //     if s.len() == 0 {
                //         continue;
                //     }
                // }

                // debug!(
                //     "will write len {}: byte( {:?}): '{}'",
                //     s.len(),
                //     s.as_bytes()[0],
                //     s.as_str(),
                // );

                // if s.len() >= max_cols as usize {
                //     debug!("LAST LINE WAS FULL");
                //     // last_line_was_full = true;
                // } else {
                //     last_line_was_full = false;
                // }

                // clear line before writing
                //
                self.draw_target
                    .fill_solid(
                        &Rectangle::new(
                            Point::new(bbox.top_left.x, self.cur_row),
                            Size::new(width, style.line_height()),
                        ),
                        RgbColor::RED,
                    )
                    .unwrap();
                self.draw_target.0.flush().await.unwrap();

                if self.reached_bottom {
                    // scroll up by character height
                    self.scroll_offset += style.line_height() as u16;
                    self.scroll_offset %= height as u16;
                    self.draw_target
                        .0
                        .set_vertical_scroll_offset(self.scroll_offset)
                        .await
                        .unwrap();
                }

                // let mut translated = self.draw_target.translated(Point::new(35, 0));
                // let mut draw_target =
                //     translated.cropped(&Rectangle::new(Point::new(0, 0), Size::new(170, 320)));

                // write line
                Text::new(
                    s.as_str(),
                    Point::new(0, self.cur_row + style.font.baseline as i32),
                    style,
                )
                .draw(&mut self.draw_target)
                .unwrap();

                self.draw_target.0.flush().await.unwrap();

                // increment row count and scroll offset
                self.cur_row += style.line_height() as i32;

                if !self.reached_bottom && self.cur_row >= height {
                    self.reached_bottom = true;

                    // scroll up by the wrapped lines
                    self.scroll_offset += (self.cur_row % height) as u16;

                    self.draw_target
                        .0
                        .set_vertical_scroll_offset(self.scroll_offset)
                        .await
                        .unwrap();
                }

                // wrap row count
                self.cur_row %= height;
            }
            self.draw_target.0.flush().await.unwrap();
        }
    }
}
