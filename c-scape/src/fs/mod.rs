mod access;
#[cfg(not(target_os = "wasi"))]
mod chmod;
mod dir;
mod fadvise;
mod fallocate;
mod fcntl;
mod flock;
mod inotify;
mod link;
mod lseek;
mod mkdir;
mod mknod;
mod open;
mod readlink;
mod realpath;
mod remove;
mod rename;
mod sendfile;
mod stat;
mod symlink;
mod sync;
mod truncate;
mod utime;
mod xattr;

use core::ffi::CStr;
use rustix::fd::BorrowedFd;
use rustix::fs::AtFlags;

use errno::{set_errno, Errno};
use libc::{c_char, c_int, c_uint};

use crate::convert_res;

#[cfg(any(target_os = "android", target_os = "linux"))]
#[no_mangle]
unsafe extern "C" fn statx(
    dirfd_: c_int,
    path: *const c_char,
    flags: c_int,
    mask: c_uint,
    stat_: *mut rustix::fs::Statx,
) -> c_int {
    libc!(libc::statx(dirfd_, path, flags, mask, checked_cast!(stat_)));

    if path.is_null() || stat_.is_null() {
        set_errno(Errno(libc::EFAULT));
        return -1;
    }

    let flags = AtFlags::from_bits(flags as _).unwrap();
    let mask = rustix::fs::StatxFlags::from_bits(mask).unwrap();
    match convert_res(rustix::fs::statx(
        BorrowedFd::borrow_raw(dirfd_),
        CStr::from_ptr(path.cast()),
        flags,
        mask,
    )) {
        Some(r) => {
            *stat_ = r;
            0
        }
        None => -1,
    }
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[no_mangle]
unsafe extern "C" fn copy_file_range(
    fd_in: c_int,
    off_in: *mut i64,
    fd_out: c_int,
    off_out: *mut i64,
    len: usize,
    flags: c_uint,
) -> isize {
    libc!(libc::copy_file_range(
        fd_in, off_in, fd_out, off_out, len, flags
    ));

    if fd_in == -1 || fd_out == -1 {
        set_errno(Errno(libc::EBADF));
        return -1;
    }
    assert_eq!(flags, 0);
    let off_in = if off_in.is_null() {
        None
    } else {
        Some(&mut *off_in.cast::<u64>())
    };
    let off_out = if off_out.is_null() {
        None
    } else {
        Some(&mut *off_out.cast::<u64>())
    };
    match convert_res(rustix::fs::copy_file_range(
        BorrowedFd::borrow_raw(fd_in),
        off_in,
        BorrowedFd::borrow_raw(fd_out),
        off_out,
        len,
    )) {
        Some(n) => n as _,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn chown(
    pathname: *const c_char,
    owner: libc::uid_t,
    group: libc::gid_t,
) -> c_int {
    libc!(libc::chown(pathname, owner, group));

    let pathname = CStr::from_ptr(pathname);
    let owner = Some(rustix::process::Uid::from_raw(owner));
    let group = Some(rustix::process::Gid::from_raw(group));
    match convert_res(rustix::fs::chown(pathname, owner, group)) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn lchown(
    pathname: *const c_char,
    owner: libc::uid_t,
    group: libc::gid_t,
) -> c_int {
    libc!(libc::lchown(pathname, owner, group));

    let pathname = CStr::from_ptr(pathname);
    let owner = Some(rustix::process::Uid::from_raw(owner));
    let group = Some(rustix::process::Gid::from_raw(group));
    let flags = rustix::fs::AtFlags::SYMLINK_NOFOLLOW;
    match convert_res(rustix::fs::chownat(
        rustix::fs::CWD,
        pathname,
        owner,
        group,
        flags,
    )) {
        Some(()) => 0,
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn fchown(fd: c_int, owner: libc::uid_t, group: libc::gid_t) -> c_int {
    libc!(libc::fchown(fd, owner, group));

    let fd = BorrowedFd::borrow_raw(fd);
    let owner = Some(rustix::process::Uid::from_raw(owner));
    let group = Some(rustix::process::Gid::from_raw(group));
    match convert_res(rustix::fs::fchown(fd, owner, group)) {
        Some(()) => 0,
        None => -1,
    }
}
