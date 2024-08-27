mod events;

use mysql::{prelude::Queryable, Pool};
use omp::{players::Player, types::colour::Colour};
use std::{
    collections::HashSet,
    error::Error,
    sync::mpsc::{channel, Receiver, Sender},
};
use threadpool::ThreadPool;

pub struct Auth {
    pool: ThreadPool,
    connection: Pool,
    register_checker_sender: Sender<(i32, Vec<String>)>,
    register_checker_receiver: Receiver<(i32, Vec<String>)>,
    login_sender: Sender<(i32, u64, bool)>,
    login_receiver: Receiver<(i32, u64, bool)>,
    register_sender: Sender<(u64, i32)>,
    register_receiver: Receiver<(u64, i32)>,
    reg_requestee: HashSet<i32>,
    login_requestee: HashSet<i32>,
    bcrypt_cost: u32,
    auth_event: Box<dyn AuthEvents>,
}

impl Auth {
    pub fn new(
        pool: ThreadPool,
        connection: Pool,
        auth_event: Box<dyn AuthEvents>,
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

        let (register_checker_sender, register_checker_receiver) = channel();
        let (login_sender, login_receiver) = channel();
        let (register_sender, register_receiver) = channel();

        Ok(Auth {
            pool,
            connection,
            register_checker_sender,
            register_checker_receiver,
            login_sender,
            login_receiver,
            register_sender,
            register_receiver,
            reg_requestee: HashSet::new(),
            login_requestee: HashSet::new(),
            auth_event,
            bcrypt_cost: 12,
        })
    }

    pub fn set_bcrypt_cost(&mut self, cost: u32) {
        self.bcrypt_cost = cost;
    }
}

#[allow(unused_variables)]
pub trait AuthEvents {
    fn on_player_login(&mut self, player: Player, accountid: u64) {
        player.send_client_message(Colour::from_rgba(0x00FF0000), "Logged in successfully!");
        player.spawn();
    }

    fn on_player_register(&mut self, player: Player, accountid: u64) {
        player.send_client_message(Colour::from_rgba(0x00FF0000), "Sucessfully registered.");
        player.spawn();
    }

    fn on_login_attempt_failed(&mut self, player: Player) {}

    fn on_authorization_cancelled(&mut self, player: Player) {}
}
