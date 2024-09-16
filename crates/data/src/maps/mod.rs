#[cfg(windows)]
mod dimensions;
mod fetch;

#[cfg(windows)]
pub use dimensions::*;
pub use fetch::*;
