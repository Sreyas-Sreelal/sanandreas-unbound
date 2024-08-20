use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
    thread,
    time::Duration,
};

use omp::events::Events;
use threadpool::ThreadPool;

type CallBackFunction = Arc<Box<dyn Send + Sync + Fn() + 'static>>;

static mut SENDER: Option<Sender<CallBackFunction>> = None;

pub struct Timer {
    receiver: Receiver<CallBackFunction>,
    //timer_ids: Vec<u32>,
}

impl Events for Timer {
    fn on_tick(&mut self, _elapsed: i32) {
        for func in self.receiver.try_iter() {
            func();
        }
    }
}

impl Timer {
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        unsafe { SENDER = Some(sender.clone()) };
        Self {
            receiver, /* timer_ids:Vec::new() */
        }
    }
    pub fn set_timer(
        pool: ThreadPool,
        duration: u64,
        repeating: bool,
        func: Box<dyn Send + Sync + Fn() + 'static>,
    ) {
        if unsafe { SENDER.is_none() } {
            omp::core::Log(
                "[WARNING] Calling Timer::set_timer without registering module instance",
            );
            return;
        }
        let sender = unsafe { SENDER.as_ref().unwrap().clone() };

        pool.execute(move || {
            let rc = Arc::new(func);
            loop {
                thread::sleep(Duration::from_secs(duration));
                sender.send(rc.clone()).unwrap();
                if !repeating {
                    break;
                }
            }
        })
    }
}
