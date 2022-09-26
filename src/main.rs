use core::ffi::{c_char, c_void};
use std::process::ExitCode;

/// A marker type to be embedded into other types just so that they cannot be
/// constructed externally.
type PrivateMarker = ();

/// A type that represents an instance of a class.
#[repr(C)]
pub struct Object {
    _priv: PrivateMarker,
}

/// A type that represents an Objective-C class.
#[repr(C)]
pub struct Class {
    _priv: PrivateMarker,
}

#[link(name = "objc", kind = "dylib")]
extern "C" {
    pub fn objc_getClass(name: *const c_char) -> *const Class;
}

#[link(name = "Foundation", kind = "framework")]
extern "C" {}

// https://github.com/rui314/mold/blob/a2552b6fd11393bfc351cbe162157e449ee54ba1/macho/macho.h#L312-L317
const OBJC_IMAGE_SUPPORTS_GC: u8 = 1 << 1;
const OBJC_IMAGE_REQUIRES_GC: u8 = 1 << 2;
const OBJC_IMAGE_OPTIMIZED_BY_DYLD: u8 = 1 << 3;
const OBJC_IMAGE_SUPPORTS_COMPACTION: u8 = 1 << 4;
const OBJC_IMAGE_IS_SIMULATED: u8 = 1 << 5;
const OBJC_IMAGE_HAS_CATEGORY_CLASS_PROPERTIES: u8 = 1 << 6;

// https://github.com/rui314/mold/blob/a2552b6fd11393bfc351cbe162157e449ee54ba1/macho/macho.h#L742-L747
#[repr(C)]
struct ObjcImageInfo {
    version: u32,
    flags: u8,
    swift_version: u8,
    swift_lang_version: u16,
}

#[no_mangle]
#[link_section = "__DATA_CONST,__objc_imageinfo,regular,no_dead_strip"]
#[used]
static __OBJC_IMAGEINFO: ObjcImageInfo = ObjcImageInfo {
    version: 0,
    flags: OBJC_IMAGE_HAS_CATEGORY_CLASS_PROPERTIES,
    swift_version: 0,
    swift_lang_version: 0,
};

extern "C" {
    #[link_name = "OBJC_CLASS_$_NSNumber"]
    static _OBJC_CLASS___NSNumber: u64;

    #[link_name = "objc_msgSend$numberWithInt:"]
    fn objc_msgSend_numberWithInt(c: *const c_void, unused: u8, n: usize) -> *const Object;

    #[link_name = "objc_msgSend$intValue"]
    fn objc_msgSend_intValue(o: *const c_void) -> usize;
}

#[inline(always)]
#[allow(non_snake_case)]
fn objc_getClass_NSNumber() -> *const c_void {
    unsafe {
        let class_ptr = &core::intrinsics::transmute(&_OBJC_CLASS___NSNumber) as *const _;
        #[cfg(debug_assertions)]
        return core::ptr::read_volatile(class_ptr);
        #[cfg(not(debug_assertions))]
        return *class_ptr;
    }
}

fn main() -> ExitCode {
    unsafe {
        // TODO: Write a test that rips through a bunch of NS* Classes, call `hash` and compare.
        let clazz: *const c_void = objc_getClass_NSNumber();
        let obj = objc_msgSend_numberWithInt(clazz as *const _, 0, 123);
        let v: usize = objc_msgSend_intValue(obj as *const _);
        ExitCode::from(v as u8)
    }
}
