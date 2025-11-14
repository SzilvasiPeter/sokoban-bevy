use crate::{AppState, Maps};
use bevy::prelude::*;

#[derive(Debug)]
enum MenuItem {
    Random,
    Rush,
    Research,
    Settings,
    Exit,
}

#[derive(Resource)]
struct Menu {
    items: Vec<MenuItem>,
    selected: usize,
}

#[derive(Resource, Default)]
struct MapSelection {
    index: usize,
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
        .init_resource::<MapSelection>()
        .add_systems(Startup, setup_menu)
        .add_systems(OnEnter(AppState::Menu), setup_menu)
        .add_systems(OnExit(AppState::Menu), clear_menu)
        .add_systems(
            OnEnter(AppState::ResearchMenu),
            (clear_menu, setup_research).chain(),
        )
        .add_systems(OnExit(AppState::ResearchMenu), clear_menu)
        .add_systems(
            Update,
            (handle_menu, update_menu)
                .chain()
                .run_if(in_state(AppState::Menu).and(menu_input)),
        )
        .add_systems(
            Update,
            (handle_research, update_research)
                .chain()
                .run_if(in_state(AppState::ResearchMenu).and(menu_input)),
        );
    }
}

fn setup_menu(mut commands: Commands, menu: Res<Menu>) {
    commands.spawn(Camera2d);
    commands.spawn(Text::new(menu_text(&menu)));
}

fn menu_input(keys: Res<ButtonInput<KeyCode>>) -> bool {
    keys.any_just_pressed([
        KeyCode::ArrowUp,
        KeyCode::ArrowDown,
        KeyCode::Enter,
        KeyCode::Backspace,
    ])
}

fn handle_menu(
    keys: Res<ButtonInput<KeyCode>>,
    mut menu: ResMut<Menu>,
    maps: Res<Maps>,
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
                maps.0[0].levels.iter().find(|level| !level.solved).unwrap()
            ),
            MenuItem::Rush => println!(
                "TODO: limited time, random map from easy to medium to hard. Use the map.difficulty: {}",
                maps.0[0].difficulty
            ),
            MenuItem::Research => {
                println!("TODO: difficulty (easy, medium, hard) option.");
                println!(
                    "TODO: display the map name {} number of levels {}.",
                    maps.0[0].name, maps.0[0].num_levels
                );
                next_state.set(AppState::ResearchMenu);
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

fn setup_research(mut cmds: Commands, maps: Res<Maps>, mut selection: ResMut<MapSelection>) {
    selection.index = 0;
    cmds.spawn(Text::new(research_text(&maps, selection.index)));
}

fn handle_research(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut selection: ResMut<MapSelection>,
    maps: Res<Maps>,
    mut commands: Commands,
) {
    let total = maps.0.len();

    if keys.just_pressed(KeyCode::ArrowUp) {
        selection.index = (selection.index + total - 1) % total;
    }
    if keys.just_pressed(KeyCode::ArrowDown) {
        selection.index = (selection.index + 1) % total;
    }
    if keys.just_pressed(KeyCode::Backspace) {
        next_state.set(AppState::Menu);
    }
    if keys.just_pressed(KeyCode::Enter) {
        let chosen = maps.0[selection.index].clone();
        commands.insert_resource(chosen);
        next_state.set(AppState::Game);
    }
}

fn update_research(maps: Res<Maps>, selection: Res<MapSelection>, mut query: Query<&mut Text>) {
    if !selection.is_changed() {
        return;
    }

    for mut text in &mut query {
        text.0 = research_text(&maps, selection.index);
    }
}

fn research_text(maps: &Maps, selected: usize) -> String {
    maps.0
        .iter()
        .enumerate()
        .map(|(i, map)| {
            if i == selected {
                format!("> {}\n", map.name)
            } else {
                format!("  {}\n", map.name)
            }
        })
        .collect()
}
