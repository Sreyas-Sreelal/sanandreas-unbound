#[macro_use]
mod helper;
pub mod auth;
pub mod timer;
pub mod user;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use auth::{Auth, AuthEvents};
use mysql::Pool;
use omp::{
    core::SetGameModeText,
    events::Events,
    main,
    players::Player,
    register,
    types::{colour::Colour, vector::Vector3},
};
use threadpool::ThreadPool;
use timer::Timer;
use user::{PlayerInfo, UserInfo};

const VERSION: &str = env!("CARGO_PKG_VERSION");
struct SanAndreasUnbound {
    timer: Rc<RefCell<Timer>>,
    authenticated_players: HashMap<i32, PlayerInfo>,
    userinfo: UserInfo,
}

impl SanAndreasUnbound {
    pub fn new(timer: Rc<RefCell<Timer>>, userinfo: UserInfo) -> Self {
        SanAndreasUnbound {
            timer,
            authenticated_players: HashMap::new(),
            userinfo,
        }
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

impl AuthEvents for Rc<RefCell<SanAndreasUnbound>> {
    fn on_player_login(&mut self, player: Player, account_id: u64) {
        player.send_client_message(Colour::from_rgba(0x00FF0000), "Logged in successfully!");
        self.borrow_mut()
            .userinfo
            .load_player_info(player, account_id);
    }

    fn on_player_register(&mut self, player: Player, account_id: u64) {
        player.send_client_message(Colour::from_rgba(0x00FF0000), "Sucessfully registered.");
        self.borrow_mut()
            .userinfo
            .load_player_info(player, account_id);
    }
}

impl Events for SanAndreasUnbound {
    fn on_tick(&mut self, _elapsed: i32) {
        for (playerid, player_info) in self.userinfo.receiver.try_iter() {
            if let Some(player) = Player::from_id(playerid) {
                self.authenticated_players
                    .insert(player.get_id(), player_info);
                player.spawn();
            }
        }
    }
    fn on_player_connect(&mut self, player: Player) {
        player.send_client_message(
            Colour::from_rgba(0xFF000000),
            "Hey!! Welcome to San Andreas Unbound!!!",
        );
    }

    fn on_player_spawn(&mut self, player: Player) {
        if !self.authenticated_players.contains_key(&player.get_id()) {
            player.send_client_message(
                Colour::from_rgba(0xFF000000),
                "You are kicked from server (Reason: Not loggedin) !!",
            );
            self.delayed_kick(player);
            return;
        }
        let player_info = self.authenticated_players.get(&player.get_id()).unwrap();
        player.set_skin(player_info.skin);
        player.set_pos(Vector3::new(
            player_info.pos_x,
            player_info.pos_y,
            player_info.pos_z,
        ));
    }
    fn on_player_disconnect(
        &mut self,
        player: Player,
        _reason: omp::types::network::PeerDisconnectReason,
    ) {
        if let Some(player_info) = self.authenticated_players.remove(&player.get_id()) {
            self.userinfo.save_player_info(player, player_info);
        }
    }
}

#[main]
fn entry() {
    SetGameModeText("Freeroam/DM/Gangwar");
    let connection = Pool::new(include_str!("../mysql.config")).unwrap();
    let pool = ThreadPool::new(2);
    let timer = register!(Timer::new());
    let userinfo = UserInfo::new(pool.clone(), connection.clone()).unwrap();

    let sau = register!(SanAndreasUnbound::new(timer, userinfo));

    register!(Auth::new(pool.clone(), connection.clone(), Box::new(sau)).unwrap());

    log!("San Andreas Unbound v{VERSION} loaded");
}
