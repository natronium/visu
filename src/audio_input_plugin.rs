use bevy::prelude::*;
use cpal::traits::{DeviceTrait as _, HostTrait as _, StreamTrait as _};
use cpal::{SampleFormat, Stream, StreamConfig};
use crossbeam_channel::{Receiver, bounded};
use std::collections::VecDeque;
use std::iter::repeat_n;

const SAMPLE_RATE: u32 = 48000;
pub const BUFFER_CAPACITY: usize = SAMPLE_RATE as usize / 10; // hold on to 0.1s of audio
pub(crate) struct AudioInputPlugin;

#[derive(Resource)]
#[expect(unused)] // Need to hold on to the Stream so that it keeps recording audio
pub(crate) struct AudioBuffer(pub(crate) VecDeque<f32>, Stream);

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

pub(crate) fn create_audioinput_stream() -> (Receiver<f32>, Stream) {
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

pub(crate) fn refill_buffer_from_stream(rx: &Receiver<f32>, buf: &mut VecDeque<f32>) {
    buf.extend(rx.try_iter());
    if buf.len() > BUFFER_CAPACITY {
        let excess = buf.len() - BUFFER_CAPACITY;
        buf.drain(0..excess);
    }
}
