use std::borrow::BorrowMut;

use mysql::prelude::Queryable;
use omp::{
    dialogs::{DialogResponse, DialogStyle},
    events::Events,
    players::Player,
    types::colour::Colour,
};

const AUTH_DIALOG: i32 = 32700;

use super::Auth;

impl Events for Auth {
    fn on_tick(&mut self, _elapsed: i32) {
        for (playerid, data) in self.register_checker_receiver.try_iter() {
            if let Some(player) = Player::from_id(playerid) {
                if data.is_empty() {
                    player.send_client_message(
                        Colour::from_rgba(0x77ff0000),
                        "You are not registered!",
                    );
                    self.reg_requestee.insert(playerid);
                    player.show_dialog(
                        AUTH_DIALOG,
                        DialogStyle::Password,
                        "Register your account",
                        "Please enter a password for your account",
                        "Register",
                        "Exit",
                    );
                } else {
                    self.login_requestee.insert(playerid);
                    player.show_dialog(
                        AUTH_DIALOG,
                        DialogStyle::Password,
                        "Login to your account",
                        "This account is Registered. Please enter your password to login.",
                        "Login",
                        "Exit",
                    );
                }
            }
        }

        for (playerid, accountid, success) in self.login_receiver.borrow_mut().try_iter() {
            if let Some(player) = Player::from_id(playerid) {
                if success {
                    self.auth_event.on_player_login(player, accountid);
                } else {
                    self.login_requestee.insert(playerid);
                    self.auth_event.on_login_attempt_failed(player);
                    player.show_dialog(
                        AUTH_DIALOG,
                        DialogStyle::Password,
                        "Login to your account",
                        "Invalid Password. Please enter your password again to login.",
                        "Login",
                        "Exit",
                    );
                }
            }
        }

        for (accountid, playerid) in self.register_receiver.try_iter() {
            if let Some(player) = Player::from_id(playerid) {
                self.auth_event.on_player_register(player, accountid);
            }
        }
    }

    fn on_player_connect(&mut self, player: Player) {
        let player_name = player.get_name();
        let mut conn = self.connection.get_conn().unwrap();
        let sender = self.register_checker_sender.clone();
        let playerid = player.get_id();

        self.pool.execute(move || {
            let data = conn
                .query_map(
                    format!("SELECT password FROM User WHERE username='{player_name}'"),
                    |password| password,
                )
                .unwrap();
            sender.send((playerid, data)).unwrap();
        });
    }

    fn on_dialog_response(
        &mut self,
        player: Player,
        dialog_id: i32,
        response: DialogResponse,
        _list_item: i32,
        input_text: String,
    ) {
        if dialog_id == AUTH_DIALOG {
            let playerid = player.get_id();
            if response == DialogResponse::Right {
                self.reg_requestee.remove(&playerid);
                self.login_requestee.remove(&playerid);
                self.auth_event.on_authorization_cancelled(player);
                return;
            }

            if self.reg_requestee.contains(&playerid) {
                self.reg_requestee.remove(&playerid);
                let mut conn = self.connection.get_conn().unwrap();
                let username = player.get_name();
                let cost = self.bcrypt_cost;
                let register_sender = self.register_sender.clone();

                self.pool.execute(move || {
                    let hashed = bcrypt::hash(input_text, cost).unwrap();
                    conn.query_drop(format!(
                        "INSERT INTO  User(username,password) VALUES(
                            '{username}',
                            '{hashed}'
                        )
                    ",
                    ))
                    .unwrap();
                    register_sender
                        .send((conn.last_insert_id(), playerid))
                        .unwrap();
                });
            } else if self.login_requestee.contains(&playerid) {
                self.login_requestee.remove(&playerid);
                let mut conn = self.connection.get_conn().unwrap();
                let player_name = player.get_name();
                let sender = self.login_sender.clone();
                let playerid = player.get_id();

                self.pool.execute(move || {
                    let data = conn
                        .query_map(
                            format!("SELECT id,password FROM User WHERE username='{player_name}'"),
                            |(id, password): (u64, String)| (id, password),
                        )
                        .unwrap();
                    sender
                        .send((
                            playerid,
                            data[0].0,
                            bcrypt::verify(input_text, &data[0].1).unwrap(),
                        ))
                        .unwrap();
                });
            }
        }
    }
}
