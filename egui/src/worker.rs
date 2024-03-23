use rusticnes_ui_common::events;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::collections::VecDeque;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::sync::mpsc::{Receiver};

lazy_static! {
    pub static ref AUDIO_OUTPUT_BUFFER: Mutex<VecDeque<f32>> = Mutex::new(VecDeque::new());
}

pub fn worker_main(runtime_rx: Receiver<events::Event>) {
    // We don't need to DO anything with the stream, but we do need to keep it around
    // or it will stop playing.
    let _audio_stream = setup_audio_stream();

    loop {
        println!("worker thread says hi");
        thread::sleep(Duration::from_millis(1000));
    }
}

pub fn setup_audio_stream() -> Box<dyn StreamTrait> {
    // Setup the audio callback, which will ultimately be in charge of trying to step emulation
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device available");

    // TODO: eventually we want to present the supported configs to the end user, and let
    // them pick
    let default_output_config = device.default_output_config().unwrap();
    println!("default config would be: {:?}", default_output_config);

    let mut stream_config: cpal::StreamConfig = default_output_config.into();
    stream_config.buffer_size = cpal::BufferSize::Fixed(256);
    stream_config.channels = 1;
    println!("stream config will be: {:?}", stream_config);

    let stream = device.build_output_stream(
        &stream_config.into(),
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let mut audio_output_buffer = AUDIO_OUTPUT_BUFFER.lock().expect("wat");
            if audio_output_buffer.len() > data.len() {
                let output_samples = audio_output_buffer.drain(0..data.len()).collect::<VecDeque<f32>>();
                for i in 0 .. data.len() {
                    data[i] = output_samples[i];
                }
            } else {
                for sample in data.iter_mut() {
                    *sample = cpal::Sample::EQUILIBRIUM;
                }
            }
        },
        move |err| {
            println!("Audio error occurred: {}", err)
        },
        None // None=blocking, Some(Duration)=timeout
    ).unwrap();

    stream.play().unwrap();

    return Box::new(stream);
}