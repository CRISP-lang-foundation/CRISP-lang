// Copyright 2026 Peter Leukanič
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! POSIX module for CRISP — opt-in via `use posix;`
//! All constants and functions live under the `posix` hash.
//!
//! Signals   : POSIX.1-2001 + Linux + BSD + Solaris
//! Errno     : Linux (glibc) + BSD + Solaris error codes
//! File flags: O_*, F_*, SEEK_*, S_* mode bits

use std::cell::RefCell;
use std::rc::Rc;

mod environment;
mod files;
mod permissions;
mod process;
mod resources;
mod signals;
mod syslog;
mod terminal;
mod time;
mod users;

pub use environment::*;
pub use files::*;
pub use process::*;
pub use users::*;

use crate::eval::Environment;
use crate::value::Value;

pub fn register(env: &mut Environment) {
    process::register(env);
    users::register(env);
    files::register(env);
    environment::register(env);
    signals::register(env);
    time::register(env);
    terminal::register(env);
    permissions::register(env);
    resources::register(env);
    syslog::register(env);
    register_constants(env);
}

fn register_constants(env: &mut Environment) {
    // ─────────────────────────────────────────────────────────────────
    //  SIGNALS  (POSIX.1-2001 + Linux + BSD + Solaris extras)
    // ─────────────────────────────────────────────────────────────────
    //
    //  Standard signals — guaranteed on all Unix
    //
    env.define("SIGHUP",    Value::Int(libc::SIGHUP as i64));     //  1  Hangup
    env.define("SIGINT",    Value::Int(libc::SIGINT as i64));     //  2  Interrupt (Ctrl-C)
    env.define("SIGQUIT",   Value::Int(libc::SIGQUIT as i64));    //  3  Quit (Ctrl-\)
    env.define("SIGILL",    Value::Int(libc::SIGILL as i64));     //  4  Illegal instruction
    env.define("SIGABRT",   Value::Int(libc::SIGABRT as i64));    //  6  Abort
    env.define("SIGFPE",    Value::Int(libc::SIGFPE as i64));     //  8  Floating-point exception
    env.define("SIGKILL",   Value::Int(libc::SIGKILL as i64));    //  9  Kill (uncatchable)
    env.define("SIGSEGV",   Value::Int(libc::SIGSEGV as i64));    // 11  Segmentation fault
    env.define("SIGPIPE",   Value::Int(libc::SIGPIPE as i64));    // 13  Broken pipe
    env.define("SIGALRM",   Value::Int(libc::SIGALRM as i64));    // 14  Alarm clock
    env.define("SIGTERM",   Value::Int(libc::SIGTERM as i64));    // 15  Termination
    env.define("SIGUSR1",   Value::Int(libc::SIGUSR1 as i64));    // 10  User-defined signal 1
    env.define("SIGUSR2",   Value::Int(libc::SIGUSR2 as i64));    // 12  User-defined signal 2
    env.define("SIGCHLD",   Value::Int(libc::SIGCHLD as i64));    // 17  Child status change
    env.define("SIGCONT",   Value::Int(libc::SIGCONT as i64));    // 18  Continue if stopped
    env.define("SIGSTOP",   Value::Int(libc::SIGSTOP as i64));    // 19  Stop (uncatchable)
    env.define("SIGTSTP",   Value::Int(libc::SIGTSTP as i64));    // 20  Terminal stop (Ctrl-Z)
    env.define("SIGTTIN",   Value::Int(libc::SIGTTIN as i64));    // 21  Background read
    env.define("SIGTTOU",   Value::Int(libc::SIGTTOU as i64));    // 22  Background write

    // POSIX.1-2001 real-time signals
    // Available on: Linux, all BSDs, macOS, Solaris/illumos
    #[cfg(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "macos",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "solaris"
    ))]
    {
        env.define("SIGBUS",    Value::Int(libc::SIGBUS as i64));     //  7  Bus error
        env.define("SIGTRAP",   Value::Int(libc::SIGTRAP as i64));    //  5  Trace/breakpoint trap
        env.define("SIGURG",    Value::Int(libc::SIGURG as i64));     // 23  Urgent socket condition
        env.define("SIGXCPU",   Value::Int(libc::SIGXCPU as i64));    // 24  CPU time limit exceeded
        env.define("SIGXFSZ",   Value::Int(libc::SIGXFSZ as i64));    // 25  File size limit exceeded
        env.define("SIGVTALRM", Value::Int(libc::SIGVTALRM as i64));  // 26  Virtual alarm clock
        env.define("SIGPROF",   Value::Int(libc::SIGPROF as i64));    // 27  Profiling timer expired
        env.define("SIGWINCH",  Value::Int(libc::SIGWINCH as i64));   // 28  Window size change
        env.define("SIGIO",     Value::Int(libc::SIGIO as i64));      // 29  I/O now possible
        env.define("SIGSYS",    Value::Int(libc::SIGSYS as i64));     // 31  Bad system call
    }

    // Linux-specific signals (not on BSD or Solaris)
    #[cfg(target_os = "linux")]
    {
        env.define("SIGSTKFLT", Value::Int(libc::SIGSTKFLT as i64));  // 16  Stack fault
        env.define("SIGPWR",    Value::Int(libc::SIGPWR as i64));     // 30  Power failure
    }

    // Solaris/illumos-specific signals
    #[cfg(target_os = "solaris")]
    {
        env.define("SIGWAITING", Value::Int(libc::SIGWAITING as i64)); // 32  LWPs waiting (Solaris)
        env.define("SIGLWP",     Value::Int(libc::SIGLWP as i64));     // 33  LWP signal (Solaris)
        env.define("SIGFREEZE",  Value::Int(libc::SIGFREEZE as i64));  // 34  Checkpoint freeze (Solaris)
        env.define("SIGTHAW",    Value::Int(libc::SIGTHAW as i64));    // 35  Checkpoint thaw (Solaris)
        env.define("SIGCANCEL",  Value::Int(libc::SIGCANCEL as i64));  // 36  Thread cancellation (Solaris)
        env.define("SIGLOST",    Value::Int(libc::SIGLOST as i64));    // 37  Resource lost (Solaris)
    }

    // ─────────────────────────────────────────────────────────────────
    //  ERRNO  (Linux glibc + BSD + Solaris error codes)
    // ─────────────────────────────────────────────────────────────────
    //
    //  General errors (POSIX.1-2001 — all platforms)
    //
    env.define("EPERM",           Value::Int(libc::EPERM as i64));           //  1  Operation not permitted
    env.define("ENOENT",          Value::Int(libc::ENOENT as i64));          //  2  No such file or directory
    env.define("ESRCH",           Value::Int(libc::ESRCH as i64));           //  3  No such process
    env.define("EINTR",           Value::Int(libc::EINTR as i64));           //  4  Interrupted system call
    env.define("EIO",             Value::Int(libc::EIO as i64));             //  5  I/O error
    env.define("ENXIO",           Value::Int(libc::ENXIO as i64));           //  6  No such device or address
    env.define("E2BIG",           Value::Int(libc::E2BIG as i64));           //  7  Argument list too long
    env.define("ENOEXEC",         Value::Int(libc::ENOEXEC as i64));         //  8  Exec format error
    env.define("EBADF",           Value::Int(libc::EBADF as i64));           //  9  Bad file descriptor
    env.define("ECHILD",          Value::Int(libc::ECHILD as i64));          // 10  No child processes
    env.define("EAGAIN",          Value::Int(libc::EAGAIN as i64));          // 11  Resource temporarily unavailable
    env.define("ENOMEM",          Value::Int(libc::ENOMEM as i64));          // 12  Cannot allocate memory
    env.define("EACCES",          Value::Int(libc::EACCES as i64));          // 13  Permission denied
    env.define("EFAULT",          Value::Int(libc::EFAULT as i64));          // 14  Bad address
    env.define("ENOTBLK",         Value::Int(libc::ENOTBLK as i64));         // 15  Block device required
    env.define("EBUSY",           Value::Int(libc::EBUSY as i64));           // 16  Device or resource busy
    env.define("EEXIST",          Value::Int(libc::EEXIST as i64));          // 17  File exists
    env.define("EXDEV",           Value::Int(libc::EXDEV as i64));           // 18  Cross-device link
    env.define("ENODEV",          Value::Int(libc::ENODEV as i64));          // 19  No such device
    env.define("ENOTDIR",         Value::Int(libc::ENOTDIR as i64));         // 20  Not a directory
    env.define("EISDIR",          Value::Int(libc::EISDIR as i64));          // 21  Is a directory
    env.define("EINVAL",          Value::Int(libc::EINVAL as i64));          // 22  Invalid argument
    env.define("ENFILE",          Value::Int(libc::ENFILE as i64));          // 23  Too many open files in system
    env.define("EMFILE",          Value::Int(libc::EMFILE as i64));          // 24  Too many open files
    env.define("ENOTTY",          Value::Int(libc::ENOTTY as i64));          // 25  Not a typewriter
    env.define("ETXTBSY",         Value::Int(libc::ETXTBSY as i64));         // 26  Text file busy
    env.define("EFBIG",           Value::Int(libc::EFBIG as i64));           // 27  File too large
    env.define("ENOSPC",          Value::Int(libc::ENOSPC as i64));          // 28  No space left on device
    env.define("ESPIPE",          Value::Int(libc::ESPIPE as i64));          // 29  Illegal seek
    env.define("EROFS",           Value::Int(libc::EROFS as i64));           // 30  Read-only file system
    env.define("EMLINK",          Value::Int(libc::EMLINK as i64));          // 31  Too many links
    env.define("EPIPE",           Value::Int(libc::EPIPE as i64));           // 32  Broken pipe
    env.define("EDOM",            Value::Int(libc::EDOM as i64));            // 33  Numerical argument out of domain
    env.define("ERANGE",          Value::Int(libc::ERANGE as i64));          // 34  Numerical result out of range

    // Extended errors (POSIX.1-2001 — all modern Unix)
    env.define("EDEADLK",         Value::Int(libc::EDEADLK as i64));         // 35  Resource deadlock avoided
    env.define("ENAMETOOLONG",    Value::Int(libc::ENAMETOOLONG as i64));    // 36  File name too long
    env.define("ENOLCK",          Value::Int(libc::ENOLCK as i64));          // 37  No locks available
    env.define("ENOSYS",          Value::Int(libc::ENOSYS as i64));          // 38  Function not implemented
    env.define("ENOTEMPTY",       Value::Int(libc::ENOTEMPTY as i64));       // 39  Directory not empty
    env.define("ELOOP",           Value::Int(libc::ELOOP as i64));           // 40  Too many levels of symlinks

    // Network / socket errors
    // Available on: Linux, all BSDs, macOS, Solaris/illumos
    #[cfg(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "macos",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "solaris"
    ))]
    {
        env.define("ENOTSOCK",        Value::Int(libc::ENOTSOCK as i64));        // 88  Socket operation on non-socket
        env.define("EDESTADDRREQ",    Value::Int(libc::EDESTADDRREQ as i64));    // 89  Destination address required
        env.define("EMSGSIZE",        Value::Int(libc::EMSGSIZE as i64));        // 90  Message too long
        env.define("EPROTOTYPE",      Value::Int(libc::EPROTOTYPE as i64));      // 91  Protocol wrong type for socket
        env.define("ENOPROTOOPT",     Value::Int(libc::ENOPROTOOPT as i64));     // 92  Protocol not available
        env.define("EPROTONOSUPPORT", Value::Int(libc::EPROTONOSUPPORT as i64)); // 93  Protocol not supported
        env.define("ESOCKTNOSUPPORT", Value::Int(libc::ESOCKTNOSUPPORT as i64)); // 94  Socket type not supported
        env.define("EOPNOTSUPP",      Value::Int(libc::EOPNOTSUPP as i64));      // 95  Operation not supported
        env.define("EPFNOSUPPORT",    Value::Int(libc::EPFNOSUPPORT as i64));    // 96  Protocol family not supported
        env.define("EAFNOSUPPORT",    Value::Int(libc::EAFNOSUPPORT as i64));    // 97  Address family not supported
        env.define("EADDRINUSE",      Value::Int(libc::EADDRINUSE as i64));      // 98  Address already in use
        env.define("EADDRNOTAVAIL",   Value::Int(libc::EADDRNOTAVAIL as i64));   // 99  Cannot assign requested address
        env.define("ENETDOWN",        Value::Int(libc::ENETDOWN as i64));        // 100 Network is down
        env.define("ENETUNREACH",     Value::Int(libc::ENETUNREACH as i64));     // 101 Network is unreachable
        env.define("ENETRESET",       Value::Int(libc::ENETRESET as i64));       // 102 Network dropped connection on reset
        env.define("ECONNABORTED",    Value::Int(libc::ECONNABORTED as i64));    // 103 Software caused connection abort
        env.define("ECONNRESET",      Value::Int(libc::ECONNRESET as i64));      // 104 Connection reset by peer
        env.define("ENOBUFS",         Value::Int(libc::ENOBUFS as i64));         // 105 No buffer space available
        env.define("EISCONN",         Value::Int(libc::EISCONN as i64));         // 106 Transport endpoint is already connected
        env.define("ENOTCONN",        Value::Int(libc::ENOTCONN as i64));        // 107 Transport endpoint is not connected
        env.define("ESHUTDOWN",       Value::Int(libc::ESHUTDOWN as i64));       // 108 Cannot send after transport endpoint shutdown
        env.define("ETIMEDOUT",       Value::Int(libc::ETIMEDOUT as i64));       // 110 Connection timed out
        env.define("ECONNREFUSED",    Value::Int(libc::ECONNREFUSED as i64));    // 111 Connection refused
        env.define("EHOSTDOWN",       Value::Int(libc::EHOSTDOWN as i64));       // 112 Host is down
        env.define("EHOSTUNREACH",    Value::Int(libc::EHOSTUNREACH as i64));    // 113 No route to host
        env.define("EALREADY",        Value::Int(libc::EALREADY as i64));        // 114 Operation already in progress
        env.define("EINPROGRESS",     Value::Int(libc::EINPROGRESS as i64));     // 115 Operation now in progress
    }

    // Solaris-specific errno values
    #[cfg(target_os = "solaris")]
    {
        env.define("ENOSTR",        Value::Int(libc::ENOSTR as i64));        // Streams: not a stream
        env.define("ENODATA",       Value::Int(libc::ENODATA as i64));       // No data available
        env.define("ETIME",         Value::Int(libc::ETIME as i64));         // Streams: timer expired
        env.define("ENOSR",         Value::Int(libc::ENOSR as i64));         // Streams: out of stream resources
        env.define("ENONET",        Value::Int(libc::ENONET as i64));        // Machine is not on the network
        env.define("ENOPKG",        Value::Int(libc::ENOPKG as i64));        // Package not installed
        env.define("EREMOTE",       Value::Int(libc::EREMOTE as i64));       // Too many levels of remote in path
        env.define("ENOLINK",       Value::Int(libc::ENOLINK as i64));       // Link has been severed
        env.define("EADV",          Value::Int(libc::EADV as i64));          // Advertise error
        env.define("ESRMNT",        Value::Int(libc::ESRMNT as i64));        // Srmount error
        env.define("ECOMM",         Value::Int(libc::ECOMM as i64));         // Communication error on send
        env.define("EPROTO",        Value::Int(libc::EPROTO as i64));        // Protocol error
        env.define("EMULTIHOP",     Value::Int(libc::EMULTIHOP as i64));     // Multihop attempted
        env.define("EDOTDOT",       Value::Int(libc::EDOTDOT as i64));       // RFS specific error
        env.define("EBADMSG",       Value::Int(libc::EBADMSG as i64));       // Not a data message
        env.define("EOVERFLOW",     Value::Int(libc::EOVERFLOW as i64));     // Value too large for data type
        env.define("ENOTUNIQ",      Value::Int(libc::ENOTUNIQ as i64));      // Name not unique on network
        env.define("EBADFD",        Value::Int(libc::EBADFD as i64));        // File descriptor in bad state
        env.define("EREMCHG",       Value::Int(libc::EREMCHG as i64));       // Remote address changed
        env.define("ELIBACC",       Value::Int(libc::ELIBACC as i64));       // Cannot access needed shared library
        env.define("ELIBBAD",       Value::Int(libc::ELIBBAD as i64));       // Accessing corrupted shared library
        env.define("ELIBSCN",       Value::Int(libc::ELIBSCN as i64));       // .lib section corrupt
        env.define("ELIBMAX",       Value::Int(libc::ELIBMAX as i64));       // Too many shared libraries
        env.define("ELIBEXEC",      Value::Int(libc::ELIBEXEC as i64));      // Cannot exec shared library directly
        env.define("EILSEQ",        Value::Int(libc::EILSEQ as i64));        // Illegal byte sequence
        env.define("ERESTART",      Value::Int(libc::ERESTART as i64));      // Interrupted system call restartable
        env.define("ESTRPIPE",      Value::Int(libc::ESTRPIPE as i64));      // Streams pipe error
    }

    // BSD / macOS extras
    #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
    {
        env.define("ENOATTR",      Value::Int(libc::ENOATTR as i64));       // Attribute not found
    }

    // ─────────────────────────────────────────────────────────────────
    //  FILE OPEN FLAGS  (fcntl.h / open(2))
    // ─────────────────────────────────────────────────────────────────
    //
    env.define("O_RDONLY",    Value::Int(libc::O_RDONLY as i64));    //  0  Read-only
    env.define("O_WRONLY",    Value::Int(libc::O_WRONLY as i64));    //  1  Write-only
    env.define("O_RDWR",      Value::Int(libc::O_RDWR as i64));      //  2  Read-write
    env.define("O_CREAT",     Value::Int(libc::O_CREAT as i64));     // 64  Create if not exists
    env.define("O_TRUNC",     Value::Int(libc::O_TRUNC as i64));     // 512 Truncate to zero
    env.define("O_APPEND",    Value::Int(libc::O_APPEND as i64));    // 1024 Append mode
    env.define("O_EXCL",      Value::Int(libc::O_EXCL as i64));      // 128 Exclusive create
    env.define("O_NONBLOCK",  Value::Int(libc::O_NONBLOCK as i64));  // Non-blocking
    env.define("O_SYNC",      Value::Int(libc::O_SYNC as i64));      // Synchronous writes
    env.define("O_DSYNC",     Value::Int(libc::O_DSYNC as i64));     // Synchronous data writes
    env.define("O_RSYNC",     Value::Int(libc::O_RSYNC as i64));     // Synchronous reads
    env.define("O_NOCTTY",    Value::Int(libc::O_NOCTTY as i64));    // Don't make controlling tty

    // Extended open flags
    // Available on: Linux, all BSDs, macOS, Solaris/illumos
    #[cfg(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "macos",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "solaris"
    ))]
    {
        env.define("O_CLOEXEC",   Value::Int(libc::O_CLOEXEC as i64));   // Close-on-exec
        env.define("O_DIRECTORY", Value::Int(libc::O_DIRECTORY as i64)); // Must be a directory
        env.define("O_NOFOLLOW",  Value::Int(libc::O_NOFOLLOW as i64));  // Don't follow symlinks
    }

    // Linux-only open flags
    #[cfg(target_os = "linux")]
    {
        env.define("O_NOATIME",   Value::Int(libc::O_NOATIME as i64));   // Don't update atime
        env.define("O_PATH",      Value::Int(libc::O_PATH as i64));      // Open for path operations only
        env.define("O_TMPFILE",   Value::Int(libc::O_TMPFILE as i64));   // Unnamed temp file
    }

    // Solaris-specific open flags
    #[cfg(target_os = "solaris")]
    {
        env.define("O_XATTR",     Value::Int(0x4000_i64));               // Open extended attribute
        env.define("O_SEARCH",    Value::Int(0x200000_i64));             // Open for search only
        env.define("O_EXEC",      Value::Int(0x400000_i64));             // Open for execute only
    }

    // ─────────────────────────────────────────────────────────────────
    //  FILE MODE BITS  (sys/stat.h)
    // ─────────────────────────────────────────────────────────────────
    //
    env.define("S_IRWXU",  Value::Int(libc::S_IRWXU as i64));   // 0700  Owner rwx
    env.define("S_IRUSR",  Value::Int(libc::S_IRUSR as i64));   // 0400  Owner read
    env.define("S_IWUSR",  Value::Int(libc::S_IWUSR as i64));   // 0200  Owner write
    env.define("S_IXUSR",  Value::Int(libc::S_IXUSR as i64));   // 0100  Owner execute
    env.define("S_IRWXG",  Value::Int(libc::S_IRWXG as i64));   // 0070  Group rwx
    env.define("S_IRGRP",  Value::Int(libc::S_IRGRP as i64));   // 0040  Group read
    env.define("S_IWGRP",  Value::Int(libc::S_IWGRP as i64));   // 0020  Group write
    env.define("S_IXGRP",  Value::Int(libc::S_IXGRP as i64));   // 0010  Group execute
    env.define("S_IRWXO",  Value::Int(libc::S_IRWXO as i64));   // 0007  Others rwx
    env.define("S_IROTH",  Value::Int(libc::S_IROTH as i64));   // 0004  Others read
    env.define("S_IWOTH",  Value::Int(libc::S_IWOTH as i64));   // 0002  Others write
    env.define("S_IXOTH",  Value::Int(libc::S_IXOTH as i64));   // 0001  Others execute
    env.define("S_ISUID",  Value::Int(libc::S_ISUID as i64));   // 4000  Set-user-ID
    env.define("S_ISGID",  Value::Int(libc::S_ISGID as i64));   // 2000  Set-group-ID
    env.define("S_ISVTX",  Value::Int(libc::S_ISVTX as i64));   // 1000  Sticky bit

    // ─────────────────────────────────────────────────────────────────
    //  SEEK WHENCE  (unistd.h / lseek(2))
    // ─────────────────────────────────────────────────────────────────
    //
    env.define("SEEK_SET",  Value::Int(libc::SEEK_SET as i64));  // 0  From beginning
    env.define("SEEK_CUR",  Value::Int(libc::SEEK_CUR as i64));  // 1  From current
    env.define("SEEK_END",  Value::Int(libc::SEEK_END as i64));  // 2  From end

    // SEEK_DATA/SEEK_HOLE originated on Solaris, now also on Linux/BSD
    #[cfg(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "solaris"
    ))]
    {
        env.define("SEEK_DATA", Value::Int(libc::SEEK_DATA as i64)); // Next data
        env.define("SEEK_HOLE", Value::Int(libc::SEEK_HOLE as i64)); // Next hole
    }

    // ─────────────────────────────────────────────────────────────────
    //  fcntl(2) command constants
    // ─────────────────────────────────────────────────────────────────
    //
    #[cfg(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "macos",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "solaris"
    ))]
    {
        env.define("F_DUPFD",       Value::Int(libc::F_DUPFD as i64));       // Duplicate fd
        env.define("F_GETFD",       Value::Int(libc::F_GETFD as i64));       // Get file descriptor flags
        env.define("F_SETFD",       Value::Int(libc::F_SETFD as i64));       // Set file descriptor flags
        env.define("F_GETFL",       Value::Int(libc::F_GETFL as i64));       // Get file status flags
        env.define("F_SETFL",       Value::Int(libc::F_SETFL as i64));       // Set file status flags
        env.define("F_GETLK",       Value::Int(libc::F_GETLK as i64));       // Get lock
        env.define("F_SETLK",       Value::Int(libc::F_SETLK as i64));       // Set lock (non-blocking)
        env.define("F_SETLKW",      Value::Int(libc::F_SETLKW as i64));      // Set lock (blocking)
        env.define("FD_CLOEXEC",    Value::Int(libc::FD_CLOEXEC as i64));    // Close-on-exec flag
    }

    // ─────────────────────────────────────────────────────────────────
    //  waitpid(2) option flags
    // ─────────────────────────────────────────────────────────────────
    //
    env.define("WNOHANG",     Value::Int(libc::WNOHANG as i64));     // Return immediately if no child
    env.define("WUNTRACED",   Value::Int(libc::WUNTRACED as i64));   // Also return for stopped children

    #[cfg(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "macos",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "solaris"
    ))]
    {
        env.define("WCONTINUED",  Value::Int(libc::WCONTINUED as i64));  // Also return for continued children
    }

    // Solaris-specific wait flags
    #[cfg(target_os = "solaris")]
    {
        env.define("WNOWAIT",    Value::Int(libc::WNOWAIT as i64));    // Leave child waitable
        env.define("WEXITED",    Value::Int(0x0002_i64));              // Wait for exited children
        env.define("WTRAPPED",   Value::Int(0x0004_i64));              // Wait for trapped children (Solaris)
    }

    // ─────────────────────────────────────────────────────────────────
    //  Standard file descriptors
    // ─────────────────────────────────────────────────────────────────
    //
    env.define("STDIN_FILENO",  Value::Int(0));
    env.define("STDOUT_FILENO", Value::Int(1));
    env.define("STDERR_FILENO", Value::Int(2));
}
