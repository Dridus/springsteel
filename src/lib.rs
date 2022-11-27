//! FRP-ish GTK applications using [`Future`](std::future::Future) and abstractions on top provided
//! by [`futures`](https://crates.io/crates/futures) such as [`Stream`](futures::stream::Stream).

#![warn(missing_docs)]

#[macro_use]
pub mod constraint_macros;

pub mod constraint_view;
pub use constraint_view::ConstraintView;

pub mod glib_future;
pub use glib_future::glib_run_future;

pub mod impulse_stream;
pub use impulse_stream::ImpulseStream;

