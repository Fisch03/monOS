use super::string::printf;
use core::ffi::{c_char, c_void, CStr, VaList};

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
    file.get_pos() as i64
}

#[no_mangle]
pub unsafe extern "C" fn fseek(stream: *mut u32, offset: i64, whence: i32) -> i32 {
    use monos_std::io::SeekMode;

    let file = &*(stream as *mut File);
    match whence {
        2 => file.seek(offset, SeekMode::Start),
        1 => file.seek(offset, SeekMode::Current),
        0 => file.seek(offset, SeekMode::End),
        _ => return -1,
    };
    0
}

#[no_mangle]
pub unsafe extern "C" fn fread(
    ptr: *mut c_void,
    size: usize,
    count: usize,
    stream: *mut c_void,
) -> i32 {
    match stream as usize {
        STDOUT | STDERR => {
            unimplemented!("fread from stdout or stderr");
        }
        _ => {
            let file = &mut *(stream as *mut File);
            let buf = core::slice::from_raw_parts_mut(ptr as *mut u8, size * count);
            let read = file.read(buf);
            read as i32
        }
    }
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
