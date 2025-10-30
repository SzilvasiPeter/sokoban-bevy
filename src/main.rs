use bevy::prelude::*;
use std::collections::HashSet;
use std::ops::Add;

const TILE: i32 = 20;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (player_input, box_movement, collision, apply_direction).chain(),
        )
        .run();
}

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
struct Wall;

#[derive(Component, Default, Clone, Copy)]
struct Direction(Vec3);

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    let maze = [
        "**********",
        "*        *",
        "*  **    *",
        "*  *     *",
        "*        *",
        "**********",
    ];

    for (y, row) in maze.iter().enumerate() {
        for (x, _) in row.chars().enumerate().filter(|(_, c)| *c == '*') {
            let position = Grid(x as i32 * TILE, y as i32 * TILE);
            commands.spawn((
                Wall,
                position,
                Text2d::new("*"),
                Transform::from_translation(position.into()),
            ));
        }
    }

    let position = Grid(100, 40);
    commands.spawn((
        Player,
        position,
        Direction::default(),
        Text2d::new("X"),
        Transform::from_translation(position.into()),
    ));

    for position in [Grid(100, 60), Grid(120, 40)] {
        commands.spawn((
            Box,
            position,
            Direction::default(),
            Text2d::new("O"),
            Transform::from_translation(position.into()),
        ));
    }
}

fn player_input(keys: Res<ButtonInput<KeyCode>>, mut query: Query<&mut Direction, With<Player>>) {
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

fn apply_direction(mut query: Query<(&mut Transform, &mut Direction, &mut Grid)>) {
    for (mut transform, mut direction, mut grid) in &mut query {
        transform.translation += direction.0;
        *grid = *grid + Grid::from(direction.0);
        direction.0 = Vec3::ZERO;
    }
}
