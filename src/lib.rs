#![warn(missing_docs)]

//! Encoding video with x264.

extern crate framing;
extern crate x264_sys;

mod data;
mod format;
mod image;
mod setup;

pub use data::{Data, Priority, Unit};
pub use format::Format;
pub use image::Image;
pub use setup::{Preset, Setup, Tune};

use std::{mem, ptr};
use std::marker::PhantomData;
use x264_sys::x264;

/// Encodes your video. Be nice to it!
pub struct Encoder<F> {
    raw: *mut x264::x264_t,
    spooky: PhantomData<F>
}

impl<F> Encoder<F> {
    /// Feeds the encoder.
    ///
    /// You have to give the encoder an image and a
    /// [`pts`](https://en.wikipedia.org/wiki/Presentation_timestamp) to encode.
    /// The output is a byte sequence you can feed to stuff, and some image
    /// data that you can use, too - you hopefully being a muxer or something.
    pub fn encode<T>(&mut self, pts: i64, image: &T)
        -> Result<(Data, Picture), Error>
    where
        T: Image<Format = F>,
        F: Format
    {
        let mut picture = unsafe {
            let mut picture = mem::uninitialized();
            x264::x264_picture_init(&mut picture);
            picture.i_pts = pts;
            picture.img = x264ify(image);
            picture
        };

        let mut len = 0;
        let mut stuff = unsafe { mem::uninitialized() };
        let mut raw = unsafe { mem::uninitialized() };

        let err = unsafe {
            x264::x264_encoder_encode(
                self.raw,
                &mut stuff,
                &mut len,
                &mut picture,
                &mut raw
            )
        };

        if err < 0 {
            Err(Error)
        } else {
            let data = unsafe { Data::from_raw_parts(stuff, len as _) };
            Ok((data, Picture { raw }))
        }
    }

    /// Tells the encoder to keep encoding.
    ///
    /// I guess you'd use this because the encoder somehow delayed some frames
    /// or something and so isn't done yet. Don't ask me, I don't know how any
    /// of this works either! That said, you might call in a loop until it's
    /// `done` before disposing of it, if you aren't using the encoder for
    /// streaming purposes (e.g. if you're actually saving the file.)
    pub fn work(&mut self) -> Result<(Data, Picture), Error> {
        let mut len = 0;
        let mut stuff = unsafe { mem::uninitialized() };
        let mut raw = unsafe { mem::uninitialized() };

        let err = unsafe {
            x264::x264_encoder_encode(
                self.raw,
                &mut stuff,
                &mut len,
                ptr::null_mut(),
                &mut raw
            )
        };

        if err < 0 {
            Err(Error)
        } else {
            let data = unsafe { Data::from_raw_parts(stuff, len as _) };
            Ok((data, Picture { raw }))
        }
    }

    /// Gets the video headers.
    ///
    /// Send this before sending other things.
    pub fn headers(&mut self) -> Result<Data, Error> {
        let mut len = 0;
        let mut stuff = unsafe { mem::uninitialized() };

        let err = unsafe {
            x264::x264_encoder_headers(
                self.raw,
                &mut stuff,
                &mut len
            )
        };

        if 0 > err {
            return Err(Error);
        }

        Ok(unsafe { Data::from_raw_parts(stuff, len as _) })
    }

    /// Indicates whether the encoder is done.
    ///
    /// Again, not sure on the specifics, but it's something to do with delayed
    /// frames, and if it's not done it *probably* won't be done until you call
    /// `work` until it says it **is** done.
    pub fn done(&self) -> bool {
        unsafe { 0 == x264::x264_encoder_delayed_frames(self.raw) }
    }
}

impl<F> Drop for Encoder<F> {
    fn drop(&mut self) {
        unsafe { x264::x264_encoder_close(self.raw); }
    }
}

/// Output picture data.
pub struct Picture {
    raw: x264::x264_picture_t
}

impl Picture {
    /// Whether the picture is a keyframe.
    pub fn keyframe(&self) -> bool {
        self.raw.b_keyframe != 0
    }

    /// The presentation timestamp.
    pub fn pts(&self) -> i64 {
        self.raw.i_pts
    }

    /// The decoding timestamp.
    pub fn dts(&self) -> i64 {
        self.raw.i_dts
    }
}

/// An opaque error.
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Error;

fn x264ify<T: Image>(img: &T) -> x264::x264_image_t {
    let planes = img.planes();
    x264::x264_image_t {
        i_csp: T::Format::colorspace(),
        i_plane: T::Format::plane_count(),
        i_stride: img.strides(),
        plane:
            [
                planes[0] as _,
                planes[1] as _,
                planes[2] as _,
                planes[3] as _
            ]
    }
}
