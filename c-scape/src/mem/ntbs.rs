//! Library routines working with Nul-Terminated Byte Sequences (NTBS).

use core::cell::SyncUnsafeCell;
use core::ffi::CStr;
use core::ptr;
use libc::{c_char, c_int, c_schar, malloc, memcpy};

use crate::sync_ptr::SyncMutPtr;

const NUL: c_char = 0;

#[no_mangle]
unsafe extern "C" fn stpcpy(mut d: *mut c_char, mut s: *const c_char) -> *mut c_char {
    libc!(libc::stpcpy(d, s));

    loop {
        *d = *s;

        if *d == NUL {
            break;
        }

        d = d.add(1);
        s = s.add(1);
    }

    d
}

#[no_mangle]
unsafe extern "C" fn stpncpy(
    mut d: *mut c_char,
    mut s: *const c_char,
    mut n: usize,
) -> *mut c_char {
    libc!(libc::stpncpy(d, s, n));

    while n > 0 {
        n -= 1;

        *d = *s;

        if *d == NUL {
            break;
        }

        d = d.add(1);
        s = s.add(1);
    }

    for _ in 0..n {
        *d = 0;
        d = d.add(1);
    }

    d
}

#[no_mangle]
unsafe extern "C" fn strcat(d: *mut c_char, s: *const c_char) -> *mut c_char {
    libc!(libc::strcat(d, s));

    strcpy(strchr(d, 0), s);
    d
}

#[no_mangle]
unsafe extern "C" fn strchr(s: *const c_char, c: c_int) -> *mut c_char {
    libc!(libc::strchr(s, c));

    let mut s = s as *mut c_char;
    loop {
        if *s == c as _ {
            return s;
        }
        if *s == NUL {
            break;
        }
        s = s.add(1);
    }

    ptr::null_mut()
}

#[no_mangle]
unsafe extern "C" fn strcmp(mut s1: *const c_char, mut s2: *const c_char) -> c_int {
    libc!(libc::strcmp(s1, s2));

    while *s1 != NUL && *s2 != NUL {
        if *s1 != *s2 {
            break;
        }

        s1 = s1.add(1);
        s2 = s2.add(1);
    }

    *s1 as c_schar as c_int - *s2 as c_schar as c_int
}

#[no_mangle]
unsafe extern "C" fn strcpy(d: *mut c_char, s: *const c_char) -> *mut c_char {
    libc!(libc::strcpy(d, s));

    stpcpy(d, s);
    d
}

#[no_mangle]
unsafe extern "C" fn strncpy(d: *mut c_char, s: *const c_char, n: usize) -> *mut c_char {
    libc!(libc::strncpy(d, s, n));

    stpncpy(d, s, n);
    d
}

#[no_mangle]
unsafe extern "C" fn strcspn(s: *const c_char, m: *const c_char) -> usize {
    libc!(libc::strspn(s, m));

    let mut w = s;
    while *w != NUL {
        let mut m = m;
        while *m != NUL {
            if *w == *m {
                break;
            }
            m = m.add(1);
        }

        if *m != NUL {
            break;
        }

        w = w.add(1);
    }

    w.offset_from(s) as usize
}

#[no_mangle]
unsafe extern "C" fn strdup(s: *const c_char) -> *mut c_char {
    libc!(libc::strdup(s));

    let len = libc::strlen(s);
    let d = malloc(len + 1);
    if !d.is_null() {
        memcpy(d, s.cast(), len + 1);
    }
    d.cast()
}

#[cfg(feature = "define-mem-functions")]
#[no_mangle]
unsafe extern "C" fn strlen(s: *const c_char) -> usize {
    libc!(libc::strlen(s));

    compiler_builtins::mem::strlen(s)
}

#[no_mangle]
unsafe extern "C" fn strncat(d: *mut c_char, mut s: *const c_char, mut n: usize) -> *mut c_char {
    libc!(libc::strncat(d, s, n));

    let mut w = strchr(d, 0);

    while n > 0 && *s != NUL {
        n -= 1;

        *w = *s;

        w = w.add(1);
        s = s.add(1);
    }
    *w = 0;

    d
}

#[no_mangle]
unsafe extern "C" fn strncmp(mut s1: *const c_char, mut s2: *const c_char, mut n: usize) -> c_int {
    libc!(libc::strncmp(s1, s2, n));

    loop {
        if n == 0 {
            return 0;
        }
        n -= 1;

        if *s1 != *s2 || *s1 == NUL {
            break;
        }

        s1 = s1.add(1);
        s2 = s2.add(1);
    }

    *s1 as c_schar as c_int - *s2 as c_schar as c_int
}

#[no_mangle]
unsafe extern "C" fn strndup(s: *const c_char, n: usize) -> *mut c_char {
    libc!(libc::strndup(s, n));

    let len = strnlen(s, n);
    let d = malloc(len + 1);
    if !d.is_null() {
        memcpy(d, s.cast(), len);
    }

    let ret = d.cast::<c_char>();
    *ret.add(len) = 0;
    ret
}

#[no_mangle]
unsafe extern "C" fn strnlen(s: *const c_char, mut n: usize) -> usize {
    libc!(libc::strnlen(s, n));

    let mut w = s;
    while n > 0 && *w != NUL {
        n -= 1;
        w = w.add(1);
    }

    w.offset_from(s) as usize
}

#[no_mangle]
unsafe extern "C" fn strpbrk(s: *const c_char, m: *const c_char) -> *mut c_char {
    libc!(libc::strpbrk(s, m));

    let s = s.add(strcspn(s, m)) as *mut _;

    if *s != NUL {
        return s;
    }

    ptr::null_mut()
}

#[no_mangle]
unsafe extern "C" fn strrchr(s: *const c_char, c: c_int) -> *mut c_char {
    libc!(libc::strrchr(s, c));

    let mut s = s as *mut c_char;
    let mut ret = ptr::null_mut::<c_char>();
    loop {
        s = strchr(s, c);
        if s.is_null() {
            break;
        }

        ret = s;
        s = s.add(1);
    }

    ret
}

#[no_mangle]
unsafe extern "C" fn strspn(s: *const c_char, m: *const c_char) -> usize {
    libc!(libc::strspn(s, m));

    let mut w = s;
    while *w != NUL {
        let mut m = m;
        while *m != NUL {
            if *w == *m {
                break;
            }
            m = m.add(1);
        }

        if *m == NUL {
            break;
        }

        w = w.add(1);
    }

    w.offset_from(s) as usize
}

#[no_mangle]
unsafe extern "C" fn strtok(s: *mut c_char, m: *const c_char) -> *mut c_char {
    libc!(libc::strtok(s, m));

    static STORAGE: SyncUnsafeCell<SyncMutPtr<c_char>> =
        SyncUnsafeCell::new(unsafe { SyncMutPtr::new(ptr::null_mut()) });

    strtok_r(s, m, SyncUnsafeCell::get(&STORAGE) as *mut *mut c_char)
}

#[no_mangle]
unsafe extern "C" fn strtok_r(
    s: *mut c_char,
    m: *const c_char,
    p: *mut *mut c_char,
) -> *mut c_char {
    libc!(libc::strtok_r(s, m, p));

    let mut s = if s.is_null() { *p } else { s };

    if s.is_null() {
        return ptr::null_mut();
    }

    s = s.add(strspn(s, m));
    if *s == NUL {
        *p = ptr::null_mut();
        return ptr::null_mut();
    }

    let t = s.add(strcspn(s, m));
    if *t != NUL {
        *t = NUL;
        *p = t.add(1);
    } else {
        *p = ptr::null_mut();
    }

    s
}

#[no_mangle]
unsafe extern "C" fn strcasecmp(mut s1: *const c_char, mut s2: *const c_char) -> c_int {
    libc!(libc::strcasecmp(s1, s2));

    while *s1 != NUL && *s2 != NUL {
        if libc::tolower(*s1 as c_schar as c_int) != libc::tolower(*s2 as c_schar as c_int) {
            break;
        }

        s1 = s1.add(1);
        s2 = s2.add(1);
    }

    libc::tolower(*s1 as c_schar as c_int) - libc::tolower(*s2 as c_schar as c_int)
}

#[no_mangle]
unsafe extern "C" fn strncasecmp(
    mut s1: *const c_char,
    mut s2: *const c_char,
    mut n: usize,
) -> c_int {
    libc!(libc::strncasecmp(s1, s2, n));

    loop {
        if n == 0 {
            return 0;
        }
        n -= 1;

        if libc::tolower(*s1 as c_schar as c_int) != libc::tolower(*s2 as c_schar as c_int)
            || *s1 == NUL
        {
            break;
        }

        s1 = s1.add(1);
        s2 = s2.add(1);
    }

    libc::tolower(*s1 as c_schar as c_int) - libc::tolower(*s2 as c_schar as c_int)
}

#[no_mangle]
unsafe extern "C" fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char {
    libc!(libc::strstr(haystack, needle));

    if *needle == 0 {
        return haystack as *mut c_char;
    }

    let mut haystack = haystack;
    loop {
        if *haystack == 0 {
            break;
        }
        let mut len = 0;
        for n in CStr::from_ptr(needle).to_bytes() {
            let h = *haystack.add(len);
            if h != *n as c_char {
                break;
            }
            len += 1;
        }
        if *needle.add(len) == 0 {
            return haystack as *mut c_char;
        }
        haystack = haystack.add(1);
    }
    ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn index(s: *const c_char, c: c_int) -> *mut c_char {
    //libc!(libc::index(s, c));

    libc::strchr(s, c)
}

#[no_mangle]
pub unsafe extern "C" fn rindex(s: *const c_char, c: c_int) -> *mut c_char {
    //libc!(libc::rindex(s, c));

    libc::strrchr(s, c)
}
