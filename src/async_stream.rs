use crate::yielder::Receiver;

use futures_core::{FusedStream, Stream};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

#[doc(hidden)]
#[derive(Debug)]
pub struct AsyncStream<T, U> {
    rx: Receiver<T>,
    done: bool,
    generator: U,
}

impl<T, U> AsyncStream<T, U> {
    #[doc(hidden)]
    pub fn new(rx: Receiver<T>, generator: U) -> AsyncStream<T, U> {
        AsyncStream {
            rx,
            done: false,
            generator,
        }
    }
}

impl<T, U> FusedStream for AsyncStream<T, U>
where
    U: Future<Output = ()>,
{
    fn is_terminated(&self) -> bool {
        self.done
    }
}

impl<T, U> Stream for AsyncStream<T, U>
where
    U: Future<Output = ()>,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        unsafe {
            let me = Pin::get_unchecked_mut(self);

            if me.done {
                return Poll::Ready(None);
            }

            let mut dst = None;
            let res = {
                let _enter = me.rx.enter(&mut dst);
                Pin::new_unchecked(&mut me.generator).poll(cx)
            };

            me.done = res.is_ready();

            if dst.is_some() {
                return Poll::Ready(dst.take());
            }

            if me.done {
                Poll::Ready(None)
            } else {
                Poll::Pending
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.done {
            (0, Some(0))
        } else {
            (0, None)
        }
    }
}
