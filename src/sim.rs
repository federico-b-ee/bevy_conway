use bevy::time::FixedTimestep;
use bevy::{prelude::*};
use rand::Rng;

const GRID_WIDTH: u32 = 25;
const GRID_HEIGHT: u32 = 25;
const SPACE_TOP: u32 = 2;
const LIFEFORM_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);
const EMPTY_COLOR: Color = Color::rgb(1.0, 1.0, 1.0);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum Element {
    Lifeform,
    Empty,
}

const SIZE: f32 = 0.95; // it gives some padding
#[derive(Component)]
pub struct CellGrid {
    element: Element,
    size: f32, // less than 1
}

#[derive(Component)]
struct Position {
    x: u32,
    y: u32,
}

#[derive(Bundle)]
struct CellBundle {
    pos: Position,
    element: CellGrid,
}

#[derive(Resource)]
struct GridMap {
    vec: Vec<Vec<bool>>,
}

fn grid_scale(
    windows: Res<Windows>,
    mut query: Query<(&CellGrid, &mut Transform), With<CellGrid>>,
) {
    let win = windows.get_primary().unwrap();
    for (cell, mut transform) in query.iter_mut() {
        transform.scale = Vec3::new(
            cell.size / GRID_WIDTH as f32 * win.width(),
            cell.size / GRID_HEIGHT as f32 * win.height(),
            0.0,
        );
    }
}

fn pos_translation(
    windows: Res<Windows>,
    mut query: Query<(&Position, &mut Transform), With<CellGrid>>,
) {
    // the bounds are set in order to have a squared window, as well as same GRID_WIDTH / GRID_HEIGHT
    fn coord_transform(pos: f32, win_bounds: f32, map_bounds: f32) -> f32 {
        let cell_size = win_bounds / map_bounds;
        pos / map_bounds * win_bounds - (win_bounds / 2.) + (cell_size / 2.)
    }
    let window = windows.get_primary().unwrap();
    for (pos, mut transform) in query.iter_mut() {
        transform.translation = Vec3::new(
            coord_transform(pos.x as f32, window.width(), GRID_WIDTH as f32),
            coord_transform(pos.y as f32, window.height(), GRID_HEIGHT as f32),
            0.0,
        );
    }
}

pub struct SimPlugin;

impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GridMap {
            vec: vec![vec![false; GRID_HEIGHT as usize]; GRID_WIDTH as usize],
        })
        .insert_resource(State(false))
        .add_startup_system(setup)
        .add_startup_system(set_text)
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(pos_translation)
                .with_system(grid_scale),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(0.025))
                //.with_system(print_vec) debug usage
                .with_system(handle_sim),
        )
        .add_system(handle_keyboard)
        .add_system(handle_click)
        .add_system(text_update_system)
        ;
    }
}

fn setup(mut commands: Commands) {
    for x in 0..GRID_WIDTH {
        for y in 0..(GRID_HEIGHT - SPACE_TOP){
            commands
                .spawn(SpriteBundle {
                    sprite: Sprite {
                        color: EMPTY_COLOR,
                        ..default()
                    },
                    ..default()
                })
                .insert(CellBundle {
                    element: CellGrid {
                        element: Element::Empty,
                        size: SIZE,
                    },
                    pos: Position { x, y },
                });
        }
    }
}

fn handle_click(
    mouse_input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    mut query: Query<(&Position, &mut CellGrid, &mut Sprite), With<CellGrid>>,
    mut map: ResMut<GridMap>,
) {
    fn coord_transform(pos: f32, win_bounds: f32, map_bounds: f32) -> u32 {
        let cell_size = win_bounds / map_bounds;
        (pos / cell_size).floor() as u32
    }

    let win = windows.get_primary().expect("no primary window");
    if mouse_input.just_pressed(MouseButton::Left) {
        if let Some(cursor_pos) = win.cursor_position() {
            let x = coord_transform(cursor_pos.x, win.width(), GRID_WIDTH as f32);
            let y = coord_transform(cursor_pos.y, win.height(), GRID_HEIGHT as f32);

            for (pos, mut cell_grid, mut spr) in query.iter_mut() {
                if (pos.x == x) & (pos.y == y) {
                    if cell_grid.element == Element::Empty {
                        cell_grid.element = Element::Lifeform;
                        spr.color = LIFEFORM_COLOR;
                        map.vec[pos.x as usize][pos.y as usize] = true;
                    } else if cell_grid.element == Element::Lifeform {
                        cell_grid.element = Element::Empty;
                        spr.color = EMPTY_COLOR;
                        map.vec[pos.x as usize][pos.y as usize] = false;
                    }
                }
            }
        }
    }
}

// debug usage
/* fn print_vec(mut map: ResMut<GridMap>) {
    println!("{}", "-".repeat(80));
    println!("{:?}", map.vec);
} */

fn handle_sim(
    mut map: ResMut<GridMap>,
    mut query: Query<(&Position, &mut CellGrid, &mut Sprite), With<CellGrid>>,
    state: ResMut<State>,
) {
    if state.0 {
        let mut cloned_map_vec = map.vec.clone(); // cloned map so it can be used for processing and then modify the actual map
        for (pos, mut cell_grid, mut spr) in query.iter_mut() {
            let mut n = 0; // neighbour counter

            // Conway's Game of Life Main Rules:
            // Any live cell with two or three live neighbours survives.
            // Any dead cell with three live neighbours becomes a live cell.
            // All other live cells die in the next generation. Similarly, all other dead cells stay dead.
            let x = pos.x as i32;
            let y = pos.y as i32;

            for i in (x - 1)..(x + 2) {
                for j in (y - 1)..(y + 2) {
                    if (i != x || j != y)
                        && (i < GRID_WIDTH as i32 && j < GRID_HEIGHT as i32)
                        && (j >= 0)
                        && (i >= 0)
                    {
                        if map.vec[i as usize][j as usize] {
                            n += 1;
                        }
                    }
                }
            }

            if n < 2 || n > 3 {
                if cell_grid.element == Element::Lifeform {
                    cell_grid.element = Element::Empty;
                    spr.color = EMPTY_COLOR;
                    cloned_map_vec[pos.x as usize][pos.y as usize] = false;
                }
            }

            let mut r = rand::thread_rng();
            let mut g = rand::thread_rng();
            let mut b = rand::thread_rng();

            if n == 3 {
                if cell_grid.element == Element::Empty {
                    cell_grid.element = Element::Lifeform;
                    spr.color = Color::rgb(
                        r.gen_range((133.0 / 255.0)..(250.0 / 255.0)),
                        g.gen_range((211.0 / 255.0)..(250.0 / 255.0)),
                        b.gen_range((56.0 / 255.0)..(110.0 / 255.0)),
                    );
                    cloned_map_vec[pos.x as usize][pos.y as usize] = true;
                }
            }
        }

        map.vec = cloned_map_vec;
    }
}

#[derive(Resource)]
struct State(bool);

// play/pause and reset
// SPACE || S -> play/pause
// R          -> clean board
fn handle_keyboard(
    key: Res<Input<KeyCode>>,
    mut state: ResMut<State>,
    commands: Commands,
    mut map: ResMut<GridMap>,
) {
    if key.just_pressed(KeyCode::Space) | key.just_pressed(KeyCode::S) {
        state.0 = !state.0;
    }
    if key.just_pressed(KeyCode::R) {
        setup(commands);
        map.vec = vec![vec![false; GRID_HEIGHT as usize]; GRID_WIDTH as usize];
    }
}

#[derive(Component)]
struct  StateText;

fn set_text(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    let font = asset_server.load("Kid Marker.ttf");
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "State: ",
                TextStyle {
                    font: font.clone(),
                    font_size: 25.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::from_style(TextStyle {
                font: font.clone(),
                font_size: 25.0,
                color: Color::RED,
            }),
        ]),
        StateText,
    ));
}

fn text_update_system(
    state: Res<State>,
    mut query: Query<&mut Text, With<StateText>>
) {
    for mut text in &mut query {
        if state.0 {
            text.sections[1].value = format!("Playing");
        } else {
            text.sections[1].value = format!("Stopped");
        }
    }
}