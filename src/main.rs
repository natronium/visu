use bevy::prelude::*;
use cpal::{
    SampleFormat, StreamConfig,
    traits::{DeviceTrait as _, HostTrait, StreamTrait as _},
};
use crossbeam_channel::{bounded, unbounded};
use core::time;
use std::{
    collections::VecDeque,
    io::{Write, stdout},
    thread,
};


const SAMPLE_RATE: u32 = 48000;
pub const BUFFER_CAPACITY: usize = SAMPLE_RATE as usize / 10; // hold on to 0.1s of audio

fn main() -> Result<(), Box<dyn std::error::Error>> {

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

    let (tx, rx) = bounded::<f32>(SAMPLE_RATE as usize);
    let stream = device.build_input_stream(
        &config,
        move |data: &[f32], _info| {
            for sample in data {
                tx.send(*sample).ok();
            }
        },
        |_e| {},
        None,
    )?;

    stream.play()?;

    let mut buf = VecDeque::<f32>::with_capacity(BUFFER_CAPACITY * 2);
    
    loop {
        buf.extend(rx.try_iter());
        if buf.len() > BUFFER_CAPACITY {
            let excess = buf.len() - BUFFER_CAPACITY;
            buf.drain(0..excess);
        }
        audiobar(&buf);
    }



    Ok(())
}

fn audiobar(buf: &VecDeque<f32>) {
    let rms: f32 = buf.iter().map(|s| s * s).sum::<f32>().sqrt();
    let bar = "#".repeat((rms * 10.0) as usize);
    let clear = " ".repeat(140);
    print!("\r{clear}");
    print!("\r[{rms:06.3}]{bar}");
    stdout().flush().unwrap();
}
