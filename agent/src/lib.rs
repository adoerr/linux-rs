/// System call wrapper.
///
/// Wrapper around `poll_sys` calls that checks `errno` on failure.
#[macro_export]
macro_rules! syscall {
    ($fn: ident ( $($arg: expr),* $(,)* ) ) => {{
        #[allow(clippy::macro_metavars_in_unsafe)]
        let res = unsafe { poll_sys::$fn($($arg, )*) };

        if res == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(res)
        }
    }};
}
