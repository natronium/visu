use bevy::{
    color::palettes::css::{GREEN, GREY},
    prelude::*,
    window::PrimaryWindow,
};

use crate::{
    audio_input_plugin::AudioInputPlugin,
    vcam_plugin::VcamPlugin,
};

mod audio_input_plugin;

mod vcam_plugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AudioInputPlugin)
        .add_plugins(VcamPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, grid)
        .add_systems(Update, wave_gizmo)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn wave_gizmo(
    buf_res: Res<audio_input_plugin::AudioBuffer>,
    mut gizmos: Gizmos,
    window: Single<&mut Window, With<PrimaryWindow>>,
) {
    let buf = &buf_res.0;
    let width = window.width();
    let height = window.height();
    let window_domain = interval(-width / 2.0, width / 2.0).unwrap();

    let curve = SampleAutoCurve::new(window_domain, buf.clone())
        .unwrap()
        .graph()
        .map(|(x, y)| vec2(x, y * height / 2.0));

    let resolution = audio_input_plugin::BUFFER_CAPACITY;
    let times = window_domain.spaced_points(resolution).unwrap();
    gizmos.curve_2d(curve, times, GREEN);
}

fn grid(mut gizmos: Gizmos) {
    gizmos.grid_2d(
        Isometry2d::IDENTITY,
        uvec2(100, 100),
        vec2(100., 100.),
        GREY,
    );
}
