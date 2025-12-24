use bevy::{
    color::palettes::css::{GREEN, GREY},
    prelude::*,
    window::PrimaryWindow,
};
use cpal::{
    SampleFormat, Stream, StreamConfig,
    traits::{DeviceTrait as _, HostTrait, StreamTrait as _},
};
use crossbeam_channel::{Receiver, bounded};
use std::{collections::VecDeque, iter::repeat_n};

const SAMPLE_RATE: u32 = 48000;
pub const BUFFER_CAPACITY: usize = SAMPLE_RATE as usize / 10; // hold on to 0.1s of audio

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AudioInputPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, grid)
        .add_systems(Update, wave_gizmo)
        .run();
}

struct AudioInputPlugin;

#[derive(Resource)]
#[expect(unused)] // Need to hold on to the Stream so that it keeps recording audio
struct AudioBuffer(VecDeque<f32>, Stream);

impl Plugin for AudioInputPlugin {
    fn build(&self, app: &mut App) {
        let (rx, stream) = create_audioinput_stream();

        let update_buffer = move |mut buf_res: ResMut<AudioBuffer>| {
            refill_buffer_from_stream(&rx, &mut buf_res.0);
        };

        let mut buf = VecDeque::with_capacity(BUFFER_CAPACITY * 2);
        buf.extend(repeat_n(0., BUFFER_CAPACITY * 2));

        app.insert_resource(AudioBuffer(buf, stream))
            .add_systems(Update, update_buffer);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn wave_gizmo(
    buf_res: Res<AudioBuffer>,
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

    let resolution = BUFFER_CAPACITY;
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

fn create_audioinput_stream() -> (Receiver<f32>, Stream) {
    let host = cpal::host_from_id(cpal::HostId::Jack).unwrap();
    let device = host.default_input_device().unwrap();
    let config = device
        .supported_input_configs()
        .unwrap()
        .find(|c| c.channels() == 1 && c.sample_format() == SampleFormat::F32)
        .unwrap()
        .try_with_sample_rate(cpal::SampleRate(SAMPLE_RATE))
        .unwrap();
    println!("{config:?}");
    let config: StreamConfig = config.into();
    let (tx, rx) = bounded::<f32>(SAMPLE_RATE as usize);
    let stream = device
        .build_input_stream(
            &config,
            move |data: &[f32], _info| {
                for sample in data {
                    tx.try_send(*sample).ok();
                }
            },
            |_e| {},
            None,
        )
        .unwrap();
    stream.play().unwrap();
    (rx, stream)
}

fn refill_buffer_from_stream(rx: &Receiver<f32>, buf: &mut VecDeque<f32>) {
    buf.extend(rx.try_iter());
    if buf.len() > BUFFER_CAPACITY {
        let excess = buf.len() - BUFFER_CAPACITY;
        buf.drain(0..excess);
    }
}
