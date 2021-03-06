pub use libc::dev_t;
pub use libc::stat as FileStat;

use std::fmt;
use std::mem;
use std::path::Path;
use libc::mode_t;
use errno::{SysResult, SysError, from_ffi};
use fcntl::Fd;
use utils::ToCStr;

mod ffi {
    use libc::{c_char, c_int, mode_t, dev_t};
    pub use libc::{stat, fstat};

    extern {
        pub fn mknod(pathname: *const c_char, mode: mode_t, dev: dev_t) -> c_int;
        pub fn umask(mask: mode_t) -> mode_t;
    }
}

bitflags!(
    flags SFlag: mode_t {
        const S_IFREG  = 0o100000,
        const S_IFCHR  = 0o020000,
        const S_IFBLK  = 0o060000,
        const S_IFIFO  = 0o010000,
        const S_IFSOCK = 0o140000
    }
);

bitflags! {
    flags Mode: mode_t {
        const S_IRWXU = 0o0700,
        const S_IRUSR = 0o0400,
        const S_IWUSR = 0o0200,
        const S_IXUSR = 0o0100,

        const S_IRWXG = 0o0070,
        const S_IRGRP = 0o0040,
        const S_IWGRP = 0o0020,
        const S_IXGRP = 0o0010,

        const S_IRWXO = 0o0007,
        const S_IROTH = 0o0004,
        const S_IWOTH = 0o0002,
        const S_IXOTH = 0o0001,

        const S_ISUID = 0o4000,
        const S_ISGID = 0o2000,
        const S_ISVTX = 0o1000,
    }
}

impl fmt::Debug for SFlag {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "SFlag {{ bits: {} }}", self.bits())
    }
}

pub fn mknod(path: &Path, kind: SFlag, perm: Mode, dev: dev_t) -> SysResult<()> {
    let res = unsafe { ffi::mknod(path.to_c_str().as_ptr(), kind.bits | perm.bits() as mode_t, dev) };
    from_ffi(res)
}

#[cfg(target_os = "linux")]
const MINORBITS: usize = 20;

#[cfg(target_os = "linux")]
pub fn mkdev(major: u64, minor: u64) -> dev_t {
    (major << MINORBITS) | minor
}

pub fn umask(mode: Mode) -> Mode {
    let prev = unsafe { ffi::umask(mode.bits() as mode_t) };
    Mode::from_bits(prev).expect("[BUG] umask returned invalid Mode")
}

pub fn stat(path: &Path) -> SysResult<FileStat> {
    let mut dst = unsafe { mem::uninitialized() };
    let res = unsafe { ffi::stat(path.to_c_str().as_ptr(), &mut dst as *mut FileStat) };

    if res < 0 {
        return Err(SysError::last());
    }

    Ok(dst)
}

pub fn fstat(fd: Fd) -> SysResult<FileStat> {
    let mut dst = unsafe { mem::uninitialized() };
    let res = unsafe { ffi::fstat(fd, &mut dst as *mut FileStat) };

    if res < 0 {
        return Err(SysError::last());
    }

    Ok(dst)
}
