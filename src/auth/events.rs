use mysql::prelude::Queryable;
use omp::{
    dialogs::{DialogResponse, DialogStyle},
    events::Events,
    players::Player,
    types::colour::Colour,
};

const AUTH_DIALOG: i32 = 32700;

use super::{remove_player_auth, set_player_auth, Auth};

impl Events for Auth {
    fn on_tick(&mut self, _elapsed: i32) {
        for (playerid, data) in self.register_receiver.try_iter() {
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

        for (playerid, success) in self.login_receiver.try_iter() {
            if let Some(player) = Player::from_id(playerid) {
                if success {
                    set_player_auth(player);
                    (self.on_player_login)(player);
                } else {
                    self.login_requestee.insert(playerid);
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
    }

    fn on_player_connect(&mut self, player: omp::players::Player) {
        remove_player_auth(player);
        let player_name = player.get_name();
        let mut conn = self.connection.get_conn().unwrap();
        let sender = self.register_sender.clone();
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
        response: omp::dialogs::DialogResponse,
        _list_item: i32,
        input_text: String,
    ) {
        if dialog_id == AUTH_DIALOG {
            let playerid = player.get_id();
            if response == DialogResponse::Right {
                self.reg_requestee.remove(&playerid);
                self.login_requestee.remove(&playerid);
                return;
            }

            if self.reg_requestee.contains(&playerid) {
                self.reg_requestee.remove(&playerid);
                let mut conn = self.connection.get_conn().unwrap();
                let username = player.get_name();
                let cost = self.bcrypt_cost;

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
                });

                set_player_auth(player);
                (self.on_player_register)(player);
            } else if self.login_requestee.contains(&playerid) {
                self.login_requestee.remove(&playerid);
                let mut conn = self.connection.get_conn().unwrap();
                let player_name = player.get_name();
                let sender = self.login_sender.clone();
                let playerid = player.get_id();

                self.pool.execute(move || {
                    let data = conn
                        .query_map(
                            format!("SELECT password FROM User WHERE username='{player_name}'"),
                            |password: String| password,
                        )
                        .unwrap();
                    sender
                        .send((playerid, bcrypt::verify(input_text, &data[0]).unwrap()))
                        .unwrap();
                });
            }
        }
    }
}
