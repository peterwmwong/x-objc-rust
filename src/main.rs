use std::process::ExitCode;
use x_objc_rust::{class, install_objc_image_info, msg_send, ObjcId};

install_objc_image_info!();

fn main() -> ExitCode {
    let ns_number_class = class!(NSNumber);
    let ns_number = msg_send![ObjcId, ns_number_class, numberWithInt: usize = 123];
    let int_value = msg_send![usize, ns_number, intValue];
    ExitCode::from(int_value as u8)
}
