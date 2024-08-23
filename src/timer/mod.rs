use std::{
    collections::HashMap,
    error::Error,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
};

use chrono::Duration;
use omp::events::Events;

type CallBack = Box<dyn Fn() + Send + Sync>;

pub struct Timer {
    receiver: Receiver<(usize, bool, Arc<CallBack>)>,
    sender: Sender<(usize, bool, Arc<CallBack>)>,
    active_timers: HashMap<usize, timer::Guard>,
    timer: timer::Timer,
}

impl Timer {
    pub fn new() -> Self {
        let (sender, receiver) = channel();

        Self {
            receiver,
            sender,
            active_timers: HashMap::new(),
            timer: timer::Timer::new(),
        }
    }

    pub fn set_timer(&mut self, func: CallBack, repeating: bool, duration: i64) -> usize {
        let func = Arc::new(func);
        let id = self.active_timers.len() + 1;
        let sender = self.sender.clone();
        if repeating {
            self.active_timers.insert(
                id,
                self.timer
                    .schedule_repeating(Duration::milliseconds(duration), move || {
                        let _ = sender.send((id, true, func.clone()));
                    }),
            );
        } else {
            self.active_timers.insert(
                id,
                self.timer
                    .schedule_with_delay(Duration::milliseconds(duration), move || {
                        let _ = sender.send((id, false, func.clone()));
                    }),
            );
        }
        id
    }

    pub fn kill_timer(&mut self, id: usize) {
        if self.active_timers.contains_key(&id) {
            self.active_timers.remove(&id);
            return;
        }

        omp::core::Log(&format!(
            "[WARNING] Tried to kill invalid Timer with ID: {id}"
        ));
    }
}

impl Events for Timer {
    fn on_tick(&mut self, _elapsed: i32) {
        for (id, repeating, cb) in self.receiver.try_iter() {
            cb();
            if !repeating {
                if self.active_timers.contains_key(&id) {
                    self.active_timers.remove(&id);
                    return;
                }
            }
        }
    }
}
