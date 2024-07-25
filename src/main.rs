#![feature(proc_macro_hygiene, decl_macro)]

use std::str::FromStr;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use serde::{Deserialize, Serialize};

use bevy::prelude::*;

use std::net::TcpListener;
use std::thread::spawn;
use tungstenite::accept;

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

impl FromStr for Movement {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Right" => Ok(Movement::Right),
            "EndRight" => Ok(Movement::EndRight),
            "Left" => Ok(Movement::Left),
            "EndLeft" => Ok(Movement::EndLeft),
            "Jump" => Ok(Movement::Jump),
            "Dive" => Ok(Movement::Dive),
            "EndDive" => Ok(Movement::EndDive),
            "Join" => Ok(Movement::Join),
            "Leave" => Ok(Movement::Leave),
            _ => Err("Unknown movement"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct RocketMessage {
    player: String,
    movement: Movement,
    time: u128,
}

use std::time::{SystemTime, UNIX_EPOCH};

fn millis() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_millis()
}

fn main() {
    let (transmitter, receiver): (Sender<RocketMessage>, Receiver<RocketMessage>) = mpsc::channel();

    let web_socket_thread = thread::spawn(move || {
        let server = TcpListener::bind("10.0.0.21:9001").unwrap();
        println!("Server is listing");
        for stream in server.incoming() {
            let connection_transmitter = transmitter.clone();
            spawn (move || {
                let mut websocket = accept(stream.unwrap()).unwrap();
                println!("Connection successful");
                loop {
                    let msg = websocket.read().unwrap();
                    println!("{}", msg);

                    let _ = connection_transmitter.send(RocketMessage {
                        player: "Adam".to_string(),
                        movement: Movement::from_str(&msg.to_string()).unwrap(),
                        time: 1
                    });

                }
            });
        }
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

    web_socket_thread.join().expect("Web socket thread panicked");
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
            while let Ok(message) = receiver.try_recv() {
                println!("{}", serde_json::to_string(&message).unwrap());
                println!("{}", millis() - message.time);

                *current_movement = message.movement 
            }
        }
        Err(_) => (),
    }
}
