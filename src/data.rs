use std::marker::PhantomData;
use std::slice;
use x264;

/// The result of encoding. Exciting stuff, isn't it?
pub struct Data<'a> {
    ptr: *mut x264::x264_nal_t,
    len: usize,
    spooky: PhantomData<&'a [x264::x264_nal_t]>
}

impl<'a> Data<'a> {
    /// Nothing to see here.
    pub unsafe fn from_raw_parts(
        ptr: *mut x264::x264_nal_t,
        len: usize
    ) -> Self {
        Data { ptr, len, spooky: PhantomData }
    }

    /// The number of units in this data sequence.
    pub fn len(&self) -> usize {
        self.len
    }

    /// The `i`th unit.
    ///
    /// # Panics
    ///
    /// Panics if `i` is out-of-bounds. In order to be not-out-of-bounds, also
    /// known as in-bounds, `i` must be less than `len`.
    pub fn unit(&self, i: usize) -> Unit<'a> {
        const D: i32 = x264::nal_priority_e::NAL_PRIORITY_DISPOSABLE as _;
        const L: i32 = x264::nal_priority_e::NAL_PRIORITY_LOW as _;
        const H: i32 = x264::nal_priority_e::NAL_PRIORITY_HIGH as _;

        assert!(i < self.len);

        let nal = unsafe {
            *self.ptr.offset(i as _)
        };

        Unit {
            priority:
                match nal.i_ref_idc {
                    D => Priority::Disposable,
                    L => Priority::Low,
                    H => Priority::High,
                    _ => Priority::Highest,
                },
            payload:
                unsafe {
                    slice::from_raw_parts(
                        nal.p_payload,
                        nal.i_payload as _
                    )
                }
        }
    }

    /// The entire chunk of data, as one big byte-slice.
    ///
    /// It has none of the elegance of processing each unit separately, but hey,
    /// it works. More importantly, it's a lot simpler, and if you're not using
    /// any of the fancy priority stuff, it won't even make a difference.
    pub fn entirety(&self) -> &[u8] {
        if self.len == 0 {
            &[]
        } else {
            let (a, b) = unsafe {
                let a = *self.ptr;
                let b = *self.ptr.offset((self.len - 1) as _);
                (a, b)
            };

            let start  = a.p_payload;
            let length = b.p_payload as usize
                       + b.i_payload as usize
                       - start as usize;

            unsafe { slice::from_raw_parts(start, length) }
        }
    }
}

/// A unit of data, which corresponds to a NAL.
///
/// Really, I just wanted desperately to avoid creating a struct with a
/// three-letter name. The alternative name isn't much better, though. Guess
/// you'll just have to deal with it.
pub struct Unit<'a> {
    priority: Priority,
    payload: &'a [u8]
}

impl<'a> Unit<'a> {
    /// how important the unit is when it comes to decoding the video.
    pub fn priority(&self) -> Priority {
        self.priority
    }
}

impl<'a> AsRef<[u8]> for Unit<'a> {
    fn as_ref(&self) -> &[u8] {
        self.payload
    }
}

/// Used to represent how careful you have to be when sending a unit.
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum Priority {
    /// So unimportant that you could get rid of it and I'd barely notice.
    Disposable,
    /// Not very important.
    Low,
    /// Pretty important.
    High,
    /// Why am I trusting you with this again?
    Highest,
}
