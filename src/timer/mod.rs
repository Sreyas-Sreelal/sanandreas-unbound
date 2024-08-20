use std::{
    collections::BTreeSet,
    error::Error,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, OnceLock,
    },
    thread,
    time::Duration,
};

use omp::events::Events;
use threadpool::ThreadPool;

type CallBack = Box<dyn Fn() + Send + Sync>;
static mut ACTIVE_TIMERS: OnceLock<BTreeSet<i32>> = OnceLock::new();
static TIMER_SENDER: OnceLock<Sender<Arc<CallBack>>> = OnceLock::new();
static TIMER_POOL: OnceLock<ThreadPool> = OnceLock::new();
pub struct Timer {
    receiver: Receiver<Arc<CallBack>>,
}

impl Timer {
    pub fn new() -> Result<Self, Box<dyn Error + 'static>> {
        if TIMER_SENDER.get().is_some() {
            return Err("Only Timer module instance is allowed".into());
        }
        unsafe { ACTIVE_TIMERS.get_or_init(BTreeSet::new) };

        let (sender, receiver) = channel();

        TIMER_POOL.get_or_init(|| ThreadPool::new(5));
        TIMER_SENDER.get_or_init(|| sender);

        Ok(Self { receiver })
    }

    pub fn set_timer(func: CallBack, repeating: bool, duration: Duration) -> Option<i32> {
        if let Some(pool) = TIMER_POOL.get() {
            let func = Arc::new(func);
            let id = if let Some(active_timers) = unsafe { ACTIVE_TIMERS.get_mut() } {
                let id = active_timers.last().unwrap_or(&-1) + 1;
                active_timers.insert(id);
                id
            } else {
                return None;
            };

            if repeating {
                thread::spawn(move || {
                    loop {
                        thread::sleep(duration);
                        if unsafe { !ACTIVE_TIMERS.get().unwrap().contains(&id) } {
                            break;
                        }
                        let _ = TIMER_SENDER.get().unwrap().send(func.clone());
                    }
                    unsafe { ACTIVE_TIMERS.get_mut().unwrap().remove(&id) };
                });
            } else {
                pool.execute(move || {
                    thread::sleep(duration);
                    let _ = TIMER_SENDER.get().unwrap().send(func.clone());
                    unsafe { ACTIVE_TIMERS.get_mut().unwrap().remove(&id) };
                });
            }
            Some(id)
        } else {
            None
        }
    }

    pub fn kill_timer(id: i32) {
        if let Some(active_timers) = unsafe { ACTIVE_TIMERS.get_mut() } {
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
