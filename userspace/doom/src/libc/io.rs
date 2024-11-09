use super::string::printf;
use core::ffi::{c_char, c_long, c_void, CStr, VaList};

const STDOUT: usize = 1;
const STDERR: usize = 2;

#[no_mangle]
pub unsafe extern "C" fn fopen(filename: *const c_char, mode: *const c_char) -> *mut u32 {
    let filename = CStr::from_ptr(filename);
    let filename = filename.to_str().unwrap();

    let file = match File::open(filename) {
        Some(file) => file,
        None => return core::ptr::null_mut(),
    };

    Box::leak(Box::new(file)) as *mut _ as *mut u32
}

#[no_mangle]
pub unsafe extern "C" fn fclose(stream: *mut u32) -> i32 {
    let file = Box::from_raw(stream as *mut File);
    file.close();
    0
}

#[no_mangle]
pub unsafe extern "C" fn ftell(stream: *mut u32) -> i64 {
    let file = &*(stream as *mut File);
    dbg!(file.get_pos()) as i64
}

#[no_mangle]
pub unsafe extern "C" fn fwrite(
    ptr: *const c_void,
    size: usize,
    count: usize,
    stream: *mut c_void,
) -> i32 {
    match stream as usize {
        STDOUT | STDERR => {
            let buf = core::slice::from_raw_parts(ptr as *const u8, size * count);
            let s = core::str::from_utf8_unchecked(buf);
            print!("{}", s);
            count as i32
        }

        _ => {
            unimplemented!("fwrite to fd");
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn vfprintf(stream: *mut c_void, format: *const u8, ap: VaList) -> i32 {
    if stream as usize > 2 {
        unimplemented!("vfprintf to fd");
    }

    printf(format, ap)
}

#[no_mangle]
pub unsafe extern "C" fn fflush(stream: *mut c_void) -> i32 {
    if stream as usize > 2 {
        unimplemented!("fflush to fd");
    }

    println!("fflush {}", stream as usize);

    0
}
