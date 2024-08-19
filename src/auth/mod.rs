mod events;

use mysql::{prelude::Queryable, Pool};
use omp::players::Player;
use std::{
    cell::RefCell,
    collections::HashSet,
    error::Error,
    sync::mpsc::{channel, Receiver, Sender},
};
use threadpool::ThreadPool;

thread_local! {
    static AUTHENTICATED_PLAYERS: RefCell<HashSet<i32>> =  RefCell::new(HashSet::new());
}

pub fn is_player_authenticated(player: Player) -> bool {
    AUTHENTICATED_PLAYERS.with(|x| x.borrow().contains(&player.get_id()))
}

fn set_player_auth(player: Player) {
    AUTHENTICATED_PLAYERS.with(|x| x.borrow_mut().insert(player.get_id()));
}

fn remove_player_auth(player: Player) {
    AUTHENTICATED_PLAYERS.with(|x| x.borrow_mut().remove(&player.get_id()));
}

pub struct Auth {
    pool: ThreadPool,
    connection: Pool,
    register_sender: Sender<(i32, Vec<String>)>,
    register_receiver: Receiver<(i32, Vec<String>)>,
    login_sender: Sender<(i32, bool)>,
    login_receiver: Receiver<(i32, bool)>,
    reg_requestee: HashSet<i32>,
    login_requestee: HashSet<i32>,
    on_player_register: fn(Player),
    on_player_login: fn(Player),
    bcrypt_cost: u32,
}

impl Auth {
    pub fn new(
        pool: ThreadPool,
        connection: Pool,
        on_player_register: fn(Player),
        on_player_login: fn(Player),
    ) -> Result<Self, Box<dyn Error>> {
        let mut conn = connection.get_conn()?;
        conn.query_drop(
            "CREATE TABLE IF NOT EXISTS  User(
                id INTEGER AUTO_INCREMENT PRIMARY KEY,
                username VARCHAR(32),
                password VARCHAR(64)
            )
        ",
        )?;

        let (register_sender, register_receiver) = channel();
        let (login_sender, login_receiver) = channel();

        Ok(Auth {
            pool,
            connection,
            register_sender,
            register_receiver,
            login_sender,
            login_receiver,
            reg_requestee: HashSet::new(),
            login_requestee: HashSet::new(),
            on_player_register,
            on_player_login,
            bcrypt_cost: 12,
        })
    }

    pub fn set_bcrypt_cost(&mut self, cost: u32) {
        self.bcrypt_cost = cost;
    }
}
