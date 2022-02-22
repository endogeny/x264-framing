use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::os::raw::{c_char, c_int};
use x264;
use {Encoder, Error, Format};

/// Used to build the encoder.
pub struct Setup {
    raw: x264::x264_param_t,
}

impl Setup {
    /// Begin with a preset.
    ///
    /// In most cases, no further customization is necessary.
    pub fn preset(preset: Preset, tune: Tune, fast_decode: bool, zero_latency: bool) -> Self {
        let mut raw = MaybeUninit::uninit();

        // Name validity verified at compile-time.
        assert_eq!(0, unsafe {
            x264::x264_param_default_preset(
                raw.as_mut_ptr(),
                preset.to_cstr(),
                tune.to_cstr(fast_decode, zero_latency),
            )
        });
        let raw = unsafe { raw.assume_init() };

        Self { raw }
    }

    /// The first pass will be faster, probably.
    pub fn fastfirstpass(mut self) -> Self {
        unsafe {
            x264::x264_param_apply_fastfirstpass(&mut self.raw);
        }
        self
    }

    /// The width of the video, in pixels. Set this!
    pub fn width(mut self, width: c_int) -> Self {
        self.raw.i_width = width;
        self
    }

    /// The height of the video, in pixels. Set this!
    pub fn height(mut self, height: c_int) -> Self {
        self.raw.i_height = height;
        self
    }

    /// The video's FPS, represented as a rational number.
    pub fn fps(mut self, num: u32, den: u32) -> Self {
        self.raw.i_fps_num = num;
        self.raw.i_fps_den = den;
        self
    }

    /// The encoder's time base, used in rate control with timestamps.
    pub fn timebase(mut self, num: u32, den: u32) -> Self {
        self.raw.i_timebase_num = num;
        self.raw.i_timebase_den = den;
        self
    }

    /// If you need to know what this is, you already do.
    pub fn annexb(mut self, annexb: bool) -> Self {
        self.raw.b_annexb = if annexb { 1 } else { 0 };
        self
    }

    /// Approximately restricts the bitrate to somewhere in a huge ballpark.
    pub fn bitrate(mut self, bitrate: c_int) -> Self {
        self.raw.rc.i_bitrate = bitrate;
        self
    }

    /// Use the baseline profile, in the event that your decoders are crippled.
    pub fn baseline(mut self) -> Self {
        unsafe {
            x264::x264_param_apply_profile(&mut self.raw, b"baseline\0" as *const _ as _);
        }
        self
    }

    /// Please don't use this.
    pub fn main(mut self) -> Self {
        unsafe {
            x264::x264_param_apply_profile(&mut self.raw, b"main\0" as *const _ as _);
        }
        self
    }

    /// Tells the encoder to use the high profile.
    ///
    /// This restricts its wizardry to something lesser decoders can understand.
    /// You shouldn't need to do this most of the time, but do it anyway if
    /// you're paranoid.
    pub fn high(mut self) -> Self {
        unsafe {
            x264::x264_param_apply_profile(&mut self.raw, b"high\0" as *const _ as _);
        }
        self
    }

    /// Builds the encoder.
    pub fn build<F: Format>(mut self) -> Result<Encoder<F>, Error> {
        self.raw.i_csp = F::colorspace();
        let raw = unsafe { x264::x264_encoder_open(&mut self.raw) };

        if raw.is_null() {
            return Err(Error);
        }

        Ok(Encoder {
            raw,
            spooky: PhantomData,
        })
    }
}

impl Default for Setup {
    fn default() -> Self {
        let raw = unsafe {
            let mut raw = MaybeUninit::uninit();
            x264::x264_param_default(raw.as_mut_ptr());
            raw.assume_init()
        };

        Self { raw }
    }
}

/// An encoder preset.
///
/// The variant names are hopefully self-explanatory.
#[allow(missing_docs)]
#[derive(Clone, Copy, Hash, Debug, Eq, PartialEq)]
pub enum Preset {
    Ultrafast,
    Superfast,
    Veryfast,
    Faster,
    Fast,
    Medium,
    Slow,
    Slower,
    Veryslow,
    Placebo,
}

impl Preset {
    /// Channels the preset into an arcane incantation fit for the encoder.
    pub fn to_cstr(self) -> *const c_char {
        use self::Preset::*;

        (match self {
            Ultrafast => b"ultrafast\0" as *const u8,
            Superfast => b"superfast\0" as *const u8,
            Veryfast => b"veryfast\0" as *const u8,
            Faster => b"faster\0" as *const u8,
            Fast => b"fast\0" as *const u8,
            Medium => b"medium\0" as *const u8,
            Slow => b"slow\0" as *const u8,
            Slower => b"slower\0" as *const u8,
            Veryslow => b"veryslow\0" as *const u8,
            Placebo => b"placebo\0" as *const u8,
        }) as _
    }
}

/// An encoder tuning.
///
/// The variant names are hopefully self-explanatory.
#[allow(missing_docs)]
#[derive(Clone, Copy, Hash, Debug, Eq, PartialEq)]
pub enum Tune {
    None,
    Film,
    Animation,
    Grain,
    StillImage,
    Psnr,
    Ssim,
}

impl Tune {
    /// Channels the tune into an arcane incantation fit for the encoder.
    pub fn to_cstr(self, fast_decode: bool, zero_latency: bool) -> *const c_char {
        (if !fast_decode && !zero_latency {
            match self {
                Tune::None => b"\0" as *const u8,
                Tune::Film => b"film\0" as *const u8,
                Tune::Animation => b"animation\0" as *const u8,
                Tune::Grain => b"grain\0" as *const u8,
                Tune::StillImage => b"stillimage\0" as *const u8,
                Tune::Psnr => b"psnr\0" as *const u8,
                Tune::Ssim => b"ssim\0" as *const u8,
            }
        } else if fast_decode && !zero_latency {
            match self {
                Tune::None => b"fastdecode\0" as *const u8,
                Tune::Film => b"fastdecode,film\0" as *const u8,
                Tune::Animation => b"fastdecode,animation\0" as *const u8,
                Tune::Grain => b"fastdecode,grain\0" as *const u8,
                Tune::StillImage => b"fastdecode,stillimage\0" as *const u8,
                Tune::Psnr => b"fastdecode,psnr\0" as *const u8,
                Tune::Ssim => b"fastdecode,ssim\0" as *const u8,
            }
        } else if !fast_decode && zero_latency {
            match self {
                Tune::None => b"zerolatency\0" as *const u8,
                Tune::Film => b"zerolatency,film\0" as *const u8,
                Tune::Animation => b"zerolatency,animation\0" as *const u8,
                Tune::Grain => b"zerolatency,grain\0" as *const u8,
                Tune::StillImage => b"zerolatency,stillimage\0" as *const u8,
                Tune::Psnr => b"zerolatency,psnr\0" as *const u8,
                Tune::Ssim => b"zerolatency,ssim\0" as *const u8,
            }
        } else {
            match self {
                Tune::None => b"fastdecode,zerolatency\0" as *const u8,
                Tune::Film => b"fastdecode,zerolatency,film\0" as *const u8,
                Tune::Animation => b"fastdecode,zerolatency,animation\0" as *const u8,
                Tune::Grain => b"fastdecode,zerolatency,grain\0" as *const u8,
                Tune::StillImage => b"fastdecode,zerolatency,stillimage\0" as *const u8,
                Tune::Psnr => b"fastdecode,zerolatency,psnr\0" as *const u8,
                Tune::Ssim => b"fastdecode,zerolatency,ssim\0" as *const u8,
            }
        }) as _
    }
}
