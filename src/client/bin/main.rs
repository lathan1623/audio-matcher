use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use rust_audio::{build_spectogram, find_key_points, find_hash_matches};

fn main() -> Result<(), anyhow::Error> {

    let audio_data = Arc::new(Mutex::new(Vec::new()));

    let host = cpal::default_host();

    // Set up the input device and stream with the default input config.
    let device = host.default_input_device()
        .expect("failed to find input device");
    println!("Input device: {}", device.name()?);

    let config = device.default_input_config()
        .expect("No supported I8 output format");

    println!("Sample rate: {}", config.sample_rate());

    println!("{}", config.sample_format());

    println!("Begin recording...");

    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {}", err);
    };

    let audio_data_clone = Arc::clone(&audio_data);
    let stream = device.build_input_stream(
        &config.into(),
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let mut buffer = audio_data_clone.lock().unwrap();
            buffer.extend_from_slice(data);
        }, 
        err_fn,
        None
    ).unwrap();

    stream.play().unwrap();

    std::thread::sleep(std::time::Duration::from_secs(3));

    drop(stream);

    println!("No longer capturing stream");

    let spectogram = build_spectogram(Arc::clone(&audio_data));
    let freq = find_key_points(&spectogram);

    find_hash_matches(&freq);

    Ok(())
}

