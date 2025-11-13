use bevy::prelude::*;
use serde::Deserialize;
use std::fs;

mod game;
mod menu;

use game::*;
use menu::*;

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
enum AppState {
    #[default]
    Menu,
    Game,
}

#[derive(Resource, Deserialize)]
struct Map {
    name: String,
    difficulty: String,
    num_levels: usize,
    levels: Vec<Level>,
    #[serde(default)]
    current: usize,
}

#[derive(Deserialize, Debug)]
struct Level {
    height: usize,
    width: usize,
    lines: Vec<String>,
    #[serde(default)]
    solved: bool,
}

fn main() {
    let content = fs::read_to_string("levels/Easy_5_boxes.json").unwrap();
    let map: Map = serde_json::from_str(&content).unwrap();

    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<AppState>()
        .insert_resource(map)
        .add_plugins((MenuPlugin, GamePlugin))
        .run();
}
