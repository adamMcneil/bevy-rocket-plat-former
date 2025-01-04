#![feature(proc_macro_hygiene, decl_macro)]

use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use bevy::ecs::query::{QueryData, QueryFilter};
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
    Left,
    None,
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
            "Left" => Ok(Movement::Left),
            "None" => Ok(Movement::None),
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
    //time: u128,
}

#[derive(Resource)]
struct Players {
    players: HashSet<String>,
    players_current_move: HashMap<String, Movement>,
    is_jumping: HashSet<String>,
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
        let server = TcpListener::bind("10.0.0.21:8000").unwrap();
        println!("Server is listing");
        for stream in server.incoming() {
            let connection_transmitter = transmitter.clone();
            spawn(move || {
                let mut websocket = accept(stream.unwrap()).unwrap();
                println!("Connection successful");
                loop {
                    let msg = websocket.read().unwrap();
                    println!("{}", msg);
                    let rocket_message: RocketMessage =
                        serde_json::from_str(&msg.to_string()).unwrap();
                    let _ = connection_transmitter.send(rocket_message);
                }
            });
        }
    });

    let bevy_receiver = BevyReceiver(Arc::new(Mutex::new(receiver)));
    let current_movement: Movement = Movement::None;
    let players: Players = Players {
        players: HashSet::new(),
        players_current_move: HashMap::new(),
        is_jumping: HashSet::new(),
    };

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(bevy_receiver)
        .insert_resource(current_movement)
        .insert_resource(players)
        .add_event::<Join>()
        .add_event::<Jump>()
        .add_systems(Startup, spawn_cam)
        .add_systems(
            Update,
            (
                receive_message,
                move_sprite,
                gravity,
                handle_jump,
                join_game,
            ),
        )
        .run();

    web_socket_thread
        .join()
        .expect("Web socket thread panicked");
}

#[derive(Component)]
struct MainCam;

#[derive(Component)]
struct Player {
    name: String,
    velocity: Vec2,
    on_ground: bool,
    // transform: Transform,
}

#[derive(Component)]
struct Gravity;

#[derive(Component)]
struct Jumping;

#[derive(Event)]
struct Join {
    player: String,
}

#[derive(Event)]
struct Jump {
    player: String,
}

fn spawn_cam(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainCam));
}

fn join_game(mut join_ev: EventReader<Join>, mut commands: Commands, mut players: ResMut<Players>) {
    for event in join_ev.read() {
        spawn_sprite(event.player.clone(), &mut commands);
        players.players.insert(event.player.clone());
    }
}

fn handle_jump(mut players: Query<(&mut Player)>, mut jump_ev: EventReader<Jump>) {
    let jump_strength = 300.0;
    for event in jump_ev.read() {
        for mut p in &mut players {
            if p.name == event.player {
                p.velocity.y = jump_strength;
            }
        }
    }
}

fn spawn_sprite(name: String, commands: &mut Commands) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(50.0, 50.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        Player {
            name: name,
            velocity: Vec2::ZERO,
            on_ground: false,
        },
        Gravity,
        Jumping,
    ));
}

fn move_sprite(mut transforms: Query<(&mut Transform, &mut Player)>, players: Res<Players>) {
    for (mut transform, player) in &mut transforms {
        let player = player.into_inner();
        if let Some(player_movement) = players.players_current_move.get(&player.name) {
            match player_movement {
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
}

fn gravity(time: Res<Time>, mut query: Query<(&mut Player, &mut Transform), With<Gravity>>) {
    let gravity = -600.0; // Gravity strength
    let delta_time = time.delta_seconds();
    let floor = -100.0;

    for (mut player, mut transform) in query.iter_mut() {
        player.velocity.y += gravity * delta_time;
        transform.translation.y += player.velocity.y * delta_time;

        // Simulate ground collision (simple example)
        if transform.translation.y <= floor {
            transform.translation.y = floor;
            player.velocity.y = 0.0;
            player.on_ground = true;
        }
    }
}

fn receive_message(
    receiver: Res<BevyReceiver>,
    mut players: ResMut<Players>,
    mut join_ev: EventWriter<Join>,
    mut jump_ev: EventWriter<Jump>,
) {
    match receiver.0.lock() {
        Ok(receiver) => {
            while let Ok(message) = receiver.try_recv() {
                match message.movement {
                    Movement::Join => {
                        join_ev.send(Join {
                            player: message.player,
                        });
                    }
                    Movement::Jump => {
                        jump_ev.send(Jump {
                            player: message.player,
                        });
                    }
                    _ => {
                        players
                            .players_current_move
                            .insert(message.player, message.movement);
                    }
                }
            }
        }
        Err(_) => (),
    }
}
