use std::{
    future::Future,
    pin::Pin,
    task::{Context, RawWaker, RawWakerVTable, Waker},
};

/// Given a future `future` that we know will
/// **never await**, this function extacts its
/// output.
#[track_caller]
pub fn poll_once<R>(mut future: Pin<Box<dyn Future<Output = R> + '_>>) -> R {
    fn panic_if_called<T>(_: *const ()) -> T {
        panic!(
            "method from waker was unexpected called -- is a dada fn somehow awaiting something?"
        );
    }
    const VTABLE: &RawWakerVTable =
        &RawWakerVTable::new(panic_if_called, panic_if_called, panic_if_called, |_| ());
    let raw_waker = RawWaker::new(std::ptr::null(), VTABLE);
    let waker = unsafe { Waker::from_raw(raw_waker) };
    match future.as_mut().poll(&mut Context::from_waker(&waker)) {
        std::task::Poll::Ready(r) => r,
        std::task::Poll::Pending => panic!("got pending result from future in `poll_once`"),
    }
}
