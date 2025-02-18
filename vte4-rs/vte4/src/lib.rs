// Take a look at the license at the top of the repository in the LICENSE file.

//! # VTE4 bindings
//!
//! This library contains safe Rust bindings for vte4 Gtk-4.0.

#![cfg_attr(feature = "dox", feature(doc_cfg))]

pub use ffi;
pub use gtk;

// no runtime to initialize
macro_rules! assert_initialized_main_thread {
    () => {};
}

// No-op
macro_rules! skip_assert_initialized {
    () => {};
}

mod auto;
pub use auto::*;

pub mod prelude;
pub use prelude::*;

mod pty;
mod terminal;
