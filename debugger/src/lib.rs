mod error;
mod ptrace;

pub use error::{Error, Result};
pub use ptrace::{Pid, Ptrace, Status, ptrace, waitpid};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pid_creation() {
        // Test Pid creation from raw pid_t
        let pid = Pid::from_raw(123);
        assert_eq!(pid.0.as_raw(), 123);
        
        // Test From trait
        let pid2: Pid = 456i32.into();
        assert_eq!(pid2.0.as_raw(), 456);
    }

    #[test]
    fn test_ptrace_operations() {
        // Test that ptrace operations can be created
        let _traceme = Ptrace::TraceMe;
        let pid = Pid::from_raw(123);
        let _attach = Ptrace::Attach { pid };
        let _cont = Ptrace::Cont { pid };
        
        // We can't actually call ptrace without proper process setup,
        // but we can verify the enum variants work
    }
    
    #[test]
    fn test_status_variants() {
        // Test that Status enum variants can be created
        let _exited = Status::Exited { status: 0 };
        let _signaled = Status::Signaled { signal: 9, dumped: false };
        let _stopped = Status::Stopped { signal: 19, status: 19 };
        let _continued = Status::Continued;
    }
}
