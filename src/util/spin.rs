use std::{pin::Pin, task::Poll};

use indicatif::ProgressBar;

pin_project_lite::pin_project! {
    #[doc(hidden)]
    pub struct SpinFuture<T> {
        #[pin]
        future: T,
        spinner: ProgressBar,
    }
}

impl<T> SpinFuture<T> {
    fn new(future: T, spinner: ProgressBar, interval: std::time::Duration) -> Self {
        spinner.enable_steady_tick(interval);
        Self { future, spinner }
    }
}

impl<T: Future> Future for SpinFuture<T> {
    type Output = T::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let me = self.project();
        if let Poll::Ready(output) = me.future.poll(cx) {
            me.spinner.finish();
            return Poll::Ready(output);
        }

        Poll::Pending
    }
}

pub trait SpinExt: Sized {
    fn spin_until_ready(self, bar: ProgressBar) -> SpinFuture<Self> {
        SpinFuture::new(self, bar, std::time::Duration::from_millis(100))
    }

    fn spin_until_ready_with(
        self,
        bar: ProgressBar,
        interval: std::time::Duration,
    ) -> SpinFuture<Self> {
        SpinFuture::new(self, bar, interval)
    }
}

impl<T: Future> SpinExt for T {}

pub async fn spin_until_ready<T: Future>(future: T, bar: ProgressBar) -> T::Output {
    SpinFuture::new(future, bar, std::time::Duration::from_millis(100)).await
}
