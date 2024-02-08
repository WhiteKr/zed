//! Derived from display-link crate under the following license:
//! <https://github.com/BrainiumLLC/display-link/blob/master/LICENSE-MIT>
//! Apple docs: [CVDisplayLink](https://developer.apple.com/documentation/corevideo/cvdisplaylinkoutputcallback?language=objc)
#![allow(dead_code, non_upper_case_globals)]

use anyhow::Result;
use core_graphics::display::CGDirectDisplayID;
use foreign_types::{foreign_type, ForeignType};
use std::{
    ffi::c_void,
    fmt::{self, Debug, Formatter},
};

#[derive(Debug)]
pub enum CVDisplayLink {}

foreign_type! {
    type CType = CVDisplayLink;
    fn drop = CVDisplayLinkRelease;
    fn clone = CVDisplayLinkRetain;
    pub struct DisplayLink;
    pub struct DisplayLinkRef;
}

impl Debug for DisplayLink {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter
            .debug_tuple("DisplayLink")
            .field(&self.as_ptr())
            .finish()
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct CVTimeStamp {
    pub version: u32,
    pub video_time_scale: i32,
    pub video_time: i64,
    pub host_time: u64,
    pub rate_scalar: f64,
    pub video_refresh_period: i64,
    pub smpte_time: CVSMPTETime,
    pub flags: u64,
    pub reserved: u64,
}

pub type CVTimeStampFlags = u64;

pub const kCVTimeStampVideoTimeValid: CVTimeStampFlags = 1 << 0;
pub const kCVTimeStampHostTimeValid: CVTimeStampFlags = 1 << 1;
pub const kCVTimeStampSMPTETimeValid: CVTimeStampFlags = 1 << 2;
pub const kCVTimeStampVideoRefreshPeriodValid: CVTimeStampFlags = 1 << 3;
pub const kCVTimeStampRateScalarValid: CVTimeStampFlags = 1 << 4;
pub const kCVTimeStampTopField: CVTimeStampFlags = 1 << 16;
pub const kCVTimeStampBottomField: CVTimeStampFlags = 1 << 17;
pub const kCVTimeStampVideoHostTimeValid: CVTimeStampFlags =
    kCVTimeStampVideoTimeValid | kCVTimeStampHostTimeValid;
pub const kCVTimeStampIsInterlaced: CVTimeStampFlags =
    kCVTimeStampTopField | kCVTimeStampBottomField;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub(crate) struct CVSMPTETime {
    pub subframes: i16,
    pub subframe_divisor: i16,
    pub counter: u32,
    pub time_type: u32,
    pub flags: u32,
    pub hours: i16,
    pub minutes: i16,
    pub seconds: i16,
    pub frames: i16,
}

pub type CVSMPTETimeType = u32;

pub const kCVSMPTETimeType24: CVSMPTETimeType = 0;
pub const kCVSMPTETimeType25: CVSMPTETimeType = 1;
pub const kCVSMPTETimeType30Drop: CVSMPTETimeType = 2;
pub const kCVSMPTETimeType30: CVSMPTETimeType = 3;
pub const kCVSMPTETimeType2997: CVSMPTETimeType = 4;
pub const kCVSMPTETimeType2997Drop: CVSMPTETimeType = 5;
pub const kCVSMPTETimeType60: CVSMPTETimeType = 6;
pub const kCVSMPTETimeType5994: CVSMPTETimeType = 7;

pub type CVSMPTETimeFlags = u32;

pub const kCVSMPTETimeValid: CVSMPTETimeFlags = 1 << 0;
pub const kCVSMPTETimeRunning: CVSMPTETimeFlags = 1 << 1;

pub type CVDisplayLinkOutputCallback = unsafe extern "C" fn(
    display_link_out: *mut CVDisplayLink,
    // A pointer to the current timestamp. This represents the timestamp when the callback is called.
    current_time: *const CVTimeStamp,
    // A pointer to the output timestamp. This represents the timestamp for when the frame will be displayed.
    output_time: *const CVTimeStamp,
    // Unused
    flags_in: i64,
    // Unused
    flags_out: *mut i64,
    // A pointer to app-defined data.
    display_link_context: *mut c_void,
) -> i32;

#[link(name = "CoreFoundation", kind = "framework")]
#[link(name = "CoreVideo", kind = "framework")]
#[allow(improper_ctypes)]
extern "C" {
    pub fn CVDisplayLinkCreateWithCGDisplay(
        display_id: u32,
        display_link_out: *mut *mut CVDisplayLink,
    ) -> i32;
    pub fn CVDisplayLinkSetOutputCallback(
        display_link: &mut DisplayLinkRef,
        callback: CVDisplayLinkOutputCallback,
        user_info: *mut c_void,
    ) -> i32;
    pub fn CVDisplayLinkStart(display_link: &mut DisplayLinkRef) -> i32;
    pub fn CVDisplayLinkStop(display_link: &mut DisplayLinkRef) -> i32;
    pub fn CVDisplayLinkRelease(display_link: *mut CVDisplayLink);
    pub fn CVDisplayLinkRetain(display_link: *mut CVDisplayLink) -> *mut CVDisplayLink;
}

impl DisplayLink {
    /// Apple docs: [CVDisplayLinkCreateWithCGDisplay](https://developer.apple.com/documentation/corevideo/1456981-cvdisplaylinkcreatewithcgdisplay?language=objc)
    pub unsafe fn start(
        display_id: CGDirectDisplayID,
        callback: CVDisplayLinkOutputCallback,
        user_info: *mut c_void,
    ) -> Result<Self> {
        let mut display_link: *mut CVDisplayLink = 0 as _;
        let code = CVDisplayLinkCreateWithCGDisplay(display_id, &mut display_link);
        anyhow::ensure!(code == 0, "could not create display link, code: {}", code);

        let mut display_link = DisplayLink::from_ptr(display_link);
        let code = CVDisplayLinkSetOutputCallback(&mut display_link, callback, user_info);
        anyhow::ensure!(code == 0, "could not set output callback, code: {}", code);

        display_link.start()?;

        Ok(display_link)
    }
}

impl DisplayLinkRef {
    /// Apple docs: [CVDisplayLinkStart](https://developer.apple.com/documentation/corevideo/1457193-cvdisplaylinkstart?language=objc)
    pub unsafe fn start(&mut self) -> Result<()> {
        let code = CVDisplayLinkStart(self);
        anyhow::ensure!(code == 0, "could not start display link, code: {}", code);
        Ok(())
    }

    /// Apple docs: [CVDisplayLinkStop](https://developer.apple.com/documentation/corevideo/1457281-cvdisplaylinkstop?language=objc)
    pub unsafe fn stop(&mut self) -> Result<()> {
        let code = CVDisplayLinkStop(self);
        anyhow::ensure!(code == 0, "could not stop display link, code: {}", code);
        Ok(())
    }
}
