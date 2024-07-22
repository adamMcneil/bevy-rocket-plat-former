#![feature(proc_macro_hygiene, decl_macro)]
extern crate rocket;
use rocket::http::Method;
use rocket::{
    get, post, routes,
};
use rocket::{Config, State};
use rocket_contrib::json::Json;
use rocket_cors::{AllowedOrigins, CorsOptions};

use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use serde::{Deserialize, Serialize};

use bevy::prelude::*;

#[derive(Resource)]
struct BevyReceiver(Arc<Mutex<Receiver<RocketMessage>>>);

#[derive(Serialize, Deserialize, Debug)]
enum Movement {
    Right,
    EndRight,
    Left,
    EndLeft,
    Jump,
    Dive,
    Join,
    Leave,
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

#[post("/control", data = "<rocket_message>")]
fn control_player(rocket_message: Json<RocketMessage>, transmitter: State<Sender<RocketMessage>>) {
    // println!("{:?}", rocket_message);
    let _ = transmitter.send(rocket_message.into_inner());
}

fn receive_message(receiver: Res<BevyReceiver>) {
    match receiver.0.lock() {
        Ok(receiver) => {
            if let Ok(message) = receiver.try_recv() {
                // println!("here");
                println!("{}", serde_json::to_string(&message).unwrap());
            }
        }
        Err(_) => (),
    }
}

fn main() {
    let rocket_message = RocketMessage {
        player: "Adam".to_string(),
        movement: Movement::Right,
    };
    println!("{}", serde_json::to_string(&rocket_message).unwrap());
    let (transmitter, receiver): (Sender<RocketMessage>, Receiver<RocketMessage>) = mpsc::channel();

    let rocket_thread = thread::spawn(move || {
        let cors = CorsOptions::default()
            .allowed_origins(AllowedOrigins::all())
            .allowed_methods(
                vec![Method::Get, Method::Post]
                    .into_iter()
                    .map(From::from)
                    .collect(),
            )
            .allow_credentials(true)
            .to_cors();

        rocket::custom(
            Config::build(rocket::config::Environment::Staging)
                .address("0.0.0.0")
                .finalize()
                .unwrap(),
        )
        .manage(transmitter)
        .mount("/api/v1", routes![heartbeat, control_player])
        .attach(cors.unwrap())
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
