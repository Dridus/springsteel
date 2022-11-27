//! Provides [`ImpulseStream`], a [`Stream`] which outputs `()`s whenever it's triggered, for
//! example by a button click or similar "it happened" event with no additional detail.

use futures::stream::Stream;
use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

/// Inner state of an [`ImpulseStream`].
struct ImpulseStreamInner {
    /// How many impulse events are waiting to be dequeued
    pending: usize,
    waker_opt: Option<std::task::Waker>,
}

/// A infinite [`Stream`] implementation which generates `()`s whenever it's
/// [`trigger`ed](Self::trigger), such as in response to a button click.
///
/// A `ImpulseSteam` pends until [`trigger`](Self::trigger) is called, either directly or
/// indirectly via [`triggerer`](Self::triggerer), at which point the stream outputs a `()` and
/// pends again.
///
/// `ImpulseStream`s are intended to be used as event handlers for widgets, e.g.
///
/// ```
///    # use gtk::Button;
///    # use gtk::prelude::ButtonExt as _;
///    # use springsteel::impulse_stream::ImpulseStream;
///    # gtk::init().expect("gtk::init");
///    #
///    let events = ImpulseStream::new();
///    let button = Button::builder().label("Trigger").build();
///    button.connect_clicked(events.triggerer());
/// ```
///
/// `ImpulseStream`s are infinite. That is, they never yield `Ready(None)` from
/// [`poll_next`](Self::poll_next).
#[derive(Clone)]
pub struct ImpulseStream(Rc<RefCell<ImpulseStreamInner>>);

/// An [`ImpulseStream`] can be unpinned as its state is a reference counted pointer.
impl Unpin for ImpulseStream {}

impl ImpulseStream {
    /// Create a new `ImpulseStream`. Any poll will pend until [`trigger`](Self::trigger) is
    /// invoked, either directly or indirectly via [`triggerer`](Self::triggerer).
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(ImpulseStreamInner {
            pending: 0,
            waker_opt: None,
        })))
    }

    /// Trigger the `ImpulseStream`, making it become ready with `()`.
    pub fn trigger(&self) {
        let mut inner = self.0.borrow_mut();
        inner.pending += 1;
        if let Some(w) = inner.waker_opt.take() {
            w.wake();
        }
    }

    /// Make a closure which can be called with a single parameter of any reference type, ignoring
    /// that parameter and just calling [`trigger`](Self::trigger). Useful for e.g.
    /// [`Button::connect_clicked`](gtk::prelude::ButtonExt::connect_clicked).
    pub fn triggerer<A>(&self) -> impl for<'a> Fn(&'a A) {
        let inst = self.clone();
        move |_: &A| inst.trigger()
    }
}

impl Stream for ImpulseStream {
    type Item = ();

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<()>> {
        let mut inner = self.0.borrow_mut();
        if inner.pending > 0 {
            inner.pending -= 1;
            Poll::Ready(Some(()))
        } else {
            inner.waker_opt = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

