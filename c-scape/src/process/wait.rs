//use errno::{set_errno, Errno};
use libc::c_int;
use rustix::process::{Pid, WaitOptions};

use crate::convert_res;

#[no_mangle]
unsafe extern "C" fn waitpid(pid: c_int, status: *mut c_int, options: c_int) -> c_int {
    libc!(libc::waitpid(pid, status, options));
    let options = WaitOptions::from_bits(options as _).unwrap();
    let ret_pid;
    let ret_status;
    match pid {
        -1 => match convert_res(rustix::process::wait(options)) {
            Some(Some((new_pid, new_status))) => {
                ret_pid = new_pid.as_raw_nonzero().get() as c_int;
                ret_status = new_status.as_raw() as c_int;
            }
            Some(None) => return 0,
            None => return -1,
        },
        // FIXME: This is a temporary workaround for rustix not supporting
        // waitpid on process groups.
        pid if pid < 0 => match convert_res(rustix::process::wait(options)) {
            Some(Some((new_pid, new_status))) => {
                ret_pid = new_pid.as_raw_nonzero().get() as c_int;
                ret_status = new_status.as_raw() as c_int;
            }
            Some(None) => return 0,
            None => return -1,
        },
        // FIXME: Use this version instead.
        /*
        // TODO: Give rustix better APIs for waiting on process groups.
        pid if pid < 0 => match convert_res(rustix::process::waitpgid(
            Pid::from_raw_unchecked(pid.wrapping_neg()),
            options,
        )) {
            Some(Some(new_status)) => {
                ret_pid = if pid == 0 {
                    rustix::process::getpid().as_raw_nonzero().get() as c_int
                } else {
                    pid
                };
                ret_status = new_status.as_raw() as c_int;
            }
            Some(None) => return 0,
            None => return -1,
        },
        */
        pid => match convert_res(rustix::process::waitpid(Pid::from_raw(pid as _), options)) {
            Some(Some(new_status)) => {
                ret_pid = if pid == 0 {
                    rustix::process::getpid().as_raw_nonzero().get() as c_int
                } else {
                    pid
                };
                ret_status = new_status.as_raw() as c_int;
            }
            Some(None) => return 0,
            None => return -1,
        },
    }
    if !status.is_null() {
        status.write(ret_status);
    }
    ret_pid
}
