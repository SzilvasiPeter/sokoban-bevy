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
    ResearchMenu,
    Game,
}

#[derive(Resource)]
struct Maps(Vec<Map>);

#[derive(Resource, Deserialize, Clone)]
struct Map {
    name: String,
    difficulty: String,
    num_levels: usize,
    levels: Vec<Level>,
    #[serde(default)]
    current: usize,
}

#[derive(Deserialize, Debug, Clone)]
struct Level {
    height: usize,
    width: usize,
    lines: Vec<String>,
    #[serde(default)]
    solved: bool,
}

fn main() {
    let mut maps = Vec::new();

    for entry in fs::read_dir("levels").unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "json") {
            let content = fs::read_to_string(&path).unwrap();
            let map: Map = serde_json::from_str(&content).unwrap();
            maps.push(map);
        }
    }

    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<AppState>()
        .insert_resource(Maps(maps))
        .add_systems(Startup, setup_camera)
        .add_plugins((MenuPlugin, GamePlugin))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
