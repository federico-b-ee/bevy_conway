use bevy::{prelude::*, window::PresentMode};

mod sim;

#[cfg(target_arch = "wasm32")]
const HEIGHT: f32 = 500.0;
#[cfg(target_arch = "wasm32")]
const WIDTH: f32 = 500.0;

#[cfg(target_arch = "x86_64")]
const HEIGHT: f32 = 1000.0;
#[cfg(target_arch = "x86_64")]
const WIDTH: f32 = 1000.0;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.05, 0.05, 0.05)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                width: WIDTH,
                height: HEIGHT,
                title: "Bevy_app".to_string(),
                resizable: false,
                present_mode: PresentMode::AutoNoVsync,
                canvas: Some("#bevy".to_owned()),
                ..default()
            },
            ..default()
        }))
        .add_startup_system(setup_camera)
        .add_plugin(sim::SimPlugin)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}