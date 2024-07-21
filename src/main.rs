#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
use rocket::{Config, State};
use rocket_contrib::json::Json;

use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use serde::{Deserialize, Serialize};

use bevy::prelude::*;

#[derive(Resource)]
struct BevyReceiver(Arc<Mutex<Receiver<Movement>>>);

#[derive(Serialize, Deserialize, Debug)]
enum Movement {
    Right,
    EndRight (String),
    Left (String),
    EndLeft (String),
    Jump (String),
    Dive (String),
    Join (String),
    Leave (String),
}

#[derive(Serialize, Deserialize, Debug)]
struct RocketMessage {
    player: String,
    movement: Movement,
}

#[get("/heartbeat")]
fn heartbeat() -> &'static str {
    "heartbeat"
}

// #[get("/")]
// fn index(transmitter: State<Sender<String>>) -> Result<()> {
//     transmitter.send("Hello".to_string()).unwrap();
//     Ok(())
//     // println!("{}", message.0.to_string());
// }

// #[post("/new/<player>")]
// fn add_player(player: String, transmitter: State<Sender<RocketMessage>>) -> String {
//     transmitter.send(geunwrap();
//     player
// }

#[post("/control", data = "<rocket_message>")]
fn control_player(rocket_message: Json<RocketMessage>, transmitter: State<Sender<RocketMessage>>) {
    transmitter.send(rocket_message.into_inner()).unwrap();
}

fn receive_message(receiver: Res<BevyReceiver>) {
    match receiver.0.lock() {
        Ok(receiver) => {
            if let Ok(message) = receiver.try_recv() {
                println!("{}", serde_json::to_string(&message).unwrap())
            }
        }
        Err(_) => (),
    }
}

fn main() {
    // let rocket_message = RocketMessage{player = }
    println!("{}", serde_json::to_string(&Movement::Right).unwrap());
    let (transmitter, receiver): (Sender<Movement>, Receiver<Movement>) = mpsc::channel();

    let rocket_thread = thread::spawn(move || {
        rocket::custom(
            Config::build(rocket::config::Environment::Staging)
                .address("0.0.0.0")
                .finalize()
                .unwrap(),
        )
        .manage(transmitter)
        .mount("/api/v1", routes![heartbeat, control_player])
        .launch();
    });

    let bevy_receiver = BevyReceiver(Arc::new(Mutex::new(receiver)));

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(bevy_receiver)
        .add_systems(Update, receive_message)
        .run();

    rocket_thread.join().expect("Rocket thread panicked");
}
