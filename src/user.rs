use std::{
    error::Error,
    sync::mpsc::{channel, Receiver, Sender},
};

use mysql::{prelude::Queryable, Pool};
use omp::{players::Player, types::vector::Vector3};
use threadpool::ThreadPool;

pub struct PlayerInfo {
    pub account_id: u64,
    pub skin: i32,
    pub pos: Vector3,
}

pub struct UserInfo {
    pool: ThreadPool,
    db: Pool,
    sender: Sender<(i32, PlayerInfo)>,
    pub receiver: Receiver<(i32, PlayerInfo)>,
}

impl UserInfo {
    pub fn new(pool: ThreadPool, db: Pool) -> Result<Self, Box<dyn Error>> {
        let mut conn = db.get_conn()?;

        conn.query_drop(
            "
            CREATE TABLE IF NOT EXISTS UserInfo(
                id INTEGER,
                skin INTEGER,
                pos_x FLOAT,
                pos_y FLOAT,
                pos_z FLOAT,
                PRIMARY KEY(id),
                FOREIGN KEY(id) REFERENCES User(id)
            )
        ",
        )?;
        let (sender, receiver) = channel();

        Ok(Self {
            pool,
            db,
            sender,
            receiver,
        })
    }
    pub fn load_player_info(&mut self, player: Player, account_id: u64) {
        let mut conn = self.db.get_conn().unwrap();
        let playerid = player.get_id();
        let sender = self.sender.clone();
        let pos = player.get_pos();
        self.pool.execute(move || {
            let mut data = conn
                .query_map(
                    format!("SELECT * FROM UserInfo WHERE id={account_id}"),
                    |(_id, skin, x, y, z): (i32, i32, f32, f32, f32)| PlayerInfo {
                        account_id,
                        skin,
                        pos: Vector3::new(x, y, z),
                    },
                )
                .unwrap();
            if !data.is_empty() {
                let _ = sender.send((playerid, data.pop().unwrap()));
            } else {
                conn.query_drop(format!(
                    "INSERT INTO UserInfo VALUES(
                        {account_id},
                        230,
                        0.0,
                        0.0,
                        0.0
                    )"
                ))
                .unwrap();
                let _ = sender.send((
                    playerid,
                    PlayerInfo {
                        account_id,
                        skin: 230,
                        pos,
                    },
                ));
            }
        });
    }

    pub fn save_player_info(
        &mut self,
        player: Player,
        PlayerInfo {
            account_id, skin, ..
        }: PlayerInfo,
    ) {
        let pos = player.get_pos();
        let pos_x = pos.x;
        let pos_y = pos.y;
        let pos_z = pos.z;

        let mut conn = self.db.get_conn().unwrap();
        self.pool.execute(move || {
            conn.query_drop(format!(
                "
                UPDATE UserInfo SET 
                skin = {skin},
                pos_x = {pos_x},
                pos_y = {pos_y},
                pos_z = {pos_z}
                WHERE id = {account_id}  
            "
            ))
            .unwrap();
        });
    }
}
