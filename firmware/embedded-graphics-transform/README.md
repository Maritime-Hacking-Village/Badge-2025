# Embedded graphics transformations

This crate provides generic implementations of simple transformations - rotate
by 90 degree increments, mirroring and transposing.

These are intended to adapt generic graphics code to a particular display
dimensions and orientation. It provides fixed transformation types which have no
space overhead and minimal time overhead to adjust coordinates and dimensions.
It also provides runtime-configurable rotations with minimal additional space
and time overheads.

Because this is generic, it cannot take advantage of any hardware support for
image rotation. In particular, `DrawTarget::fill_contiguous` will not use any
specialized implementation.

This crate is no-std and has minimal additional dependencies.