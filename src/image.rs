use framing::Image as Frame;
use framing::{AsBytes, Chunky};
use std::os::raw::c_int;
use std::ptr;
use Format;

/// Represents an image that can be handed over to x264.
pub trait Image {
    /// The format of the image.
    type Format: Format;
    /// The stride for each plane.
    fn strides(&self) -> [c_int; 4];
    /// Each plane.
    fn planes(&self) -> [*const u8; 4];
}

impl<F, T> Image for Chunky<F, T>
where
    T: AsRef<[u8]>,
    F: AsBytes + Format,
{
    type Format = F;

    fn strides(&self) -> [c_int; 4] {
        [(F::width() * self.width()) as _, 0, 0, 0]
    }

    fn planes(&self) -> [*const u8; 4] {
        [
            self.bytes().as_ref().as_ptr(),
            ptr::null(),
            ptr::null(),
            ptr::null(),
        ]
    }
}
