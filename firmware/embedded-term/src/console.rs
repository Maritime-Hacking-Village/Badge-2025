use crate::ansi::{Attr, ClearMode, Handler, LineClearMode, Mode, Performer};
use crate::cell::{Cell, Flags};
use crate::color::Rgb888;
use crate::graphic::TextOnGraphic;
use crate::text_buffer::TextBuffer;
use crate::text_buffer_cache::TextBufferCache;
use alloc::collections::VecDeque;
use core::cmp::min;
use core::fmt;

use embedded_graphics::prelude::{DrawTarget, OriginDimensions};
use vte::Parser;

/// Console
///
/// Input string with control sequence, output to a [`TextBuffer`].
pub struct Console<T: TextBuffer> {
    /// ANSI escape sequence parser
    parser: Parser,
    /// Inner state
    inner: ConsoleInner<T>,
}

#[derive(Debug, Default, Clone, Copy)]
struct Cursor {
    row: usize,
    col: usize,
}

struct ConsoleInner<T: TextBuffer> {
    /// cursor
    cursor: Cursor,
    /// Saved cursor
    saved_cursor: Cursor,
    /// current attribute template
    temp: Cell,
    /// character buffer
    buf: T,
    /// auto wrap
    auto_wrap: bool,
    /// Reported data for CSI Device Status Report
    report: VecDeque<u8>,
}

/// Console on top of a frame buffer
pub type ConsoleOnGraphic<D> = Console<TextBufferCache<TextOnGraphic<D>>>;

impl<C: From<Rgb888>, D: DrawTarget<Color = C> + OriginDimensions>
    Console<TextBufferCache<TextOnGraphic<D>>>
{
    /// Create a console on top of a frame buffer
    pub fn on_frame_buffer(buffer: D) -> Self {
        let size = buffer.size();
        Self::on_cached_text_buffer(TextOnGraphic::new(buffer, size.width, size.height))
    }
}

impl<T: TextBuffer> Console<TextBufferCache<T>> {
    /// Create a console on top of a [`TextBuffer`] with a cache layer
    pub fn on_cached_text_buffer(buffer: T) -> Self {
        Self::on_text_buffer(TextBufferCache::new(buffer))
    }

    /// Get the text buffer cache
    pub fn get_buffer(&mut self) -> &mut T {
        self.inner.buf.get_inner()
    }

    /// Return the underlying text buffer cache
    pub fn into_inner(self) -> T {
        self.inner.buf.into_inner()
    }
}

impl<T: TextBuffer> Console<T> {
    /// Create a console on top of a [`TextBuffer`]
    pub fn on_text_buffer(buffer: T) -> Self {
        Console {
            parser: Parser::new(),
            inner: ConsoleInner {
                cursor: Cursor::default(),
                saved_cursor: Cursor::default(),
                temp: Cell::default(),
                buf: buffer,
                auto_wrap: true,
                report: VecDeque::new(),
            },
        }
    }

    /// Write a single `byte` to console
    pub fn write_byte(&mut self, byte: u8) {
        self.parser
            .advance(&mut Performer::new(&mut self.inner), byte);
    }

    /// Read result for some commands
    pub fn pop_report(&mut self) -> Option<u8> {
        self.inner.report.pop_front()
    }

    /// Number of rows
    pub fn rows(&self) -> usize {
        self.inner.buf.height()
    }

    /// Number of columns
    pub fn columns(&self) -> usize {
        self.inner.buf.width()
    }
}

impl<T: TextBuffer> fmt::Write for Console<T> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

impl<T: TextBuffer> Handler for ConsoleInner<T> {
    #[inline]
    fn input(&mut self, c: char) {
        trace!("  [input]: {:?} @ {:?}", c, self.cursor);
        if self.cursor.col >= self.buf.width() {
            if !self.auto_wrap {
                // skip this one
                return;
            }
            self.cursor.col = 0;
            self.linefeed();
        }
        let mut temp = self.temp;
        temp.c = c;
        self.buf.write(self.cursor.row, self.cursor.col, temp);
        self.cursor.col += 1;
    }

    #[inline]
    fn goto(&mut self, row: usize, col: usize) {
        trace!("Going to: line={}, col={}", row, col);
        self.cursor.row = min(row, self.buf.height());
        self.cursor.col = min(col, self.buf.width());
    }

    #[inline]
    fn goto_line(&mut self, row: usize) {
        trace!("Going to line: {}", row);
        self.goto(row, self.cursor.col)
    }

    #[inline]
    fn goto_col(&mut self, col: usize) {
        trace!("Going to column: {}", col);
        self.goto(self.cursor.row, col)
    }

    #[inline]
    fn move_up(&mut self, rows: usize) {
        trace!("Moving up: {}", rows);
        self.goto(self.cursor.row.saturating_sub(rows), self.cursor.col)
    }

    #[inline]
    fn move_down(&mut self, rows: usize) {
        trace!("Moving down: {}", rows);
        self.goto(
            min(self.cursor.row + rows, self.buf.height() - 1) as _,
            self.cursor.col,
        )
    }

    #[inline]
    fn move_forward(&mut self, cols: usize) {
        trace!("Moving forward: {}", cols);
        self.cursor.col = min(self.cursor.col + cols, self.buf.width() - 1);
    }

    #[inline]
    fn move_backward(&mut self, cols: usize) {
        trace!("Moving backward: {}", cols);
        self.cursor.col = self.cursor.col.saturating_sub(cols);
    }

    #[inline]
    fn move_down_and_cr(&mut self, rows: usize) {
        trace!("Moving down and cr: {}", rows);
        self.goto(min(self.cursor.row + rows, self.buf.height() - 1) as _, 0)
    }

    #[inline]
    fn move_up_and_cr(&mut self, rows: usize) {
        trace!("Moving up and cr: {}", rows);
        self.goto(self.cursor.row.saturating_sub(rows), 0)
    }

    #[inline]
    fn put_tab(&mut self, count: u16) {
        let mut count = count;
        let bg = self.temp.bg();
        while self.cursor.col < self.buf.width() && count > 0 {
            count -= 1;
            loop {
                self.buf.write(self.cursor.row, self.cursor.col, bg);
                self.cursor.col += 1;
                if self.cursor.col == self.buf.width() || self.cursor.col % 8 == 0 {
                    break;
                }
            }
        }
    }

    #[inline]
    fn backspace(&mut self) {
        trace!("Backspace");
        if self.cursor.col > 0 {
            self.cursor.col -= 1;
        }
    }

    #[inline]
    fn carriage_return(&mut self) {
        trace!("Carriage return");
        self.cursor.col = 0;
    }

    #[inline]
    fn linefeed(&mut self) {
        trace!("Linefeed");
        self.cursor.col = 0;
        if self.cursor.row < self.buf.height() - 1 {
            self.cursor.row += 1;
        } else {
            self.buf.new_line(self.temp);
        }
    }

    #[inline]
    fn scroll_up(&mut self, rows: usize) {
        debug!("[Unhandled CSI] scroll_up {:?}", rows);
    }

    #[inline]
    fn scroll_down(&mut self, rows: usize) {
        debug!("[Unhandled CSI] scroll_down {:?}", rows);
    }

    #[inline]
    fn erase_chars(&mut self, count: usize) {
        trace!("Erasing chars: count={}, col={}", count, self.cursor.col);

        let start = self.cursor.col;
        let end = min(start + count, self.buf.width());

        // Cleared cells have current background color set.
        let bg = self.temp.bg();
        for i in start..end {
            self.buf.write(self.cursor.row, i, bg);
        }
    }
    #[inline]
    fn delete_chars(&mut self, count: usize) {
        let columns = self.buf.width();
        let count = min(count, columns - self.cursor.col - 1);
        let row = self.cursor.row;

        let start = self.cursor.col;
        let end = start + count;

        let bg = self.temp.bg();
        for i in end..columns {
            self.buf.write(row, i - count, self.buf.read(row, i));
            self.buf.write(row, i, bg);
        }
    }

    /// Save current cursor position.
    fn save_cursor_position(&mut self) {
        trace!("Saving cursor position");
        self.saved_cursor = self.cursor;
    }

    /// Restore cursor position.
    fn restore_cursor_position(&mut self) {
        trace!("Restoring cursor position");
        self.cursor = self.saved_cursor;
    }

    #[inline]
    fn clear_line(&mut self, mode: LineClearMode) {
        trace!("Clearing line: {:?}", mode);
        let bg = self.temp.bg();
        match mode {
            LineClearMode::Right => {
                for i in self.cursor.col..self.buf.width() {
                    self.buf.write(self.cursor.row, i, bg);
                }
            }
            LineClearMode::Left => {
                for i in 0..=self.cursor.col {
                    self.buf.write(self.cursor.row, i, bg);
                }
            }
            LineClearMode::All => {
                for i in 0..self.buf.width() {
                    self.buf.write(self.cursor.row, i, bg);
                }
            }
        }
    }

    #[inline]
    fn clear_screen(&mut self, mode: ClearMode) {
        trace!("Clearing screen: {:?}", mode);
        let bg = self.temp.bg();
        let row = self.cursor.row;
        let col = self.cursor.col;
        match mode {
            ClearMode::Above => {
                for i in 0..row {
                    for j in 0..self.buf.width() {
                        self.buf.write(i, j, bg);
                    }
                }
                for j in 0..col {
                    self.buf.write(row, j, bg);
                }
            }
            ClearMode::Below => {
                for j in col..self.buf.width() {
                    self.buf.write(row, j, bg);
                }
                for i in row + 1..self.buf.height() {
                    for j in 0..self.buf.width() {
                        self.buf.write(i, j, bg);
                    }
                }
            }
            ClearMode::All => {
                self.buf.clear(bg);
                self.cursor = Cursor::default();
            }
            _ => {}
        }
    }

    #[inline]
    fn terminal_attribute(&mut self, attr: Attr) {
        trace!("Setting attribute: {:?}", attr);
        match attr {
            Attr::Foreground(color) => self.temp.fg = color,
            Attr::Background(color) => self.temp.bg = color,
            Attr::Reset => self.temp = Cell::default(),
            Attr::Reverse => self.temp.flags |= Flags::INVERSE,
            Attr::CancelReverse => self.temp.flags.remove(Flags::INVERSE),
            Attr::Bold => self.temp.flags.insert(Flags::BOLD),
            Attr::CancelBold => self.temp.flags.remove(Flags::BOLD),
            Attr::Dim => self.temp.flags.insert(Flags::DIM),
            Attr::CancelBoldDim => self.temp.flags.remove(Flags::BOLD | Flags::DIM),
            Attr::Italic => self.temp.flags.insert(Flags::ITALIC),
            Attr::CancelItalic => self.temp.flags.remove(Flags::ITALIC),
            Attr::Underline => self.temp.flags.insert(Flags::UNDERLINE),
            Attr::CancelUnderline => self.temp.flags.remove(Flags::UNDERLINE),
            Attr::Hidden => self.temp.flags.insert(Flags::HIDDEN),
            Attr::CancelHidden => self.temp.flags.remove(Flags::HIDDEN),
            Attr::Strike => self.temp.flags.insert(Flags::STRIKEOUT),
            Attr::CancelStrike => self.temp.flags.remove(Flags::STRIKEOUT),
            _ => {
                debug!("Term got unhandled attr: {:?}", attr);
            }
        }
    }

    #[inline]
    fn set_mode(&mut self, mode: Mode) {
        if mode == Mode::LineWrap {
            self.auto_wrap = true;
        } else {
            debug!("[Unhandled CSI] Setting mode: {:?}", mode);
        }
    }

    #[inline]
    fn unset_mode(&mut self, mode: Mode) {
        if mode == Mode::LineWrap {
            self.auto_wrap = false;
        } else {
            debug!("[Unhandled CSI] Setting mode: {:?}", mode);
        }
    }

    #[inline]
    fn set_scrolling_region(&mut self, top: usize, bottom: Option<usize>) {
        let bottom = bottom.unwrap_or_else(|| self.buf.height());
        debug!(
            "[Unhandled CSI] Setting scrolling region: ({};{})",
            top, bottom
        );
    }

    #[inline]
    fn device_status(&mut self, arg: usize) {
        trace!("Reporting device status: {}", arg);
        match arg {
            5 => {
                for &c in b"\x1b[0n" {
                    self.report.push_back(c);
                }
            }
            6 => {
                let s = alloc::format!("\x1b[{};{}R", self.cursor.row + 1, self.cursor.col + 1);
                for c in s.bytes() {
                    self.report.push_back(c);
                }
            }
            _ => debug!("unknown device status query: {}", arg),
        }
    }
}
