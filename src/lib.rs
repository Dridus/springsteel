//! FRP-ish GTK applications using [`Future`](std::future::Future) and abstractions on top provided
//! by [`futures`](https://crates.io/crates/futures) such as [`Stream`](futures::stream::Stream)
//! together with linear-system-of-equations constraint solving layout using the Cassowary
//! constraint solving algorithm as implemented by [`casuarius`](https://crates.io/crates/casuarius).

#![warn(missing_docs)]

pub mod glib_future;
pub use glib_future::glib_run_future;

pub mod impulse_stream;
pub use impulse_stream::ImpulseStream;

