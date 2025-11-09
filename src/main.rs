use bevy::prelude::*;
use std::collections::HashSet;
use std::ops::Add;

use crate::loader::load_levels;

mod loader;

const TILE: i32 = 32;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Levels {
            levels: load_levels("levels/microban.slc").ok().unwrap(),
            current: 0,
        })
        .insert_resource(MoveCounter(0))
        .add_systems(Startup, (setup, fill_background, setup_ui))
        .add_systems(
            Update,
            (
                move_player,
                box_movement,
                collision,
                apply_direction,
                update_ui,
            )
                .chain()
                .run_if(player_input),
        )
        .add_systems(
            Update,
            (clear_map, next_map, render_map, update_ui)
                .chain()
                .run_if(winning),
        )
        .add_systems(
            Update,
            (keyboard_nav_system, clear_map, render_map, update_ui)
                .chain()
                .run_if(shortcut),
        )
        .run();
}

#[derive(Resource)]
struct Levels {
    levels: Vec<loader::Level>,
    current: usize,
}

#[derive(Resource)]
struct MoveCounter(u32);

#[derive(Component)]
struct MoveText;

#[derive(Component, Clone, Copy, Eq, PartialEq, Hash)]
struct Grid(i32, i32);

impl From<Grid> for Vec3 {
    fn from(Grid(x, y): Grid) -> Self {
        Vec3::new(x as f32, y as f32, 0.)
    }
}

impl From<Vec3> for Grid {
    fn from(v: Vec3) -> Self {
        Grid(v.x.round() as i32, v.y.round() as i32)
    }
}

impl Add for Grid {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Grid(self.0 + rhs.0, self.1 + rhs.1)
    }
}

#[derive(Component)]
struct Player;
#[derive(Component)]
struct Box;
#[derive(Component)]
struct Goal;
#[derive(Component)]
struct Wall;
#[derive(Component, Default, Clone, Copy)]
struct Direction(Vec3);

fn setup(
    mut commands: Commands,
    levels: Res<Levels>,
    asset_server: Res<AssetServer>,
    windows: Query<&Window>,
) {
    commands.spawn(Camera2d);
    render_map(commands, levels, asset_server, windows);
}

fn setup_ui(mut commands: Commands) {
    commands.spawn((Text::new("Moves: 0"), MoveText));
}

fn update_ui(counter: Res<MoveCounter>, mut query: Query<&mut Text, With<MoveText>>) {
    if counter.is_changed() {
        let mut text = query.single_mut().unwrap();
        text.0 = format!("Moves: {}", counter.0);
    }
}

fn player_input(keys: Res<ButtonInput<KeyCode>>) -> bool {
    keys.any_just_pressed([
        KeyCode::ArrowUp,
        KeyCode::ArrowDown,
        KeyCode::ArrowLeft,
        KeyCode::ArrowRight,
    ])
}

fn move_player(keys: Res<ButtonInput<KeyCode>>, mut query: Query<&mut Direction, With<Player>>) {
    let mut dir = query.single_mut().unwrap();
    let delta = match () {
        _ if keys.just_pressed(KeyCode::ArrowUp) => Vec3::Y,
        _ if keys.just_pressed(KeyCode::ArrowDown) => Vec3::NEG_Y,
        _ if keys.just_pressed(KeyCode::ArrowRight) => Vec3::X,
        _ if keys.just_pressed(KeyCode::ArrowLeft) => Vec3::NEG_X,
        _ => Vec3::ZERO,
    };
    dir.0 = delta * TILE as f32;
}

fn box_movement(
    player: Query<(&Direction, &Transform), With<Player>>,
    mut boxes: Query<(&mut Direction, &Transform), (With<Box>, Without<Player>)>,
) {
    let (player_dir, player_pos) = player.single().unwrap();
    for (mut box_dir, box_pos) in &mut boxes {
        if box_pos.translation == player_pos.translation + player_dir.0 {
            box_dir.0 = player_dir.0;
        }
    }
}

fn collision(
    mut player_q: Query<(&mut Direction, &Grid), With<Player>>,
    mut box_q: Query<(&mut Direction, &Grid), (With<Box>, Without<Player>)>,
    walls: Query<&Grid, With<Wall>>,
) {
    let (mut player_dir, player_grid) = player_q.single_mut().unwrap();
    if player_dir.0 == Vec3::ZERO {
        return;
    }

    let walls: HashSet<Grid> = walls.iter().copied().collect();
    let boxes: HashSet<Grid> = box_q.iter().map(|(_, grid)| *grid).collect();
    let target = *player_grid + Grid::from(player_dir.0);

    if walls.contains(&target) {
        player_dir.0 = Vec3::ZERO;
    } else if let Some((mut box_dir, box_grid)) = box_q.iter_mut().find(|(_, g)| **g == target) {
        let box_target = *box_grid + Grid::from(box_dir.0);
        if walls.contains(&box_target) || boxes.contains(&box_target) {
            player_dir.0 = Vec3::ZERO;
            box_dir.0 = Vec3::ZERO;
        }
    }
}

fn apply_direction(
    mut counter: ResMut<MoveCounter>,
    mut query: Query<(&mut Transform, &mut Direction, &mut Grid, Option<&Player>)>,
) {
    for (mut transform, mut direction, mut grid, player) in &mut query {
        if direction.0 == Vec3::ZERO {
            continue;
        }
        if player.is_some() {
            counter.0 += 1;
        }
        transform.translation += direction.0;
        *grid = *grid + Grid::from(direction.0);
        direction.0 = Vec3::ZERO;
    }
}

fn winning(box_q: Query<&Grid, With<Box>>, goal_q: Query<&Grid, With<Goal>>) -> bool {
    let goals: HashSet<Grid> = goal_q.iter().copied().collect();
    let boxes: HashSet<Grid> = box_q.iter().copied().collect();

    goals == boxes
}

fn clear_map(
    mut counter: ResMut<MoveCounter>,
    mut commands: Commands,
    query: Query<Entity, Or<(With<Player>, With<Box>, With<Goal>, With<Wall>)>>,
) {
    counter.0 = 0;
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn next_map(mut levels: ResMut<Levels>) {
    if levels.current + 1 < levels.levels.len() {
        levels.current += 1;
    }
}

fn shortcut(keys: Res<ButtonInput<KeyCode>>) -> bool {
    keys.any_just_pressed([KeyCode::KeyR, KeyCode::KeyN, KeyCode::KeyB])
}

fn keyboard_nav_system(keyboard: Res<ButtonInput<KeyCode>>, mut levels: ResMut<Levels>) {
    if keyboard.just_pressed(KeyCode::KeyN) && levels.current + 1 < levels.levels.len() {
        levels.current += 1;
    }
    if keyboard.just_pressed(KeyCode::KeyB) && levels.current > 0 {
        levels.current -= 1;
    }
}

fn fill_background(mut commands: Commands, assets: Res<AssetServer>, windows: Query<&Window>) {
    let Ok(window) = windows.single() else {
        return;
    };

    let cols = window.width() as i32 / TILE;
    let rows = window.height() as i32 / TILE;

    let start_x = -window.width() as i32 / 2;
    let start_y = window.height() as i32 / 2;

    for y in 0..rows {
        for x in 0..cols {
            commands.spawn((
                Sprite::from_image(assets.load("ground.png")),
                Transform::from_translation(Vec3::new(
                    (start_x + x * TILE + TILE / 2) as f32,
                    (start_y - y * TILE - TILE / 2) as f32,
                    -2.0,
                )),
            ));
        }
    }
}

fn render_map(
    mut commands: Commands,
    levels: Res<Levels>,
    assets: Res<AssetServer>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let start_x = -window.width() as i32 / 2;
    let start_y = window.height() as i32 / 2;

    if let Some(level) = levels.levels.get(levels.current) {
        for (y, line) in level.lines.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                let pos = Grid(
                    start_x + x as i32 * TILE + TILE / 2,
                    start_y - y as i32 * TILE - TILE / 2,
                );

                if matches!(ch, '.' | '*' | '+') {
                    commands.spawn((
                        Goal,
                        pos,
                        Sprite::from_image(assets.load("goal.png")),
                        Transform::from_translation(Vec3::from(pos) - Vec3::Z),
                    ));
                }
                if matches!(ch, '$' | '*') {
                    commands.spawn((
                        Box,
                        pos,
                        Direction::default(),
                        Sprite::from_image(assets.load("box.png")),
                        Transform::from_translation(Vec3::from(pos)),
                    ));
                }
                if matches!(ch, '@' | '+') {
                    commands.spawn((
                        Player,
                        pos,
                        Direction::default(),
                        Sprite::from_image(assets.load("player.png")),
                        Transform::from_translation(Vec3::from(pos)),
                    ));
                }
                if ch == '#' {
                    commands.spawn((
                        Wall,
                        pos,
                        Sprite::from_image(assets.load("wall.png")),
                        Transform::from_translation(Vec3::from(pos)),
                    ));
                }
            }
        }
    }
}
