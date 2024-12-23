use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
#[allow(unused_imports)]
use cpal::{SampleRate, Stream};
use hound;
use crossbeam_channel::{Sender, Receiver, bounded};
use dasp_signal::{self as signal, Signal};
use std::sync::{Arc, Mutex};
use std::f32::consts::PI;
use std::time::{Duration, Instant};
use rand::Rng;


// -------------------------------------
// SPECS, PARAMS, VOICES
// -------------------------------------
pub struct Specs {
    pub sample_rate: u32,
    pub channels: u16,
    pub filesize: usize,
}

pub struct GrainParams {
    pub grain_start: usize,
    pub grain_duration: usize,
    pub grain_overlap: f32,
    pub grain_pitch: f32,
    pub specs: Specs,
}

pub struct GrainVoice {
    mystart: f32,
    mypitch: f32,
    mydur: f32,
}


impl GrainVoice {
    pub fn new(mystart: f32, mypitch: f32, mydur: f32) -> Self {
        Self {
            mystart,
            mypitch,
            mydur,
        }
    }
    impl GrainVoice {
    pub fn process_grain(
        &self,
        source_array: &[f32],
        grain_env: &[f32],
        grain_params: &GrainParams,
    ) -> Vec<f32> {
        // If we need to mimic PD’s exact 4-point interpolation 
        // (tabread4~ style) for the source array or the envelope array, 
        // we’d do additional code for interpolation.

        // How many samples in this grain?
        let duration_in_samples =
            (self.mydur * grain_params.grain_duration as f32) as usize;

        let mut output = vec![0.0; duration_in_samples];
        let base_source_start = grain_params.grain_start as f32 + self.mystart;
        let playback_rate = self.mypitch * grain_params.grain_pitch;

        // If your envelope is always the same size as duration_in_samples 
        // (i.e., you generate a new envelope each time to match the grain length exactly), 
        // you might do direct indexing: let env_value = grain_env[i]—but only 
        // if your envelope array is exactly the same length as duration_in_samples.
        for i in 0..duration_in_samples {
            // ----------------------------
            // 1) Envelope ramp
            // ----------------------------
            // env_pos goes from 0..1 across duration_in_samples
            let env_pos = i as f32 / (duration_in_samples as f32);
            // Map env_pos [0..1] -> [0..grain_env.len()-1] 
            let env_index_float = env_pos * (grain_env.len() as f32 - 1.0);
            let env_index = env_index_float.floor() as usize;
            // Later do linear interpolation for a smoother read
            let envelope_value = if env_index < grain_env.len() {
                grain_env[env_index]
            } else {
                0.0
            };

            // ----------------------------
            // 2) Source read ramp
            // ----------------------------
            // Each sample, we move forward by `playback_rate`
            // starting from `base_source_start`.
            let source_index_float = base_source_start + (i as f32 * playback_rate);
            let source_index = source_index_float.floor() as usize;

            let source_value = if source_index < source_array.len() {
                source_array[source_index]
            } else {
                0.0
            };

            output[i] = source_value * envelope_value;
        }

        output
    }

}
// -------------------------------------
// MAIN SYNTH STRUCT
// -------------------------------------
pub struct GranularSynth {
    source_array: Arc<Mutex<Vec<f32>>>,
    grain_env: Arc<Mutex<Vec<f32>>>,
    grain_voices: Vec<GrainVoice>,
    params: Arc<Mutex<GrainParams>>,
    counter: Arc<Mutex<usize>>,
    should_stop: Arc<AtomicBool>,
    grain_sender: Sender<Vec<f32>>,
    grain_receiver: Receiver<Vec<f32>>,
}

impl GranularSynth {
    pub fn new() -> Self {
        let specs = {
            sample_rate: 44100,
            channels: 2,
            filesize: 0,
        };

        let grain_voices = Vec::new();
        grain_voices.push(GrainVoice::new(0.0, 1.0, 1.0));
        grain_voices.push(GrainVoice::new(0.0, 1.0, 1.0));
        grain_voices.push(GrainVoice::new(0.0, 1.0, 1.0));
        grain_voices.push(GrainVoice::new(0.0, 1.0, 1.0));

        Self {
            source_array: Arc::new(Mutex::new(vec![])),
            grain_env: Arc::new(Mutex::new(vec![])),
            grain_voices: grain_voices,
            params: Arc::new(Mutex::new(GrainParams {
                grain_start: 0,
                grain_duration: 4410,
                grain_overlap: 2.0,
                grain_pitch: 1.0,
                specs: specs,
            })),
            counter: Arc::new(Mutex::new(0)),
            should_stop: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn calculate_metro_time_in_ms(&self) -> f32 {
        let params = self.params.lock().unwrap();
        (params.grain_duration as f32 / 2.0) / params.grain_overlap
    }

    pub fn start_scheduler(&self) {
        let synth_clone = self.clone_for_thread(); 
        // see below on how to clone needed fields

        thread::spawn(move || {
            // This is the old schedule_grains loop, 
            // but we exit if should_stop is set to true.
            let metro_time = synth_clone.calculate_metro_time_in_ms();
            let interval = Duration::from_millis(metro_time as u64);
            let mut next_time = Instant::now();

            while !synth_clone.should_stop.load(Ordering::SeqCst) {
                let now = Instant::now();
                if now >= next_time {
                    synth_clone.increment_counter();

                    let output = synth_clone.route_to_grainvoice(
                        &synth_clone.get_source_array(),
                        &synth_clone.get_grain_envelope(),
                    );
                    // Possibly queue this `output` for playback, or 
                    // handle in the audio callback, etc.
                    println!("Generated grain with size: {}", output.len());

                    // Schedule next event
                    next_time = now + interval;
                } else {
                    thread::sleep(Duration::from_millis(1));
                }
            }
        });
    }

    /// Request the scheduler thread to stop.
    /// You could also drop the Arc if you want. 
    pub fn stop_scheduler(&self) {
        self.should_stop.store(true, Ordering::SeqCst);
    }

    // One detail: to move `GranularSynth` into a thread’s closure, 
    // you need to clone the Arc fields. 
    fn clone_for_thread(&self) -> GranularSynth {
        GranularSynth {
            source_array: Arc::clone(&self.source_array),
            grain_env: Arc::clone(&self.grain_env),
            grain_voices: self.grain_voices.clone(), // If you implement Clone for GrainVoice
            params: Arc::clone(&self.params),
            counter: Arc::clone(&self.counter),
            should_stop: Arc::clone(&self.should_stop),
        }
    }

    pub fn increment_counter(&self) {
        let mut counter = self.counter.lock().unwrap();
        *counter = (*counter + 1) % 4;
    }

    pub fn route_to_grainvoice(
        &self,
        source_array: &[f32],
        grain_env: &[f32],
        ) -> Vec<f32> {
        let (r_a, r_b) = Self::generate_random_parameters();
        let mut output = vec![];
        let counter = self.counter.lock().unwrap();

        for voice in &mut self.grain_voices {
            voice.mystart = r_a;
            voice.mypitch = r_b;
            voice.mydur = 1.0;
        }
        // send the random numbers here
        match *counter {
            0 => output = self.grain_voices[0]
                .process_grain(source_array, grain_env, &self.params),
            1 => output = self.grain_voices[1]
                .process_grain(source_array, grain_env, &self.params),
            2 => output = self.grain_voices[2]
                .process_grain(source_array, grain_env, &self.params),
            3 => output = self.grain_voices[3]
                .process_grain(source_array, grain_env, &self.params),
            _ => {}
        }
        output
    }
    pub fn generate_random_parameters() -> (f32, f32) {
        let mut rng = rand::thread_rng();
        let r_a = rng.gen_range(0..10000) as f32;
        let r_b = rng.gen_range(0..200) as f32 / 10000.0 + 1.0;
        (r_a, r_b)
    }
    #[no_mangle]
    pub extern "C" fn load_audio_from_file(
        &self, 
        file_path: *const u8,
        file_path_len: usize
        ) -> i32 {
        // Safety: Convert the raw pointer and length to a Rust string slice
        let file_path_slice = unsafe {
            std::slice::from_raw_parts(file_path, file_path_len)
        };

        match hound::WavReader::open(file_path_str) {
            Ok(mut reader) => {
                let spec = reader.spec();
                println!(
                    "loaded audio: sample rate = {}, channels = {}",
                    spec.sample_rate, spec.channels
                );

                let samples: vec<f32> = reader
                    .samples::<i16>()
                    .map(|s| s.unwrap_or(0) as f32 / 32768.0)
                    .collect();

                let filesize = samples.len();
                let file_path_str = std::str::from_utf8(file_path_slice)
                    .unwrap_or("");
                let mut params = self.params.lock().unwrap();
                params.specs.filesize = filesize;
                params.specs.sample_rate = spec.sample_rate;
                params.specs.channels = spec.channels;

                let mut buffer = self.source_array.lock().unwrap();
                *buffer = samples;
                0 // Success
                  // Return file size and format validity
            }
            Err(_) => -1, // Error
        }
    }
    #[no_mangle]
    pub extern "C" fn generate_grain_envelope(&self, size: usize) {
        let mut env = self.grain_env.lock().unwrap();
        env.clear();
        for i in 0..size {
            let x = (i as f32 / size as f32) * 2.0 - 1.0;
            let value = 0.5 + (0.5 * (x * PI).cos());
            env.push(value);
        }
    }
    pub fn get_source_array(&self) -> Vec<f32> {
        self.source_array.lock().unwrap().clone()
    }
    pub fn get_grain_envelope(&self) -> Vec<f32> {
        self.grain_env.lock().unwrap().clone()
    }
    pub fn set_params(
        &self, 
        start: usize, 
        duration: usize, 
        overlap: usize,
        pitch: f32) {
        let mut params = self.params.lock().unwrap();
        params.grain_start = (start.clamp(0.0, 1.0) * params.specs.filesize as f32) as usize;
        params.grain_duration = duration;
        params.grain_overlap = overlap.clamp(1.0, 2.0);
        params.grain_pitch = pitch.clamp(0.1, 2.0) * params.specs.sample_rate as f32;
    }
    /// A simplistic audio callback. Typically, you'd want a read pointer
    /// that increments for each sample or for each triggered grain.
    #[no_mangle]
    pub extern "C" fn play_audio(&self) -> i32 {
        let source_array = Arc::clone(&self.source_array);
        let params = Arc::clone(&self.params);

        let host = cpal::default_host();
        let output_device = match host.default_output_device() {
            Some(dev) => dev,
            None => {
                eprintln!("No output device found");
                return -1;
            }
        };
        let config = match output_device.default_output_config() {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("Could not get default output config: {}", e);
                return -1;
            }
        };

        let stream = match output_device.build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // This is extremely simplistic. Each sample in `data`
                // is just "used as an index." That’s generally not correct DSP logic.
                // But it shows how you'd lock + read a buffer in real time.
                // In a real system, the read pointer and scheduling would be more advanced.

                let buffer = source_array.lock().unwrap();
                let params = params.lock().unwrap();

                let grain_start = params.grain_start as usize;
                let grain_duration = params.grain_duration as usize;
                // pitch is in samples-per-second scaling
                let grain_pitch = params.grain_pitch;

                for (i, sample) in data.iter_mut().enumerate() {
                    // A naive "index" to read from the buffer
                    // In real usage, you'd track a separate float read pointer,
                    // increment by `grain_pitch`, etc.
                    let index_float =
                        (grain_start as f32 + (i as f32 % grain_duration as f32)) * grain_pitch;
                    let index = index_float as usize;

                    *sample = if let Some(&value) = buffer.get(index) {
                        value
                    } else {
                        0.0
                    };
                }
            },
            |err| eprintln!("Error: {}", err),
            None,
        ) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to build output stream: {}", e);
                return -1;
            }
        };

        if let Err(e) = stream.play() {
            eprintln!("Failed to play stream: {}", e);
            return -1;
        }

        // In real usage, you'd keep the stream alive or store it in self.
        // If this function returns, the stream might go out of scope. 
        // For demonstration, we just sleep for a second:
        thread::sleep(Duration::from_secs(1));

        0
    }
}

// -------------------------------------
// TESTS
// -------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_generation() {
        let synth = GranularSynth::new();
        synth.generate_grain_envelope(1024);
        let env = synth.get_grain_envelope();
        assert_eq!(env.len(), 1024);
        // Check the first and last sample to see if they’re 0.5 + 0.5*cos(...) style
        // They won’t be exactly 1 or 0, but we expect some boundary values
        // Just a sanity check that it’s not empty or out of range
        assert!(env[0] >= 0.0 && env[0] <= 1.0);
        assert!(env[1023] >= 0.0 && env[1023] <= 1.0);
    }

    #[test]
    fn test_load_invalid_file() {
        let synth = GranularSynth::new();

        // Make up an invalid path
        let path_str = "non_existent_file.wav";
        let result = {
            let bytes = path_str.as_bytes();
            synth.load_audio_from_file(bytes.as_ptr(), bytes.len())
        };
        // We expect -1 (an error)
        assert_eq!(result, -1);
    }

    #[test]
    fn test_process_grain_basic() {
        let synth = GranularSynth::new();
        // Put some dummy audio data in source_array
        {
            let mut buf = synth.source_array.lock().unwrap();
            *buf = vec![1.0; 44100]; // 1 second of "1.0" samples at 44.1kHz
        }

        // Generate a short envelope
        synth.generate_grain_envelope(128);

        // Prepare a voice
        let voice = GrainVoice::new(0.0, 1.0, 1.0);

        // Lock params
        let params = synth.params.lock().unwrap();

        // Process one grain
        let source = synth.get_source_array();
        let env = synth.get_grain_envelope();
        let out = voice.process_grain(&source, &env, &params);

        // We expect a certain length: mydur(1.0)*grain_duration(4410)=4410
        assert_eq!(out.len(), 4410);
        // Because we used "1.0" for the source and an envelope in [0..1],
        // The output should be in [0..1] range. Let's do a loose check:
        let max_val = out.iter().cloned().fold(f32::MIN, f32::max);
        assert!(max_val <= 1.0);
        let min_val = out.iter().cloned().fold(f32::MAX, f32::min);
        assert!(min_val >= 0.0);
    }
}
