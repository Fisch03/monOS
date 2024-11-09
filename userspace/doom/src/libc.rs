use core::ffi::{c_void, CStr, VaList};
use core::fmt::Arguments;

mod io;
mod mem;
mod string;

#[no_mangle]
pub unsafe extern "C" fn unimplemented(s: *const i8) {
    let s = CStr::from_ptr(s);
    println!("unimplemented: {}", s.to_str().unwrap());
}

unsafe fn match_format<F: FnOnce(Arguments<'_>)>(c: u8, ap: &mut VaList, out: F) {
    match c {
        b'd' | b'i' => {
            let x: i32 = ap.arg();
            out(format_args!("{}", x))
        }
        b's' => {
            let s: *const i8 = ap.arg();
            let s = CStr::from_ptr(s).to_str().unwrap();
            out(format_args!("{}", s))
        }
        b'p' => {
            let p: *const c_void = ap.arg();
            out(format_args!("{:p}", p))
        }
        b'x' => {
            let x: i32 = ap.arg();
            out(format_args!("{:x}", x))
        }
        _ => {
            print!("unknown format specifier: {}", c as char);
        }
    }
}
