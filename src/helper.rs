use omp::players::Player;
use threadpool::ThreadPool;

use crate::timer::Timer;

macro_rules! log {
    ($string:literal) => {
        omp::core::Log(&format!($string));
    };
    ($string:literal,$($args:expr),*) => {
        omp::core::Log(&format!($string,$($args),*));
    };
}

pub fn delayed_kick(pool: ThreadPool, player: Player) {
    let playerid = player.get_id();
    Timer::set_timer(
        pool,
        1,
        false,
        Box::new(move || {
            if let Some(player) = Player::from_id(playerid) {
                player.kick();
            }
        }),
    )
}
