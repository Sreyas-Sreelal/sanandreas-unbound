#[macro_use]
mod helper;
pub mod auth;
mod timer;

use auth::{is_player_authenticated, Auth};
use helper::delayed_kick;
use mysql::Pool;
use omp::{events::Events, main, players::Player, register, types::colour::Colour};
use threadpool::ThreadPool;
use timer::Timer;

const VERSION: &str = env!("CARGO_PKG_VERSION");
struct SanAndreasUnbound {
    pub pool: ThreadPool,
}

impl SanAndreasUnbound {
    pub fn new(pool: ThreadPool) -> Self {
        SanAndreasUnbound { pool }
    }
}

impl Events for SanAndreasUnbound {
    fn on_player_connect(&mut self, player: Player) {
        player.send_client_message(
            Colour::from_rgba(0xFF000000),
            "Hey!! Welcome to San Andreas Unbound!!!",
        );
    }

    fn on_player_spawn(&mut self, player: Player) {
        if !is_player_authenticated(player) {
            player.send_client_message(
                Colour::from_rgba(0xFF000000),
                "You are kicked from server (Reason: Not loggedin) !!",
            );
            delayed_kick(self.pool.clone(), player);
        }
        let playerid = player.get_id();
        timer::Timer::set_timer(
            self.pool.clone(),
            5,
            true,
            Box::new(move || {
                if let Some(player) = Player::from_id(playerid) {
                    player.send_client_message(Colour::from_rgba(0x0000EE00), "Test message!!");
                }
            }),
        );
    }
}

fn on_player_login(player: Player) {
    player.send_client_message(Colour::from_rgba(0x00FF0000), "Logged in successfully!");
    player.spawn();
}

fn on_player_register(player: Player) {
    player.send_client_message(Colour::from_rgba(0x00FF0000), "Sucessfully registered.");
    player.spawn();
}

#[main]
fn entry() {
    let connection = Pool::new(include_str!("../mysql.config")).unwrap();
    let pool = ThreadPool::new(2);

    let auth_module = Auth::new(
        pool.clone(),
        connection.clone(),
        on_player_register,
        on_player_login,
    )
    .unwrap();
    register!(Timer::new());
    register!(auth_module);
    register!(SanAndreasUnbound::new(pool.clone()));

    log!("San Andreas Unbound v{VERSION} loaded");
}
