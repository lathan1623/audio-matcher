use std::{fs, sync::{Arc, Mutex}};
use rust_audio::{build_spectogram, find_key_points, hash};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut data = Vec::new();

    for song in fs::read_dir("./songs/").expect("Unable to find dir") {
        let path = song?.path();

        if  path.extension().unwrap() != "wav" {
            continue 
        }

        if let Some(file) = path.to_str() {
            let wav = read_wav_file(file);
            for point in wav.unwrap() {
                data.push(point);
            }
        }
    }
    
    let ld = Arc::new(Mutex::new(data));

    let spec = build_spectogram(ld);
    let points = find_key_points(&spec);
    hash(&points);

    Ok(())
}

fn read_wav_file(path: &str) -> anyhow::Result<Vec<f32>> {
    let mut reader = hound::WavReader::open(path)?;
    let spec = reader.spec();

    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Int => {
            let max_amplitude = 2_i32.pow(spec.bits_per_sample as u32 - 1) as f32;
            reader
                .samples::<i32>()
                .map(|s| s.unwrap() as f32 / max_amplitude)
                .collect()
        }
        hound::SampleFormat::Float => {
            reader.samples::<f32>().map(|s| s.unwrap()).collect()
        }
    };

    Ok(samples)
}
