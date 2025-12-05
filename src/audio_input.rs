use bevy::prelude::*;
use cpal::{
    SampleFormat, StreamConfig,
    traits::{DeviceTrait as _, HostTrait as _, StreamTrait as _},
};
use crossbeam_channel::{Receiver, bounded};
use log::warn;
use std::collections::VecDeque;

pub struct AudioInputPlugin;

const SAMPLE_RATE: u32 = 48000;

#[derive(Resource)]
struct AudioInput(Receiver<Vec<f32>>);

#[derive(Resource)]
pub struct AudioBuffer(pub(crate) VecDeque<f32>);

impl Plugin for AudioInputPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Startup, create_input)
            .insert_resource(AudioBuffer(VecDeque::with_capacity(
                SAMPLE_RATE as usize / 10,
            )))
            .add_systems(Update, update_buffer);
    }
}

fn create_input(mut commands: Commands) {
    let Ok(rx) = setup_audio_stream() else {
        return warn!("Failed to audio!");
    };

    commands.insert_resource(AudioInput(rx));
}

fn update_buffer(input_res: Res<AudioInput>, mut buffer_res: ResMut<AudioBuffer>) {
    let rx = &input_res.0;
    let buf = &mut buffer_res.0;
    while let Ok(samples) = rx.recv() {
        buf.drain(0..samples.len());
        buf.extend(samples);
    }
}

fn setup_audio_stream() -> Result<Receiver<Vec<f32>>, Box<dyn std::error::Error>> {
    let host = cpal::host_from_id(cpal::HostId::Jack)?;
    // let host = cpal::default_host();
    let device = host.default_input_device().unwrap();

    let config = device
        .supported_input_configs()?
        .find(|c| c.channels() == 1 && c.sample_format() == SampleFormat::F32)
        .unwrap()
        .try_with_sample_rate(cpal::SampleRate(SAMPLE_RATE))
        .unwrap();

    println!("{config:?}");

    let config: StreamConfig = config.into();

    let (tx, rx) = bounded::<Vec<f32>>(10);
    let stream = device.build_input_stream(
        &config,
        move |data: &[f32], _info| {
            tx.try_send(data.to_vec()).ok(); // if we're out of space, just don't bother
        },
        |_e| {},
        None,
    )?;

    stream.play()?;

    Ok(rx)
}
