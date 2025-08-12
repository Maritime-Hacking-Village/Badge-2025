//! Terminal emulator on [embedded-graphics].
//!
//! The crate is `no_std` compatible. It is suitable for embedded systems and OS kernels.
//!
//! The [`Console`] can be built on top of either a [`TextBuffer`] or a frame buffer ([`DrawTarget`]).
//! For example, the [VGA text mode] has a text buffer, while the graphic mode has a frame buffer.
//!
//! It can be tested in SDL2 with the help of [`embedded_graphics_simulator`](https://docs.rs//embedded-graphics/#simulator).
//! See examples for details.
//!
//! [embedded-graphics]: embedded_graphics
//! [`DrawTarget`]: embedded_graphics::draw_target::DrawTarget
//! [VGA text mode]: https://en.wikipedia.org/wiki/VGA_text_mode

#![no_std]
#![deny(unsafe_code)]
#![deny(warnings)]
#![deny(missing_docs)]

#[macro_use]
extern crate alloc;

#[cfg(feature = "log")]
#[macro_use]
extern crate log;
#[cfg(not(feature = "log"))]
#[macro_use]
mod log;

pub use console::{Console, ConsoleOnGraphic};
pub use graphic::TextOnGraphic;
pub use text_buffer::TextBuffer;
pub use text_buffer_cache::TextBufferCache;

mod ansi;
mod cell;
mod color;
mod console;
mod graphic;
mod text_buffer;
mod text_buffer_cache;
