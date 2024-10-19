#[cfg(windows)]
mod dimensions;
mod fetch;

#[cfg(windows)]
pub use self::dimensions::*;
pub use self::fetch::*;
