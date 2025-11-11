use bevy::prelude::KeyCode::{ArrowDown, ArrowLeft, ArrowRight, ArrowUp, Enter};
use bevy::prelude::*;
use serde::Deserialize;

use std::collections::HashSet;

const TILE: i32 = 32;

type GameObjectsQuery<'world, 'system> =
    Query<'world, 'system, Entity, Or<(With<Player>, With<Box>, With<Goal>, With<Wall>)>>;

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
enum AppState {
    #[default]
    Menu,
    Game,
}

#[derive(Resource)]
struct Menu {
    items: Vec<&'static str>,
    selected: usize,
}

#[derive(Resource)]
struct MoveCounter(u32);
#[derive(Resource, Debug, Deserialize)]
struct Map {
    levels: Vec<Vec<String>>,
    #[serde(default)]
    current: usize,
}

#[derive(Component)]
struct GameStat;
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
#[derive(Component, Clone, Copy, Eq, PartialEq, Hash)]
struct Grid(i32, i32);

impl From<Grid> for Vec3 {
    fn from(g: Grid) -> Self {
        Vec3::new(g.0 as f32, g.1 as f32, 0.0)
    }
}

impl From<Vec3> for Grid {
    fn from(v: Vec3) -> Self {
        Grid(v.x.round() as i32, v.y.round() as i32)
    }
}

impl std::ops::Add for Grid {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Grid(self.0 + rhs.0, self.1 + rhs.1)
    }
}

fn main() {
    let map: Map =
        serde_json::from_str(&std::fs::read_to_string("levels/microban.json").unwrap()).unwrap();

    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<AppState>()
        .insert_resource(Menu {
            items: vec!["Play", "Exit"],
            selected: 0,
        })
        .insert_resource(map)
        .insert_resource(MoveCounter(0))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (handle_menu, update_menu)
                .chain()
                .run_if(in_state(AppState::Menu).and(menu_input)),
        )
        .add_systems(
            OnEnter(AppState::Game),
            (clear_menu, setup_game, render_map),
        )
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
                .run_if(in_state(AppState::Game).and(player_input)),
        )
        .add_systems(
            Update,
            (next_map, clear_map, render_map, update_ui)
                .chain()
                .run_if(in_state(AppState::Game).and(winning)),
        )
        .add_systems(
            Update,
            (keyboard_nav_system, clear_map, render_map, update_ui)
                .chain()
                .run_if(in_state(AppState::Game).and(shortcut)),
        )
        .run();
}

fn setup(mut commands: Commands, menu: Res<Menu>) {
    commands.spawn(Camera2d);
    commands.spawn(Text::new(menu_text(&menu)));
}

fn menu_input(keys: Res<ButtonInput<KeyCode>>) -> bool {
    keys.any_just_pressed([ArrowUp, ArrowDown, Enter])
}

fn handle_menu(
    keys: Res<ButtonInput<KeyCode>>,
    mut menu: ResMut<Menu>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(ArrowUp) {
        menu.selected = (menu.selected + menu.items.len() - 1) % menu.items.len();
    }
    if keys.just_pressed(ArrowDown) {
        menu.selected = (menu.selected + 1) % menu.items.len();
    }
    if keys.just_pressed(Enter) {
        match menu.items[menu.selected] {
            "Play" => next_state.set(AppState::Game),
            "Exit" => println!("TODO: Exit from the app"),
            _ => {}
        }
    }
}

fn update_menu(menu: Res<Menu>, mut query: Query<&mut Text>) {
    if menu.is_changed() {
        for mut text in &mut query {
            text.0 = menu_text(&menu);
        }
    }
}

fn clear_menu(mut commands: Commands, query: Query<Entity, With<Text>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn menu_text(menu: &Menu) -> String {
    menu.items
        .iter()
        .enumerate()
        .map(|(i, &item)| {
            if i == menu.selected {
                format!("> {}\n", item)
            } else {
                format!("  {}\n", item)
            }
        })
        .collect()
}

fn update_ui(
    counter: Res<MoveCounter>,
    map: Res<Map>,
    mut query: Query<&mut Text, With<GameStat>>,
) {
    if counter.is_changed() {
        if let Ok(mut text) = query.single_mut() {
            text.0 = format!("Moves: {} Levels: {}", counter.0, map.current + 1);
        }
    }
}

fn player_input(keys: Res<ButtonInput<KeyCode>>) -> bool {
    keys.any_just_pressed([ArrowUp, ArrowDown, ArrowLeft, ArrowRight])
}

fn move_player(keys: Res<ButtonInput<KeyCode>>, mut query: Query<&mut Direction, With<Player>>) {
    let mut dir = query.single_mut().unwrap();
    dir.0 = match () {
        _ if keys.just_pressed(ArrowUp) => Vec3::Y,
        _ if keys.just_pressed(ArrowDown) => Vec3::NEG_Y,
        _ if keys.just_pressed(ArrowRight) => Vec3::X,
        _ if keys.just_pressed(ArrowLeft) => Vec3::NEG_X,
        _ => Vec3::ZERO,
    } * TILE as f32;
}

fn box_movement(
    player: Query<(&Direction, &Transform), With<Player>>,
    mut boxes: Query<(&mut Direction, &Transform), Without<Player>>,
) {
    let (player_dir, player_pos) = player.single().unwrap();
    let target = player_pos.translation + player_dir.0;
    for (mut b_dir, b_pos) in &mut boxes {
        if b_pos.translation == target {
            b_dir.0 = player_dir.0;
        }
    }
}

fn collision(
    mut player: Query<(&mut Direction, &Grid), With<Player>>,
    mut boxes: Query<(&mut Direction, &Grid), Without<Player>>,
    walls: Query<&Grid, With<Wall>>,
) {
    let (mut player_dir, player_grid) = player.single_mut().unwrap();
    if player_dir.0 == Vec3::ZERO {
        return;
    }

    let wall_set: HashSet<_> = walls.iter().copied().collect();
    let box_set: HashSet<_> = boxes.iter().map(|(_, g)| *g).collect();
    let target = *player_grid + Grid::from(player_dir.0);

    if wall_set.contains(&target) {
        player_dir.0 = Vec3::ZERO;
        return;
    }

    if let Some((mut b_dir, b_grid)) = boxes.iter_mut().find(|(_, g)| **g == target) {
        let box_target = *b_grid + Grid::from(b_dir.0);
        if wall_set.contains(&box_target) || box_set.contains(&box_target) {
            player_dir.0 = Vec3::ZERO;
            b_dir.0 = Vec3::ZERO;
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

fn winning(boxes: Query<&Grid, With<Box>>, goals: Query<&Grid, With<Goal>>) -> bool {
    let goal_set: HashSet<Grid> = goals.iter().copied().collect();
    let box_set: HashSet<Grid> = boxes.iter().copied().collect();

    goal_set == box_set
}

fn clear_map(mut counter: ResMut<MoveCounter>, mut commands: Commands, entities: GameObjectsQuery) {
    counter.0 = 0;
    entities.iter().for_each(|e| commands.entity(e).despawn());
}

fn next_map(mut map: ResMut<Map>) {
    map.current = (map.current + 1).min(map.levels.len() - 1);
}

fn shortcut(keys: Res<ButtonInput<KeyCode>>) -> bool {
    keys.any_just_pressed([KeyCode::KeyR, KeyCode::KeyN, KeyCode::KeyB])
}

fn keyboard_nav_system(keyboard: Res<ButtonInput<KeyCode>>, mut map: ResMut<Map>) {
    if keyboard.just_pressed(KeyCode::KeyN) {
        map.current = (map.current + 1).min(map.levels.len().saturating_sub(1));
    }
    if keyboard.just_pressed(KeyCode::KeyB) {
        map.current = map.current.saturating_sub(1);
    }
}

fn setup_game(mut commands: Commands) {
    commands.spawn((Text::new("Moves: 0 Levels: 1"), GameStat));
}

fn render_map(mut cmds: Commands, map: Res<Map>, assets: Res<AssetServer>, win: Query<&Window>) {
    let Ok(window) = win.single() else {
        return;
    };
    let start_x = -window.width() as i32 / 2;
    let start_y = (window.height() as i32 / 2) - TILE;

    let goal_texture = assets.load("chess/goal.png");
    let box_texture = assets.load("chess/box.png");
    let player_texture = assets.load("chess/player.png");
    let wall_texture = assets.load("chess/wall.png");

    if let Some(level) = map.levels.get(map.current) {
        for (y, line) in level.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                let pos = Grid(
                    start_x + x as i32 * TILE + TILE / 2,
                    start_y - y as i32 * TILE - TILE / 2,
                );
                let pos_vec = Vec3::from(pos);

                if matches!(ch, '.' | '*' | '+') {
                    cmds.spawn((
                        Goal,
                        pos,
                        Sprite::from_image(goal_texture.clone()),
                        Transform::from_translation(pos_vec - Vec3::Z),
                    ));
                }
                if matches!(ch, '$' | '*') {
                    cmds.spawn((
                        Box,
                        pos,
                        Direction::default(),
                        Sprite::from_image(box_texture.clone()),
                        Transform::from_translation(pos_vec),
                    ));
                }
                if matches!(ch, '@' | '+') {
                    cmds.spawn((
                        Player,
                        pos,
                        Direction::default(),
                        Sprite::from_image(player_texture.clone()),
                        Transform::from_translation(pos_vec),
                    ));
                }
                if ch == '#' {
                    cmds.spawn((
                        Wall,
                        pos,
                        Sprite::from_image(wall_texture.clone()),
                        Transform::from_translation(pos_vec),
                    ));
                }
            }
        }
    }
}
