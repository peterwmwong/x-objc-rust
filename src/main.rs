use std::ffi::{c_char, c_void};

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
static __OBJC_IMAGEINFO: ObjcImageInfo = ObjcImageInfo {
    version: 0,
    flags: OBJC_IMAGE_HAS_CATEGORY_CLASS_PROPERTIES,
    swift_version: 0,
    swift_lang_version: 0,
};

struct SyncWrap(pub *const c_void);
unsafe impl Sync for SyncWrap {}

extern "C" {
    #[link_name = "OBJC_CLASS_$_NSNumber"]
    static _OBJC_CLASS___NSNumber: u64;

    #[link_name = "objc_msgSend$numberWithInt:"]
    fn objc_msgSend_numberWithInt(c: *const c_void, _: u32, n: usize) -> *const Object;

    #[link_name = "objc_msgSend$intValue"]
    fn objc_msgSend_intValue(o: *const c_void) -> usize;
}

// #[link_section = "__DATA_const,__objc_classrefs,regular,no_dead_strip"]
// #[export_name = "\x01L_OBJC_CLASSLIST_REFERENCES_$_.NSNumber"]
// static REF: SyncWrap = SyncWrap(unsafe { std::mem::transmute(&_OBJC_CLASS___NSNumber) });

fn main() -> Result<(), ()> {
    unsafe {
        let clazz: *const c_void =
            core::ptr::read_volatile(&(std::mem::transmute(&_OBJC_CLASS___NSNumber)) as *const _);

        // TODO: Write a test that rips through a bunch of NS* Classes, call `hash` and compare.
        // let clazz2 = objc_getClass("NSNumber\0".as_ptr() as *const _);
        // assert_eq!(clazz2 as usize, clazz as usize);

        let obj = objc_msgSend_numberWithInt(clazz as *const _, 0, 1234);
        let v: usize = objc_msgSend_intValue(obj as *const _);

        println!("{v}");
        Ok(())
    }
}
