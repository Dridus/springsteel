//! Provides a [`Future`] executor which runs in the glib main loop, suitable for doing GTK UI
//! side effects: [`glib_run_future`].

use glib::source::{idle_add_local, Continue, SourceId};
use std::boxed::Box;
use std::future::Future;
use std::mem::drop;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

/// Internal state for the glib executor, with the state of the future computation along with
/// scheduling details.
struct GlibWaker {
    /// Contains the future being iterated.
    fut: Box<dyn Future<Output = ()> + Unpin>,

    /// Contains the `Some(`[`SourceId`]`)` of the scheduled idle callback step or `None` if no 
    /// step is presently scheduled.
    pending_idle_opt: Option<SourceId>,
}

/// Run a given future on the glib main loop until it becomes `Ready`.
///
/// Because this is running on the glib main loop, it's especially imperative for the
/// responsiveness of the user interface that the future never blocks but instead always pends.
pub fn glib_run_future<F>(fut: F)
where
    F: Future<Output = ()> + Unpin + 'static,
{
    glib_waker_schedule(&Arc::new(Mutex::new(GlibWaker {
        fut: Box::new(fut),
        pending_idle_opt: None,
    })))
}

/// Given a [`GlibWaker`] state pointer, make the [`RawWaker`] instance by untyping the pointer
/// and supplying the vtable.
fn glib_raw_waker(arc: Arc<Mutex<GlibWaker>>) -> RawWaker {
    let raw_waker = RawWaker::new(Arc::into_raw(arc).cast(), &GLIB_WAKER_VTABLE);
    raw_waker
}

/// Given a [`GlibWaker`] state pointer, make a [`Waker`] instance.
fn glib_waker(arc: Arc<Mutex<GlibWaker>>) -> Waker {
    // SAFETY: "The behavior of the returned Waker is undefined if the contract defined in
    // RawWaker's and RawWakerVTable's documentation is not upheld."
    //
    // Individual obligations are discharged in each of the vtable functions, below.

    // SAFETY: "Implements Clone, Send, and Sync; therefore, a waker may be invoked from any
    // thread, including ones not in any way managed by the executor."
    //
    // The data is retained in an Arc<Mutex<..>> so that it is safely counted in a multithreaded
    // context and access to the underlying future state and schedule is only updated
    // single-threaded.
    //
    // The future state is also only ever permuted inside glib_waker_step, which should only ever
    // be run on the glib main loop thread, meaning that all future effects are on that thread
    // unless a future specifically spawns or causes to be spawned a separate thread. This makes it
    // difficult to accidentally do GTK/GDK/GLIB things not on the main thread.

    let waker = unsafe { Waker::from_raw(glib_raw_waker(arc)) };
    waker
}

/// Implement [`RawWakerVTable`] `clone` function by increasing the strong reference count of the
/// state pointer.
unsafe fn glib_waker_clone(arc_ptr: *const ()) -> RawWaker {
    // SAFETY: "the implementation of this function must retain all resources that are required for
    // this additional instance"
    //
    // A strong count is present on entry, by contract with RawWaker, so we add one for the new
    // RawWaker.

    // SAFETY: "calling wake on the resulting RawWaker should result in a wakeup of the same
    // task that would have been awoken by the original RawWaker"
    //
    // The resulting RawWaker is exactly the same as the new one, modulo type conversions, with one
    // more strong reference.

    Arc::increment_strong_count(arc_ptr);
    let arc = Arc::from_raw(arc_ptr.cast::<Mutex<GlibWaker>>());
    let raw_waker = glib_raw_waker(arc);
    raw_waker
}

/// Make the closure that should be run during the idle callback. A closure-making separate
/// function just to make it a top level notion.
fn glib_waker_step(arc: Arc<Mutex<GlibWaker>>) -> impl Fn() -> Continue {
    move || {
        let mut inner = arc.lock().unwrap();
        let waker = glib_waker(arc.clone());
        let poll = Pin::new(&mut inner.fut).poll(&mut Context::from_waker(&waker));
        let cont = matches!(poll, Poll::Ready(()));
        if !cont {
            inner.pending_idle_opt = None;
        }
        Continue(cont)
    }
}

/// Ensure that the idle callback is scheduled, because either the waker was triggered indicating
/// that there's work waiting to do, or from the initial step to get things as pending as possible.
fn glib_waker_schedule(arc: &Arc<Mutex<GlibWaker>>) {
    let mut inner = arc.lock().unwrap();
    if inner.pending_idle_opt.is_some() {
        return;
    }

    inner.pending_idle_opt = Some(idle_add_local(glib_waker_step(arc.clone())));
}

/// Implement [`RawWakerVTable`] `wake` function by triggering a wake via
/// [`glib_waker_schedule`], then dropping the waker.
unsafe fn glib_waker_wake(arc_ptr: *const ()) {
    // SAFETY: "the implementation of this function must release any resources that are associated
    // with this RawWaker"
    //
    // We convert the raw pointer into an Arc, retaining the same strong count, then drop the Arc
    // which decrements the count. We don't hold any other resources.
    //
    // glib_waker_schedule will increment the reference count so it ensures that the resources live
    // until the idle callback.

    let arc = Arc::from_raw(arc_ptr.cast::<Mutex<GlibWaker>>());
    glib_waker_schedule(&arc);
    drop(arc);
}

/// Implement [`RawWakerVTable`] `wake_by_ref` function by triggering a wake via
/// [`glib_waker_schedule`].
unsafe fn glib_waker_wake_by_ref(arc_ptr: *const ()) {
    // SAFETY: "this function is similar to wake, but but not consume the provided data pointer"
    //
    // We convert the raw pointer into an Arc, retaining the same strong count, then increase it by
    // one so that when the Arc is dropped and decreases the strong count, the overall refernce
    // count remains the same.
    //
    // glib_waker_schedule will increment the reference count so it ensures that the resources live
    // until the idle callback.

    let arc = Arc::from_raw(arc_ptr.cast::<Mutex<GlibWaker>>());
    Arc::increment_strong_count(arc_ptr);
    glib_waker_schedule(&arc);
}

/// Implement [`RawWakerVTable`] `drop` function by decrementing the state strong reference count.
unsafe fn glib_waker_drop(arc_ptr: *const ()) {
    // SAFETY: "the implementation of this function must release any resources that are associated
    // with this instance of a RawWaker"
    //
    // We convert the raw pointer into an Arc, retaining the same strong count, then drop the
    // Arc which decrements the strong count.

    let arc = Arc::from_raw(arc_ptr.cast::<Mutex<GlibWaker>>());
    drop(arc);
}

/// The [`RawWakerVTable`] that implements a [`RawWaker`]/[`Waker`] which schedules on the glib
/// main loop via [`idle_add_local`].
static GLIB_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
    glib_waker_clone,
    glib_waker_wake,
    glib_waker_wake_by_ref,
    glib_waker_drop,
);
