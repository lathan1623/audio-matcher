use std::sync::{Arc, Mutex};

use rustfft::num_complex::{Complex, ComplexFloat};

use std::hash::{Hash, Hasher};

#[derive(Hash, PartialEq, Eq)]
pub struct Key([usize; 6]);

pub fn find_hash_matches(points: &Vec<[usize; 6]>) {
    let hash_db = sled::open("hash_db").unwrap();
    for point_set in points {
        let hash = hash(*point_set);
        let result = hash_db.get(hash.to_be_bytes()).unwrap();
        if let Some(i) = result {
            let song_name = String::from_utf8(i.to_vec()).unwrap();
            println!("song name found {}", song_name);
        }
    }
}

pub fn hash(points_slice: [usize; 6]) -> u64 {
    let key = Key(points_slice);
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    key.hash(&mut hasher);
    let hash = hasher.finish();
    return hash
}

pub fn find_key_points(data: &Vec<[Complex<f32>; 1024]>) -> Vec<[usize; 6]> {
    let mut freq_data = Vec::new();
    let freq_ranges = [0, 10, 20, 40, 160, 511];
    for time_slice in data {
        let mut cur_freq_index = 0;
        let mut max_points = [f32::NEG_INFINITY; 6];
        let mut max_freqs = [0; 6];
        for freq in 0..511 {
            let mag = 20.0 * time_slice[freq].abs().max(1e-10).log10();
            if freq > freq_ranges[cur_freq_index] {
                cur_freq_index += 1
            }
            if mag > max_points[cur_freq_index] {
                max_points[cur_freq_index] = mag;
                max_freqs[cur_freq_index] = freq
            } 
        }
        freq_data.push(max_freqs);
    }
    freq_data
}

pub fn build_spectogram(data: Arc<Mutex<Vec<f32>>>) -> Vec<[Complex<f32>; 1024]>  {
    //TODO might want to deal with the remainder
    let mut time_sliced_data: Vec<[Complex<f32>; 1024]> = data
        .lock()
        .unwrap()
        .chunks_exact(1024)
        .map(|chunk| {
            let mut buf = [Complex::new(0.0, 0.0); 1024];
            for (i, &real) in chunk.iter().enumerate() {
                buf[i].re = real;
            }
            buf
        })
        .collect();

    let mut planner = rustfft::FftPlanner::new();
    let fft = planner.plan_fft_forward(1024);

    for time_slice in &mut time_sliced_data {
        fft.process(time_slice);
    } 

    time_sliced_data
}
