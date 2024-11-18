#[macro_use]
mod helper;
pub mod auth;
pub mod timer;
pub mod user;

use auth::{Auth, AuthEvents};
use mysql::Pool;
use omp::{
    classes::{Class, PlayerClass},
    core::SetGameModeText,
    events::Events,
    main,
    players::{Player, WeaponSlots},
    register,
    types::{colour::Colour, network::PeerDisconnectReason, vector::Vector3},
};
use std::{
    collections::HashMap,
    error::Error,
    sync::{Arc, Mutex},
};
use threadpool::ThreadPool;
use timer::Timer;
use user::{PlayerInfo, UserInfo};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const MAX_LOGIN_ATTEMPTS: usize = 3;

struct SanAndreasUnbound {
    timer: Arc<Mutex<Timer>>,
    authenticated_players: HashMap<i32, PlayerInfo>,
    userinfo: UserInfo,
    login_attempts: HashMap<i32, usize>,
}

impl SanAndreasUnbound {
    pub fn new(timer: Arc<Mutex<Timer>>, userinfo: UserInfo) -> Self {
        SanAndreasUnbound {
            timer,
            authenticated_players: HashMap::new(),
            userinfo,
            login_attempts: HashMap::new(),
        }
    }
    pub fn delayed_kick(&mut self, player: Player, reason: &str) {
        let playerid = player.get_id();
        player.send_client_message(
            Colour::from_rgba(0xFF000000),
            &format!("You are kicked from server (Reason: {reason}) !!"),
        );
        self.timer.lock().unwrap().set_timer(
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

impl AuthEvents for Arc<Mutex<SanAndreasUnbound>> {
    fn on_player_login(&mut self, player: Player, account_id: u64) {
        player.send_client_message(Colour::from_rgba(0x00FF0000), "Logged in successfully!");
        self.lock()
            .unwrap()
            .userinfo
            .load_player_info(player, account_id);
        player.toggle_spectating(false);
        self.lock().unwrap().login_attempts.remove(&player.get_id());
    }

    fn on_player_register(&mut self, player: Player, account_id: u64) {
        player.send_client_message(Colour::from_rgba(0x00FF0000), "Sucessfully registered.");
        self.lock()
            .unwrap()
            .userinfo
            .load_player_info(player, account_id);
        player.toggle_spectating(false);
    }

    fn on_login_attempt_failed(&mut self, player: Player) {
        let mut gm = self.lock().unwrap();
        let attempts = gm.login_attempts.entry(player.get_id()).or_insert(0);

        *attempts += 1;

        if *attempts == MAX_LOGIN_ATTEMPTS {
            gm.delayed_kick(player, "Multiple failed attempts to login");
            return;
        }

        player.send_client_message(
            Colour::from_rgba(0xFFFF0000),
            &format!("WARNING: Invalid Password Entered ({attempts}/{MAX_LOGIN_ATTEMPTS})"),
        );
    }
    fn on_authorization_cancelled(&mut self, player: Player) {
        self.lock()
            .unwrap()
            .delayed_kick(player, "Didn't login to their account");
    }
}

impl Events for SanAndreasUnbound {
    fn on_tick(&mut self, _elapsed: i32) {
        for (playerid, player_info) in self.userinfo.receiver.try_iter() {
            if let Some(player) = Player::from_id(playerid) {
                self.authenticated_players.insert(playerid, player_info);
                player.spawn();
            }
        }
    }
    fn on_player_connect(&mut self, player: Player) {
        self.login_attempts.remove(&player.get_id());
        player.send_client_message(
            Colour::from_rgba(0xFF000000),
            "Hey!! Welcome to San Andreas Unbound!!!",
        );
    }

    fn on_player_spawn(&mut self, player: Player) {
        if let Some(player_info) = self.authenticated_players.get(&player.get_id()) {
            player.set_skin(player_info.skin);
            player.set_pos(player_info.pos);
        } else {
            self.delayed_kick(player, "Not logged in");
        }
    }

    fn on_player_request_class(&mut self, player: Player, _class_id: i32) -> bool {
        player.set_spawn_info(PlayerClass::default());
        player.toggle_spectating(true);
        if self.authenticated_players.contains_key(&player.get_id()) {
            player.toggle_spectating(false);
        }
        true
    }

    fn on_player_disconnect(&mut self, player: Player, _reason: PeerDisconnectReason) {
        if let Some(player_info) = self.authenticated_players.remove(&player.get_id()) {
            self.userinfo.save_player_info(player, player_info);
        }
    }

    fn on_player_death(&mut self, player: Player, _killer: Option<Player>, _reason: i32) {
        if let Some(playerinfo) = self.authenticated_players.get_mut(&player.get_id()) {
            playerinfo.pos = player.get_pos();
        }
    }
}

#[main]
fn entry() -> Result<(), Box<dyn Error>> {
    SetGameModeText("Freeroam/DM/Gangwar");

    let connection = Pool::new(include_str!("../mysql.config"))?;
    let pool = ThreadPool::new(2);
    let timer = register!(Timer::new());

    let userinfo = UserInfo::new(pool.clone(), connection.clone())?;
    let sau = register!(SanAndreasUnbound::new(timer, userinfo));
    register!(Auth::new(pool.clone(), connection.clone(), Box::new(sau))?);

    log!("San Andreas Unbound v{VERSION} loaded");

    Class::add(
        101,
        0,
        Vector3::new(300.0, 1800.0, 18.0),
        0.0,
        WeaponSlots::default(),
    );

    Ok(())
}
