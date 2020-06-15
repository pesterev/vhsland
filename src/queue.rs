use futures::stream::Stream;
use std::{
    collections::VecDeque,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Arc::new(Mutex::new(Inner {
        queue: VecDeque::new(),
        waker: None,
        closed: false,
    }));

    let sender = Sender(Arc::clone(&inner));
    let receiver = Receiver(inner);

    (sender, receiver)
}

struct Inner<T> {
    queue: VecDeque<T>,
    waker: Option<Waker>,
    closed: bool,
}

#[derive(Clone)]
pub struct Sender<T>(Arc<Mutex<Inner<T>>>);

impl<T> Sender<T> {
    pub fn push_front(&self, value: T) {
        let mut guard = self.0.lock().unwrap();
        if guard.closed {
            return;
        }
        guard.queue.push_front(value);

        if let Some(waker) = guard.waker.take() {
            waker.wake();
        }
    }

    pub fn push_back(&self, value: T) {
        let mut guard = self.0.lock().unwrap();
        if guard.closed {
            return;
        }
        guard.queue.push_back(value);

        if let Some(waker) = guard.waker.take() {
            waker.wake();
        }
    }
}

pub struct Receiver<T>(Arc<Mutex<Inner<T>>>);

impl<T> Receiver<T> {
    #[allow(dead_code)]
    pub fn close(&mut self) {
        let mut guard = self.0.lock().unwrap();
        guard.closed = true;
    }
}

impl<T> Stream for Receiver<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let mut guard = match self.0.try_lock() {
            Ok(guard) => guard,
            Err(_) => return Poll::Pending,
        };

        if guard.closed {
            return Poll::Ready(None);
        }

        if let Some(item) = guard.queue.pop_front() {
            return Poll::Ready(Some(item));
        }

        guard.waker.replace(cx.waker().clone());
        Poll::Pending
    }
}
