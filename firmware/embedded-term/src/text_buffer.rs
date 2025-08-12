use crate::cell::Cell;

/// A 2D array of `Cell` to render on screen
pub trait TextBuffer {
    /// Columns
    fn width(&self) -> usize;

    /// Rows
    fn height(&self) -> usize;

    /// Read the character at `(row, col)`
    ///
    /// Avoid use this because it's usually very slow on real hardware.
    fn read(&self, row: usize, col: usize) -> Cell;

    /// Write a character `ch` at `(row, col)`
    fn write(&mut self, row: usize, col: usize, cell: Cell);

    /// Delete one character at `(row, col)`.
    fn delete(&mut self, row: usize, col: usize) {
        self.write(row, col, Cell::default());
    }

    /// Insert one blank line at the bottom, and scroll up one line.
    ///
    /// The default method does single read and write for each pixel.
    /// Usually it needs rewrite for better performance.
    fn new_line(&mut self, cell: Cell) {
        for i in 1..self.height() {
            for j in 0..self.width() {
                self.write(i - 1, j, self.read(i, j));
            }
        }
        for j in 0..self.width() {
            self.write(self.height() - 1, j, cell);
        }
    }

    /// Clear the buffer
    fn clear(&mut self, cell: Cell) {
        for i in 0..self.height() {
            for j in 0..self.width() {
                self.write(i, j, cell);
            }
        }
    }
}
