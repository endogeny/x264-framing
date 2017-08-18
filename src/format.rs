use framing::{Bgra, Bgr, Rgb};
use std::os::raw::c_int;
use x264;

/// An image format.
pub unsafe trait Format {
    /// The colorspace.
    fn colorspace() -> c_int;
    /// The number of planes.
    fn plane_count() -> c_int;
}

unsafe impl Format for Bgra {
    fn colorspace() -> c_int { x264::X264_CSP_BGRA as _ }
    fn plane_count() -> c_int { 1 }
}

unsafe impl Format for Rgb {
    fn colorspace() -> c_int { x264::X264_CSP_RGB as _ }
    fn plane_count() -> c_int { 1 }
}

unsafe impl Format for Bgr {
    fn colorspace() -> c_int { x264::X264_CSP_BGR as _ }
    fn plane_count() -> c_int { 1 }
}

// TODO(quadrupleslap): Rgb<16>, Bgr<16>, Bgra<16>
// TODO(quadrupleslap): YUV420, YUV422, YUV444
