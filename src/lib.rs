#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(Debug))]
pub struct ObjcId(pub *const core::ffi::c_void);

#[link(name = "Foundation", kind = "framework")]
extern "C" {}

#[macro_export]
macro_rules! install_objc_image_info {
    () => {
        mod install_objc_image_info_mod {
            // See https://github.com/apple-opensource/ld64/blob/master/src/other/objcimageinfo.cpp#L60-L75
            #[repr(C)]
            struct ObjcImageInfo {
                version: u32,
                flags: u8,
                swift_version: u8,
                swift_lang_version: u16,
            }

            impl ObjcImageInfo {
                // See https://github.com/apple-opensource/ld64/blob/master/src/other/objcimageinfo.cpp#L64-L74
                // const FLAG_SUPPORTS_GC: u8 = 1 << 1;
                // const FLAG_REQUIRES_GC: u8 = 1 << 2;
                // const FLAG_OPTIMIZED_BY_DYLD: u8 = 1 << 3;
                // const FLAG_SUPPORTS_COMPACTION: u8 = 1 << 4;
                // const FLAG_IS_SIMULATED: u8 = 1 << 5;
                const FLAG_HAS_CATEGORY_CLASS_PROPERTIES: u8 = 1 << 6;
            }

            #[no_mangle]
            #[link_section = "__DATA_CONST,__objc_imageinfo,regular,no_dead_strip"]
            #[used]
            static __OBJC_IMAGEINFO: ObjcImageInfo = ObjcImageInfo {
                version: 0,
                flags: ObjcImageInfo::FLAG_HAS_CATEGORY_CLASS_PROPERTIES,
                swift_version: 0,
                swift_lang_version: 0,
            };
        }
    };
}

#[macro_export]
macro_rules! msg_send {
    ($rtnTy:ty, $obj:expr, $name:ident) => {{
        extern "C" {
            #[link_name = concat!("objc_msgSend$",stringify!($name))]
            fn msg_send_fn(c: ObjcId) -> $rtnTy;
        }
        unsafe { msg_send_fn($obj) }
    }};
    ($rtnTy:ty, $obj:expr, $($name:ident : $ty:ty = $arg:expr)+) => {{
        extern "C" {
            #[link_name = concat!("objc_msgSend$",$(stringify!($name), ':'),+)]
            fn msg_send_fn(c: ObjcId, unused: u8 $(,$name : $ty)+) -> $rtnTy;
        }
        unsafe { msg_send_fn($obj, 0 $(,$arg)+) }
    }};
}

#[macro_export]
macro_rules! class {
    ($name:ident) => {{
        extern "C" {
            #[link_name = concat!("OBJC_CLASS_$_",stringify!($name))]
            static OBJC_CLASS_ADDR: u64;
        }
        ObjcId(unsafe {
            let class_ptr = &core::intrinsics::transmute(&OBJC_CLASS_ADDR) as *const _;
            #[cfg(debug_assertions)]
            {
                core::ptr::read_volatile(class_ptr)
            }
            #[cfg(not(debug_assertions))]
            {
                *class_ptr
            }
        })
    }};
}

#[cfg(test)]
mod test {
    use super::*;
    install_objc_image_info!();

    #[link(name = "objc", kind = "dylib")]
    extern "C" {
        fn objc_getClass(name: *const core::ffi::c_char) -> ObjcId;
    }

    #[test]
    fn test() {
        macro_rules! t {
            ($name:ident) => {{
                let actual = class!($name);
                let expected =
                    unsafe { objc_getClass(concat!(stringify!($name), '\0').as_ptr() as _) };

                assert_eq!(
                    actual, expected,
                    concat!("Wrong class - ", stringify!($name))
                );

                let actual = msg_send![usize, actual, hash];
                let expected = msg_send![usize, expected, hash];
                assert_eq!(
                    actual, expected,
                    concat!("Wrong class hash - ", stringify!($name))
                );
            }};
        }

        t!(NSObject);
        t!(NSNumber);
        t!(NSString);
        t!(NSDecimalNumber);
        t!(NSNumberFormatter);
        t!(NSData);
        t!(NSMutableData);
        t!(NSURL);
        t!(NSURLComponents);
        t!(NSURLQueryItem);
        t!(NSArray);
        t!(NSDictionary);
        t!(NSSet);
        t!(NSMutableSet);
    }
}
