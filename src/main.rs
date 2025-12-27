use bevy::{
    camera::CameraProjection as _,
    color::palettes::css::{GREEN, GREY},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

use crate::{
    audio_input_plugin::AudioInputPlugin,
    vcam_plugin::{CamBuffer, VcamPlugin},
};

mod audio_input_plugin;

mod vcam_plugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(AudioInputPlugin)
        .add_plugins(VcamPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, grid)
        .add_systems(Update, wave_gizmo)
        .run();
}

fn setup(mut commands: Commands, vcam_image: Res<CamBuffer>) {
    commands.spawn((
        Name::new("Window Camera"),
        Transform::from_xyz(0., 0., 1300.),
        Camera3d::default(),
    ));

    commands
        .spawn((
            Name::new("VCam Texture Camera"),
            Transform::from_xyz(0., 0., 500.),
            Camera3d::default(),
            Camera {
                order: -1,
                target: vcam_image.0.clone().into(),
                ..default()
            },
            Projection::Orthographic(OrthographicProjection::default_2d()),
            OrthographicProjection::default_2d()
                .compute_frustum(&GlobalTransform::from(Transform::default())),
        ));

    commands.spawn((
        Name::new("Camera Output Preview"),
        Node {
            position_type: PositionType::Absolute,
            top: px(50),
            left: px(50),
            border: UiRect::all(px(5)),
            ..default()
        },
        BorderColor::all(Color::WHITE),
        ImageNode::new(vcam_image.0.clone()),
    ));
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
