#![feature(proc_macro_hygiene, decl_macro)]
extern crate rocket;
use rocket::http::Method;
use rocket::{get, post, routes};
use rocket::{Config, State};
use rocket_contrib::json::Json;
use rocket_cors::{AllowedOrigins, CorsOptions};

use std::collections::HashSet;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use serde::{Deserialize, Serialize};

use bevy::prelude::*;

#[derive(Resource)]
struct BevyReceiver(Arc<Mutex<Receiver<RocketMessage>>>);

#[derive(Serialize, Deserialize, Debug, Resource)]
enum Movement {
    Right,
    EndRight,
    Left,
    EndLeft,
    Jump,
    Dive,
    EndDive,
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
fn control_player(
    rocket_message: Json<RocketMessage>,
    players: State<HashSet<String>>,
    transmitter: State<Sender<RocketMessage>>,
) -> Result<(), ()> {
    let rocket_message = rocket_message.into_inner();
    // if players.contains(&rocket_message.player) {
    //     match rocket_message.movement {
    //         Movement::Join => {
    //             Err(())
    //         },
    //         Movement::Leave => {
    //             players.remove(&rocket_message.player);
    //             transmitter.send(rocket_message);
    //             Ok(())
    //         }
    //         _ => {
    //             transmitter.send(rocket_message);
    //             Ok(())
    //         },
    //     }
    // } else {
    //     match rocket_message.movement {
    //         Movement::Join => {
    //             players.insert(rocket_message.player.clone());
    //             transmitter.send(rocket_message);
    //             Ok(())
    //         },
    //         _ => {
    //             Err(())
    //         },
    //     }
    // }
    let _ = transmitter.send(rocket_message);
    Ok(())
}

fn main() {
    let (transmitter, receiver): (Sender<RocketMessage>, Receiver<RocketMessage>) = mpsc::channel();

    let rocket_thread = thread::spawn(move || {
        let players: HashSet<String> = HashSet::new();
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
                // .tls(certs_path, key_path)
                .address("0.0.0.0")
                .finalize()
                .unwrap(),
        )
        .manage(transmitter)
        .manage(players)
        .mount("/api/v1", routes![heartbeat, control_player])
        .attach(cors.unwrap())
        .launch();
    });

    let bevy_receiver = BevyReceiver(Arc::new(Mutex::new(receiver)));
    let current_movement: Movement = Movement::EndLeft;

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(bevy_receiver)
        .insert_resource(current_movement)
        .add_systems(Update, (receive_message, move_sprite))
        .add_systems(Startup, (spawn_cam, spawn_sprite))
        .run();

    rocket_thread.join().expect("Rocket thread panicked");
}

#[derive(Component)]
struct MainCam;

fn spawn_cam(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainCam));
}

fn spawn_sprite(mut commands: Commands) {
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(50.0, 50.0)),
            ..Default::default()
        },
        ..Default::default()
    });
}

fn move_sprite(mut sprite: Query<&mut Transform, With<Sprite>>, current_movement: Res<Movement>) {
    let transform = sprite.get_single_mut();
    if let Ok(mut transform) = transform {
        let current_movement = current_movement.into_inner();
        match current_movement {
            Movement::Right => {
                transform.translation.x += 5.0;
            }
            Movement::Left => {
                transform.translation.x -= 5.0;
            }
            _ => {}
        }
    }
}

fn receive_message(receiver: Res<BevyReceiver>, mut current_movement: ResMut<Movement>) {
    match receiver.0.lock() {
        Ok(receiver) => {
            if let Ok(message) = receiver.try_recv() {
                println!("{}", serde_json::to_string(&message).unwrap());
                *current_movement = message.movement 
            }
        }
        Err(_) => (),
    }
}
