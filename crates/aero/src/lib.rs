#![allow(clippy::too_many_arguments)]
#![allow(clippy::manual_clamp)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::assign_op_pattern)]

pub mod barrowman;
pub mod body_flaps;
pub mod compute;
pub mod interference;
pub mod ring_fins;
pub mod supersonic;
pub mod types;

pub use barrowman::*;
pub use body_flaps::*;
pub use compute::*;
pub use interference::*;
pub use ring_fins::*;
pub use supersonic::*;
pub use types::*;
