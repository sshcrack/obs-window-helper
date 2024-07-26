#[cfg(not(windows))]
compile_error!("This library only supports windows!");

#[cfg(not(target_pointer_width = "64"))]
compile_error!("compilation is only allowed for 64-bit targets");

mod game;
mod validators;
mod helper;
mod string_conv;
mod monitor;
mod window;

pub use game::*;
pub use helper::*;