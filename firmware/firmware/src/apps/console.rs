use crate::platform::{
    cropped_wrapped_draw_target::CroppedWrappedDrawTarget, flushing_display::FlushingDisplay,
    vertical_scrolling::VerticalScrolling,
};
use alloc::string::FromUtf8Error;
use embedded_graphics::{
    mono_font::MonoFont,
    pixelcolor::Rgb565,
    prelude::{DrawTargetExt, OriginDimensions, Point, Size},
    primitives::Rectangle,
};
use embedded_graphics_transform::Rotate90;
use embedded_io_async::{Error, ErrorKind, ErrorType, Write};
use embedded_term::{Console, TextBufferCache, TextOnGraphic};
use panic_probe as _;

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

pub struct ConsoleDisplay<T>
where
    T: DrawTargetExt<Color = Rgb565> + FlushingDisplay,
{
    // pub display: ColorConverted<'a, T, Rgb888>,
    // pub console:
    //     Console<TextBufferCache<TextOnGraphic<Cropped<'a, ColorConverted<'a, T, Rgb888>>>>>,
    // pub console: Console<TextBufferCache<TextOnGraphic<T>>>,
    // pub console: Console<TextBufferCache<TextOnGraphic<T>>>,
    pub console: Console<TextBufferCache<TextOnGraphic<Rotate90<CroppedWrappedDrawTarget<T>>>>>,

    // converted: ColorConverted<'a, T, Rgb888>,
    cur_row: i32,
    scroll_offset: u16,
    reached_bottom: bool,
    font: MonoFont<'static>,
}

impl<T> ErrorType for ConsoleDisplay<T>
where
    T: DrawTargetExt<Color = Rgb565> + FlushingDisplay + VerticalScrolling,
{
    type Error = ConsoleError;
}

impl<T> Write for ConsoleDisplay<T>
where
    T: DrawTargetExt<Color = Rgb565> + FlushingDisplay + VerticalScrolling,
{
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        for byte in buf {
            self.console.write_byte(*byte);
        }

        Ok(buf.len())
    }
}

impl<T> ConsoleDisplay<T>
where
    T: DrawTargetExt<Color = Rgb565> + FlushingDisplay + VerticalScrolling + OriginDimensions,
{
    // pub fn new<C: PixelColor + Into<T::Color>>(display: T) -> Self {
    pub async fn new(display: T, font: MonoFont<'static>) -> Self {
        // display.set_vertical_scroll_region(0, 0).await.unwrap();
        // let converted =
        //     CroppedWrappedConvertedDrawTarget::<'a, _, Rgb888>(display, area, PhantomData);
        // let mut converted = display.color_converted();
        //
        let cropped = CroppedWrappedDrawTarget(
            display,
            Rectangle::new(Point::new(35, 0), Size::new(170, 320)),
        );
        let rotated = Rotate90::new(cropped);
        let console = Console::on_frame_buffer(rotated);

        // Console<TextBufferCache<TextOnGraphic<CroppedWrappedConvertedDrawTarget<'a, T, Rgb888>>>>
        Self {
            // display: converted,
            console: console,
            // converted,
            cur_row: 0,
            scroll_offset: 0,
            reached_bottom: false,
            font,
        }
    }

    pub async fn flush(&mut self) {
        self.get_display()
            .flush()
            .await
            .map_err(|_| "fucked")
            .unwrap();
    }

    pub fn get_display(&mut self) -> &mut T {
        &mut self.console.get_buffer().get_graphic().0
    }

    #[allow(unused)]
    pub fn get_cropped(&mut self) -> &mut CroppedWrappedDrawTarget<T> {
        self.console.get_buffer().get_graphic()
    }

    pub fn get_rotated(&mut self) -> &mut Rotate90<CroppedWrappedDrawTarget<T>> {
        self.console.get_buffer().get_graphic()
    }

    pub fn into_inner(self) -> T {
        self.console.into_inner().into_inner().into_inner().0
    }

    // TODO: This is broken! :-)
    // pub async fn write_line(&mut self, text: &str) {
    //     let font = self.font.clone();
    //     let style = MonoTextStyle::new(&font, RgbColor::WHITE);

    //     // let mut translated = self.draw_target.translated(Point::new(35, 0));
    //     // let draw_target =
    //     //     translated.cropped(&Rectangle::new(Point::new(0, 0), Size::new(170, 320)));

    //     let mut cur_row = self.cur_row;
    //     let mut reached_bottom = self.reached_bottom;
    //     let mut scroll_offset = self.scroll_offset;
    //     let mut display = self.get_display();
    //     // let mut display = Rotate90::new(self.get_display());

    //     // let draw_target = CroppedWrappedDrawTarget(display, area);

    //     let bbox = display
    //         // .translated(Point::new(0, 35))
    //         // .cropped(&Rectangle::new(Point::new(0, 0), Size::new(320, 170)))
    //         .bounding_box();
    //     let bottom_right = bbox.bottom_right().unwrap();
    //     let width = (bottom_right.x - bbox.top_left.x + 1) as u32;
    //     let max_cols =
    //         width / (style.font.character_size.width as u32 + style.font.character_spacing as u32);
    //     let height = bottom_right.y - bbox.top_left.y + 1;
    //     // let mut last_line_was_full = false;
    //     defmt::info!("Height: {}", height);

    //     for line in text.replace("\r\n", "\n").split('\n') {
    //         for chunk in &line.chars().chunks(max_cols as usize) {
    //             let s: String = chunk.collect();

    //             // Clear line before writing
    //             display
    //                 // .translated(Point::new(0, 35))
    //                 // .cropped(&Rectangle::new(Point::new(0, 0), Size::new(320, 170)))
    //                 .fill_solid(
    //                     &Rectangle::new(
    //                         Point::new(bbox.top_left.x, cur_row),
    //                         Size::new(width, style.line_height()),
    //                     ),
    //                     RgbColor::RED,
    //                 )
    //                 .map_err(|_| "Fucked")
    //                 .unwrap();

    //             display.flush().await.map_err(|_| "Fucked").unwrap();

    //             if reached_bottom {
    //                 // Scroll up by character height
    //                 scroll_offset += style.line_height() as u16;
    //                 scroll_offset %= height as u16;
    //                 let _ = display.set_vertical_scroll_offset(scroll_offset).await;
    //             }

    //             defmt::info!("Current row: {}", cur_row);
    //             // Write line
    //             Text::new(
    //                 s.as_str(),
    //                 Point::new(0, cur_row + style.font.baseline as i32),
    //                 style,
    //             )
    //             .draw(
    //                 display, // .translated(Point::new(0, 35))
    //                         // .cropped(&Rectangle::new(Point::new(0, 0), Size::new(320, 170))),
    //             )
    //             .map_err(|_| "Fucked")
    //             .unwrap();
    //             display.flush().await.map_err(|_| "Fucked").unwrap();
    //             // Increment row count and scroll offset
    //             cur_row += style.line_height() as i32;

    //             if !reached_bottom && cur_row >= height {
    //                 reached_bottom = true;

    //                 // Scroll up by the wrapped lines
    //                 scroll_offset += (cur_row % height) as u16;
    //                 let _ = display.set_vertical_scroll_offset(scroll_offset).await;
    //             }

    //             // Wrap row count
    //             cur_row %= height;
    //         }

    //         display.flush().await.map_err(|_| "Fucked").unwrap();
    //     }

    //     self.cur_row = cur_row;
    //     self.reached_bottom = reached_bottom;
    //     self.scroll_offset = scroll_offset;
    // }
    // pub async fn run(&mut self) {
    //     // let converted = CroppedWrappedConvertedDrawTarget::<_, Rgb888>(self.0, area, PhantomData);
    //     // let mut cconv_display = self.0.color_converted::<Rgb888>();
    //     // let mut translated = cconv_display.translated(Point::new(35, 0));
    //     // let cropped_display = translated.cropped(&area);

    //     loop {
    //         // self.console.write_str("Hello!").unwrap();

    //         // self.flush().await;

    //         let mut n = 0;
    //         for i in 0..self.console.rows() * 2 {
    //             for _j in 0..self.console.columns() / 2 {
    //                 n = n + 1;
    //                 // if n > console.rows() * (console.columns() / 2) {
    //                 //     break;
    //                 // }
    //                 self.console
    //                     .write_fmt(format_args!("\x1B[{}m{:2x}\x1B[m", n, n))
    //                     .unwrap();

    //                 self.flush().await;
    //                 Timer::after_millis(100).await;
    //             }
    //             if i < self.console.rows() - 1 {
    //                 self.console.write_str("\n").unwrap();
    //             }
    //         }
    //         Timer::after_secs(1).await;
    //     }
    // }
}

// #[embassy_executor::task]
// pub async fn task(
//     mut console_display: ConsoleDisplay<
//         DisplayAsync<
//             'static,
//             SpiInterfaceAsync<
//                 SpiDeviceWithConfig<
//                     'static,
//                     NoopRawMutex,
//                     Spi<'static, SPI0, Async>,
//                     Output<'static>,
//                 >,
//                 Output<'static>,
//             >,
//             ST7789,
//             Output<'static>,
//         >,
//     >,
// ) -> () {
//     // console_display.run().await;
// }
