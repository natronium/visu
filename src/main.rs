use bevy::prelude::*;
use cpal::{
    SampleFormat, StreamConfig,
    traits::{DeviceTrait as _, HostTrait, StreamTrait as _},
};
use crossbeam_channel::unbounded;
use std::{
    collections::VecDeque,
    io::{Write, stdout},
    thread,
};

use crate::audio_input::{AudioBuffer, AudioInputPlugin};

mod audio_input;



fn main() -> Result<(), Box<dyn std::error::Error>> {
   

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AudioInputPlugin)
        .run();


        
    // loop {
    //     if let Ok(data) = rx.try_recv() {
    //         let rms: f32 = data.iter().map(|s| s * s).sum::<f32>().sqrt();
    //         let bar = "#".repeat((rms * 10.0) as usize);
    //         print!("\r                                                                                                                                            ");
    //         print!("\r{bar}");
    //         stdout().flush().unwrap();
    //     }
    // }

    Ok(())
}

fn read_audio_input_system(res: Res<AudioBuffer>) {
    let rms: f32 = res.0.iter().map(|s| s * s).sum::<f32>().sqrt();
    let len = res.0.len();
    let bar = "#".repeat((rms * 10.0) as usize);
    print!("\r{bar}");
    stdout().flush().unwrap();
}
