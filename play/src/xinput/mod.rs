
#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;

#[cfg(not(windows))]
mod dummy;
#[cfg(not(windows))]
pub use dummy::*;
