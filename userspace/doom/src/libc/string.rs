use super::match_format;
use core::ffi::{CStr, VaList};

#[no_mangle]
pub unsafe extern "C" fn putchar(c: i32) {
    print!("{}", c as u8 as char);
}

#[no_mangle]
pub unsafe extern "C" fn puts(s: *const i8) {
    let s = CStr::from_ptr(s);
    println!("{}", s.to_str().unwrap());
}

#[no_mangle]
pub unsafe extern "C" fn strcmp(s1: *const i8, s2: *const i8) -> i32 {
    let s1 = CStr::from_ptr(s1);
    let s2 = CStr::from_ptr(s2);
    s1.to_str().unwrap().cmp(s2.to_str().unwrap()) as i32
}

#[no_mangle]
pub unsafe extern "C" fn strcasecmp(s1: *const i8, s2: *const i8) -> i32 {
    let s1 = CStr::from_ptr(s1);
    let s2 = CStr::from_ptr(s2);
    s1.to_str()
        .unwrap()
        .to_lowercase()
        .cmp(&s2.to_str().unwrap().to_lowercase()) as i32
}

#[no_mangle]
pub unsafe extern "C" fn strncpy(dst: *mut i8, src: *const i8, n: u32) -> *mut i8 {
    let dst_ptr = dst;
    let dst = core::slice::from_raw_parts_mut(dst as *mut u8, n as usize);
    let src = CStr::from_ptr(src).to_bytes();
    let len = core::cmp::min(dst.len(), src.len());
    dst[..len].copy_from_slice(&src[..len]);
    if len < dst.len() {
        dst[len] = 0;
    }
    dst_ptr
}

#[no_mangle]
pub unsafe extern "C" fn strrchr(s: *const i8, c: i32) -> *mut i8 {
    let s_ptr = s;
    let s = CStr::from_ptr(s).to_bytes();
    if c == 0 {
        return s_ptr.offset(s.len() as isize) as *mut i8;
    }

    let c = c as u8 as char;
    let mut last = core::ptr::null_mut();
    for (i, &b) in s.iter().enumerate() {
        if b as char == c {
            last = s.as_ptr().offset(i as isize) as *mut i8;
        }
    }
    last
}

#[no_mangle]
pub unsafe extern "C" fn printf(format: *const u8, mut ap: ...) -> i32 {
    let mut i = 0;
    let mut ap = ap.as_va_list();

    loop {
        let c = *format.offset(i);

        if c == 0 {
            break 1; // TODO: return number of characters printed
        }

        if c == b'%' {
            let c = *format.offset(i + 1);
            match_format(c, &mut ap, |args| print!("{}", args));

            i += 2;
        } else {
            print!("{}", c as char);
            i += 1;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn vsnprintf(s: *mut i8, n: u32, format: *const u8, mut ap: VaList) -> i32 {
    let n = n as usize;
    let buf = core::slice::from_raw_parts_mut(s as *mut u8, n);

    let mut format_i = 0;
    let mut slice_i = 0;

    loop {
        let c = *format.offset(format_i);

        if c == 0 {
            buf[slice_i] = 0;

            // println!("{}", CStr::from_ptr(s).to_str().unwrap());
            break slice_i as i32;
        }

        if c == b'%' {
            let c = *format.offset(format_i + 1);
            match_format(c, &mut ap, |args| {
                let fmt = args.to_string(); //TODO: avoid allocation
                let fmt = fmt.as_bytes();

                let slice = buf[slice_i..].as_mut();
                let len = core::cmp::min(slice.len(), fmt.len());
                slice[..len].copy_from_slice(&fmt[..len]);
                slice_i += len;
            });
            format_i += 2;
        } else {
            if slice_i < n {
                buf[slice_i] = c;
                slice_i += 1;
            }
            format_i += 1;
        }

        if slice_i == n {
            buf[slice_i - 1] = 0;

            // println!("{}", CStr::from_ptr(s).to_str().unwrap());
            break slice_i as i32;
        }
    }
}
