use crate::{AppState, Map};
use bevy::prelude::*;

#[derive(Debug)]
pub enum MenuItem {
    Random,
    Rush,
    Research,
    Settings,
    Exit,
}

#[derive(Resource)]
pub struct Menu {
    items: Vec<MenuItem>,
    selected: usize,
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Menu {
            items: vec![
                MenuItem::Random,
                MenuItem::Rush,
                MenuItem::Research,
                MenuItem::Settings,
                MenuItem::Exit,
            ],
            selected: 0,
        })
        .add_systems(Startup, setup_menu)
        .add_systems(
            Update,
            (handle_menu, update_menu)
                .chain()
                .run_if(in_state(AppState::Menu).and(menu_input)),
        )
        .add_systems(OnExit(AppState::Menu), clear_menu);
    }
}

fn setup_menu(mut commands: Commands, menu: Res<Menu>) {
    commands.spawn(Camera2d);
    commands.spawn(Text::new(menu_text(&menu)));
}

fn menu_input(keys: Res<ButtonInput<KeyCode>>) -> bool {
    keys.any_just_pressed([KeyCode::ArrowUp, KeyCode::ArrowDown, KeyCode::Enter])
}

fn handle_menu(
    keys: Res<ButtonInput<KeyCode>>,
    mut menu: ResMut<Menu>,
    map: Res<Map>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::ArrowUp) {
        menu.selected = (menu.selected + menu.items.len() - 1) % menu.items.len();
    }
    if keys.just_pressed(KeyCode::ArrowDown) {
        menu.selected = (menu.selected + 1) % menu.items.len();
    }
    if keys.just_pressed(KeyCode::Enter) {
        match menu.items[menu.selected] {
            MenuItem::Random => println!(
                "TODO: random unsolved level. Use the level.solved: {:?}",
                map.levels.iter().find(|level| !level.solved).unwrap()
            ),
            MenuItem::Rush => println!(
                "TODO: limited time, random map from easy to medium to hard. Use the map.difficulty: {}",
                map.difficulty
            ),
            MenuItem::Research => {
                println!("TODO: difficulty (easy, medium, hard) option.");
                println!(
                    "TODO: display the map name {} number of levels {}.",
                    map.name, map.num_levels
                );
                next_state.set(AppState::Game);
            }
            MenuItem::Settings => println!("TODO: shortcut description, skin selection"),
            MenuItem::Exit => println!("TODO: exit from the app"),
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
        .map(|(i, item)| {
            if i == menu.selected {
                format!("> {:?}\n", item)
            } else {
                format!("  {:?}\n", item)
            }
        })
        .collect()
}
