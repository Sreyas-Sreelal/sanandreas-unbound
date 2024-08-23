#[macro_use]
mod helper;
pub mod auth;
pub mod timer;

use std::{cell::RefCell, rc::Rc};

use auth::Auth;
use mysql::Pool;
use omp::{events::Events, main, players::Player, register, types::colour::Colour};
use threadpool::ThreadPool;
use timer::Timer;

const VERSION: &str = env!("CARGO_PKG_VERSION");
struct SanAndreasUnbound {
    timer: Rc<RefCell<Timer>>,
    auth: Rc<RefCell<Auth>>,
}

impl SanAndreasUnbound {
    pub fn new(timer: Rc<RefCell<Timer>>, auth: Rc<RefCell<Auth>>) -> Self {
        SanAndreasUnbound { timer, auth }
    }
    pub fn delayed_kick(&mut self, player: Player) {
        let playerid = player.get_id();
        self.timer.borrow_mut().set_timer(
            Box::new(move || {
                if let Some(player) = Player::from_id(playerid) {
                    player.kick();
                }
            }),
            false,
            1000,
        );
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
        if !self.auth.borrow_mut().is_player_authenticated(player) {
            player.send_client_message(
                Colour::from_rgba(0xFF000000),
                "You are kicked from server (Reason: Not loggedin) !!",
            );
            self.delayed_kick(player);
        }
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

    let timer = register!(Timer::new().unwrap());
    let auth_module = register!(Auth::new(
        pool.clone(),
        connection.clone(),
        on_player_register,
        on_player_login,
    )
    .unwrap());
    register!(SanAndreasUnbound::new(timer, auth_module));

    log!("San Andreas Unbound v{VERSION} loaded");
}
