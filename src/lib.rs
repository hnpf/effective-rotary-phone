use futures::task::{self, ArcWake};
use futures::future::BoxFuture;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

// our shitty custom executor
pub struct Dumbexec {
    task_sender: Sender<Arc<Task>>,
    task_receiver: Receiver<Arc<Task>>,
}

impl Dumbexec {
    pub fn new() -> Self {
        let (task_sender, task_receiver) = mpsc::channel();
        Dumbexec {
            task_sender,
            task_receiver,
        }
    }

    pub fn spawn<F>(&self, future: F)
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let task = Arc::new(Task {
            future: Mutex::new(Some(Box::pin(future))),
            task_sender: self.task_sender.clone(),
        });
        let _ = self.task_sender.send(task);
    }

    pub fn run(&self) {
        while let Ok(task) = self.task_receiver.recv() {
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take() {
                let waker = task::waker_ref(&task);
                let context = &mut task::Context::from_waker(&waker);
                if future.as_mut().poll(context).is_pending() {
                    *future_slot = Some(future);
                }
            }
        }
    }
}

struct Task {
    future: Mutex<Option<BoxFuture<'static, ()>>>,
    task_sender: Sender<Arc<Task>>,
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let _ = arc_self.task_sender.send(arc_self.clone());
    }
}

// shitty block on a future but who cares about efficiency when you can just spin up threads
pub fn shitty_block_on<F, T>(future: F) -> T
where
    F: std::future::Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let executor = Dumbexec::new();
    let (tx, rx) = mpsc::channel();
    executor.spawn(async move {
        let result = future.await;
        let _ = tx.send(result);
    });
    thread::spawn(move || executor.run());
    rx.recv().unwrap()
}