use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use thiserror::Error;
use tokio::sync::Notify;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ChannelError {
    #[error("sending end of this channel has closed")]
    SenderClosed,
    #[error("receiving end of this channel has closed")]
    ReceiverClosed,
    #[error("internal lock was poisoned")]
    PoisonError,
}

pub fn latest_message_channel<T>() -> (Sender<T>, Receiver<T>) {
    let value = Arc::new(Mutex::new(None));
    let notify = Arc::new(Notify::new());
    let both_alive = Arc::new(AtomicBool::new(true));

    let sender = Sender {
        value: Arc::clone(&value),
        notify: Arc::clone(&notify),
        both_alive: Arc::clone(&both_alive),
    };
    let receiver = Receiver {
        value,
        notify,
        both_alive,
    };
    (sender, receiver)
}

pub struct Sender<T> {
    value: Arc<Mutex<Option<T>>>,
    notify: Arc<Notify>,
    both_alive: Arc<AtomicBool>,
}

impl<T> Sender<T> {
    pub fn send(&self, value: T) -> Result<(), ChannelError> {
        if !self.both_alive.load(Ordering::SeqCst) {
            Err(ChannelError::ReceiverClosed)
        } else {
            if let Ok(mut mutex_guard) = self.value.lock() {
                *mutex_guard = Some(value);
            } else {
                return Err(ChannelError::PoisonError);
            }
            self.notify.notify_one();
            Ok(())
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        self.both_alive.store(false, Ordering::SeqCst);
        self.notify.notify_waiters()
    }
}

pub struct Receiver<T> {
    value: Arc<Mutex<Option<T>>>,
    notify: Arc<Notify>,
    both_alive: Arc<AtomicBool>,
}

impl<T> Receiver<T> {
    pub async fn recv(&self) -> Result<T, ChannelError> {
        if !self.both_alive.load(Ordering::SeqCst) {
            return Err(ChannelError::SenderClosed);
        }
        self.notify.notified().await;
        if !self.both_alive.load(Ordering::SeqCst) {
            return Err(ChannelError::SenderClosed);
        }
        if let Ok(content) = &mut self.value.lock() {
            Ok(content
                .take()
                .expect("latest_message_channel woken up but empty."))
        } else {
            Err(ChannelError::PoisonError)
        }
    }

    pub fn try_recv(&self) -> Result<Option<T>, ChannelError> {
        if !self.both_alive.load(Ordering::SeqCst) {
            return Err(ChannelError::SenderClosed);
        }
        if let Ok(content) = &mut self.value.lock() {
            Ok(content.take())
        } else {
            Err(ChannelError::PoisonError)
        }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.both_alive.store(false, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn send_single_message() {
        let (tx, rx) = latest_message_channel();
        tx.send("hi").unwrap();
        assert_eq!(rx.recv().await.unwrap(), "hi");
    }

    #[tokio::test]
    async fn only_receive_last_message() {
        let (tx, rx) = latest_message_channel();
        tx.send(1).unwrap();
        tx.send(2).unwrap();
        tx.send(3).unwrap();
        assert_eq!(rx.recv().await.unwrap(), 3);
        assert_eq!(rx.try_recv().unwrap(), None);
    }

    #[tokio::test]
    async fn future_await_until_sent() {
        let (tx, rx) = latest_message_channel();
        tokio::spawn(async move {
            assert_eq!(rx.recv().await.unwrap(), 1);
            assert_eq!(rx.recv().await.unwrap(), 2);
            assert_eq!(rx.recv().await.unwrap(), 3);
        });
        tx.send(1).unwrap();
        tx.send(2).unwrap();
        tx.send(3).unwrap();
    }

    #[test]
    fn send_try_recv_single_message() {
        let (tx, rx) = latest_message_channel();
        tx.send("hi").unwrap();
        assert_eq!(rx.try_recv().unwrap(), Some("hi"));
    }

    #[test]
    fn try_recv_empty_message() {
        let (_tx, rx) = latest_message_channel::<i32>();
        assert_eq!(rx.try_recv().unwrap(), None);
    }

    #[test]
    fn fail_to_send_with_dead_receiver() {
        let (tx, rx) = latest_message_channel();
        drop(rx);
        assert_eq!(tx.send("hi"), Err(ChannelError::ReceiverClosed));
    }

    #[test]
    fn fail_to_receive_with_dead_sender() {
        let (tx, rx) = latest_message_channel::<i32>();
        drop(tx);
        assert_eq!(rx.try_recv(), Err(ChannelError::SenderClosed));
    }
}
