//! Process exit codes for agents and scripts.

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Config used when Phase 2+ validates project state
pub enum ExitCode {
    Success = 0,
    Failure = 1,
    /// Subcommand exists but is not implemented yet (Phase stub) / missing inputs.
    NotImplemented = 2,
    /// Policy failure (e.g. `--require-sig` unmet) or configuration problem.
    Policy = 3,
}
