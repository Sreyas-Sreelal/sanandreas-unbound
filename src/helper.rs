use omp::players::Player;
use std::time::Duration;

use crate::timer::Timer;

macro_rules! log {
    ($string:literal) => {
        omp::core::Log(&format!($string));
    };
    ($string:literal,$($args:expr),*) => {
        omp::core::Log(&format!($string,$($args),*));
    };
}

pub fn delayed_kick(player: Player) {
    let playerid = player.get_id();
    Timer::set_timer(
        Box::new(move || {
            if let Some(player) = Player::from_id(playerid) {
                player.kick();
            }
        }),
        false,
        Duration::from_secs(1),
    );
}
