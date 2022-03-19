use std::sync::{mpsc::{SendError, Sender, channel}, Mutex};

use once_cell::sync::Lazy;
use rayon::ThreadPool;

use super::action::Action;

pub static GLOBAL_ACTOR: Lazy<Mutex<Actor>> = Lazy::new(|| {
    Mutex::new(Actor::new())
});

static THREAD_POOL: Lazy<ThreadPool> = Lazy::new(|| { rayon::ThreadPoolBuilder::new().num_threads(num_cpus::get()).build().unwrap() });

/// 动作执行者
/// 注意：不是 Actor 设计模式
pub struct Actor {
    tx: Sender<Box<Action>>,
}

impl Actor {
    pub fn new() -> Actor {
        let (sender, receiver) = channel();
        let actor = Actor {
            tx: sender
        };
        std::thread::spawn(move || {
            loop {
                match receiver.recv() {
                    Ok(action) => {
                        THREAD_POOL.spawn(move || action.execute());
                    },
                    Err(error) => {
                        if error.to_string().contains("closed channel") {
                            break;
                        }
                        println!("{}", error);
                    }
                }
            }
        });
        actor
    }

    pub fn add_action(&mut self, action: Box<Action>) -> Result<(), SendError<Box<Action>>> {
        self.tx.send(action)
    }
}

/// 执行动作
/// @param action 动作
pub fn act(action: Box<Action>) {
    let mut actor = GLOBAL_ACTOR.lock().unwrap();
    match actor.add_action(action) {
        Ok(_) => {},
        Err(error) => {
            println!("{}", error);
        }
    }
}