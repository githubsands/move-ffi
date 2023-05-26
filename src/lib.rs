mod internal;
use internal::*;

extern crate libc;

use move_binary_format::file_format::{Signature, Visibility};
use move_core_types::vm_status::StatusCode;

#[no_mangle]
pub extern "C" fn rustdemo() {
    println!("Creating module...");

    let (module, function_name) = make_module_with_function(
        Visibility::Public,
        false,
        Signature(vec![]),
        Signature(vec![]),
        vec![],
    );

    assert_eq!(
        call_script_function_with_args_ty_args_signers(
            module,
            function_name,
            vec![],
            vec![],
            vec![],
        )
        .err()
        .unwrap()
        .major_status(),
        StatusCode::ABORTED,
    );

    println!("Ran move code!")
}
