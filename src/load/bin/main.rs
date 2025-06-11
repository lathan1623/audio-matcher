use std::{fs, sync::{Arc, Mutex}};
use rust_audio::{hash, build_spectogram, find_key_points};
use std::ffi::OsString;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for dir_entry in fs::read_dir("./songs/").expect("Unable to find dir") {
        let mut data = Vec::new();
        let song = dir_entry?; 
        let path = song.path();
        let name = song.file_name();

        if  path.extension().unwrap() != "wav" {
            continue 
        }

        if let Some(file) = path.to_str() {
            let wav = read_wav_file(file);
            for point in wav.unwrap() {
                data.push(point);
            }
            let ld = Arc::new(Mutex::new(data));
            let spec = build_spectogram(ld);
            let points = find_key_points(&spec);
            load_hash_file(&points, &name);
        }
    }
    Ok(())
}

fn load_hash_file(points: &Vec<[usize; 6]>, song_name: &OsString) {
    let hash_db = sled::open("hash_db").unwrap();
    for points_slice in points {
        let hash = hash(*points_slice);
        let _ = hash_db.insert(hash.to_be_bytes(), song_name.as_encoded_bytes());
    }
    let _ = hash_db.flush();
}

fn read_wav_file(path: &str) -> anyhow::Result<Vec<f32>> {
    let mut reader = hound::WavReader::open(path)?;
    let spec = reader.spec();

    println!("WAV sample rate: {}", spec.sample_rate);

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
