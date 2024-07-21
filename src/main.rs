#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
use rocket::{Config, State};

use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use bevy::prelude::*;

#[derive(Resource)]
struct BevyReceiver(Arc<Mutex<Receiver<String>>>);

#[get("/")]
fn index(transmitter: State<Sender<String>>) -> () {
    transmitter.send("Hello".to_string()).unwrap();
    // println!("{}", message.0.to_string());
}

#[post("/<player>")]
fn add_player(player: String, transmitter: State<Sender<String>>) -> String {
    transmitter.send(player.clone()).unwrap();
    player
}

fn receive_message(receiver: Res<BevyReceiver>) {
    match receiver.0.lock() {
        Ok(receiver) => {
            if let Ok(message) = receiver.try_recv() {
                println!("{}", message)
            }
        }
        Err(_) => (),
    }
}

fn main() {
    let (transmitter, receiver): (Sender<String>, Receiver<String>) = mpsc::channel();

    let rocket_thread = thread::spawn(move || {
        rocket::custom(
            Config::build(rocket::config::Environment::Staging)
                .address("0.0.0.0")
                .finalize()
                .unwrap(),
        )
        .manage(transmitter)
        .mount("/api/v1", routes![index, add_player])
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
