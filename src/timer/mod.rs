use std::{
    collections::BTreeSet,
    error::Error,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use omp::events::Events;
use threadpool::ThreadPool;

type CallBack = Box<dyn Fn() + Send + Sync>;

pub struct Timer {
    receiver: Receiver<Arc<CallBack>>,
    sender: Sender<Arc<CallBack>>,
    active_timers: Arc<Mutex<BTreeSet<i32>>>,
    pool: ThreadPool,
}

impl Timer {
    pub fn new() -> Result<Self, Box<dyn Error + 'static>> {
        let (sender, receiver) = channel();

        Ok(Self {
            receiver,
            sender,
            active_timers: Arc::new(Mutex::new(BTreeSet::new())),
            pool: ThreadPool::new(5),
        })
    }

    pub fn set_timer(&mut self, func: CallBack, repeating: bool, duration: Duration) -> i32 {
        let func = Arc::new(func);
        let active_timer = self.active_timers.clone();

        let id = active_timer.lock().unwrap().last().unwrap_or(&-1) + 1;
        active_timer.lock().unwrap().insert(id);
        let sender = self.sender.clone();
        if repeating {
            thread::spawn(move || {
                loop {
                    thread::sleep(duration);
                    if !active_timer.lock().unwrap().contains(&id) {
                        break;
                    }
                    let _ = sender.send(func.clone());
                }
                active_timer.lock().unwrap().remove(&id);
            });
        } else {
            self.pool.execute(move || {
                thread::sleep(duration);
                let _ = sender.send(func.clone());
                active_timer.lock().unwrap().remove(&id);
            });
        }
        id
    }

    pub fn kill_timer(&mut self, id: i32) {
        if let Ok(mut active_timers) = self.active_timers.lock() {
            if active_timers.contains(&id) {
                active_timers.remove(&id);
                return;
            }
        }

        omp::core::Log(&format!(
            "[WARNING] Tried to kill invalid Timer with ID: {id}"
        ));
    }
}

impl Events for Timer {
    fn on_tick(&mut self, _elapsed: i32) {
        for cb in self.receiver.try_iter() {
            cb();
        }
    }
}
