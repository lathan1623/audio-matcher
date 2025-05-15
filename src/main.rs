use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rustfft::num_complex::Complex;

fn main() -> Result<(), anyhow::Error> {
    let audio_data = Arc::new(Mutex::new(Vec::new()));
    let audio_data_clone = Arc::clone(&audio_data);

    let host = cpal::default_host();

    // Set up the input device and stream with the default input config.
    let device = host.default_input_device()
        .expect("failed to find input device");
    println!("Input device: {}", device.name()?);

    let config = device.default_input_config()
        .expect("No supported I8 output format");

    println!("Begin recording...");

    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {}", err);
    };

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
    visualize_spectogram(&spectogram);

    Ok(())
}

fn visualize_spectogram(data: &Vec<[Complex<f32>; 128]>) {
    for time_slice in data {
        println!("$$$$$$$$$$$$$$$$$$$$$$$$$$$$$");
        for point in time_slice {
            println!("{}", point);
        }
        println!("$$$$$$$$$$$$$$$$$$$$$$$$$$$$$");
    }
}

fn build_spectogram(data: Arc<Mutex<Vec<f32>>>) -> Vec<[Complex<f32>; 128]>  {
    //TODO might want to deal with the remainder
    let mut time_sliced_data: Vec<[Complex<f32>; 128]> = data
        .lock()
        .unwrap()
        .chunks_exact(128)
        .map(|chunk| {
            let mut buf = [Complex::new(0.0, 0.0); 128];
            for (i, &real) in chunk.iter().enumerate() {
                buf[i].re = real;
            }
            buf
        })
        .collect();

    let mut planner = rustfft::FftPlanner::new();
    let fft = planner.plan_fft_forward(128);

    for time_slice in &mut time_sliced_data {
        fft.process(time_slice);
    } 

    time_sliced_data
}
