use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
#[allow(unused_imports)]
use cpal::{SampleRate, Stream};
use hound;
use crossbeam_channel::{Sender, Receiver};
#[allow(unused_imports)]
use dasp_signal::{self as signal, Signal};
use std::sync::{Arc, Mutex};
use std::f32::consts::PI;
use std::time::{Duration, Instant};
use rand::Rng;
//use core::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicBool;
use std::thread;
use std::sync::atomic::Ordering;

// -------------------------------------
// SPECS, PARAMS
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

// -------------------------------------
// GRAIN VOICE
// -------------------------------------
#[derive(Clone)]
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

    pub fn process_grain(
        &self,
        source_array: &[f32],
        grain_env: &[f32],
        grain_params: &GrainParams,
    ) -> Vec<f32> {
        // How many samples in this grain?
        let duration_in_samples =
            (self.mydur * grain_params.grain_duration as f32) as usize;

        let mut output = vec![0.0; duration_in_samples];
        let base_source_start = grain_params.grain_start as f32 + self.mystart;
        let playback_rate = self.mypitch * grain_params.grain_pitch;

        for i in 0..duration_in_samples {
            // ----------------------------
            // 1) Envelope ramp
            // ----------------------------
            // env_pos goes from 0..1 across duration_in_samples
            let env_pos = i as f32 / (duration_in_samples as f32);
            // Map env_pos [0..1] -> [0..grain_env.len()-1] 
            let env_index_float = env_pos * (grain_env.len() as f32 - 1.0);
            // let env_index = env_index_float.floor() as usize;
            // Later do linear interpolation for a smoother read
            let envelope_value = 
                four_point_interpolation(grain_env, env_index_float);
            // ----------------------------
            // 2) Source read ramp
            // ----------------------------
            // Each sample, we move forward by `playback_rate` (set by pitch)
            // starting from `base_source_start`.
            let source_index_float = base_source_start + (i as f32 * playback_rate);
            let source_value = 
                four_point_interpolation(source_array, source_index_float);
            
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
    grain_voices: Arc<Mutex<Vec<GrainVoice>>>,
    params: Arc<Mutex<GrainParams>>,
    counter: Arc<Mutex<usize>>,
    should_stop: Arc<AtomicBool>,
    grain_sender: Sender<Vec<f32>>,
    grain_receiver: Receiver<Vec<f32>>,
}
impl GranularSynth {
    pub fn new() -> Self {
        let specs = Specs {
            sample_rate: 44100,
            channels: 2,
            filesize: 0,
        };

        let (s, r) = crossbeam_channel::unbounded();

        let mut grain_voices = Vec::new();
        grain_voices.push(GrainVoice::new(0.0, 1.0, 1.0));
        grain_voices.push(GrainVoice::new(0.0, 1.0, 1.0));
        grain_voices.push(GrainVoice::new(0.0, 1.0, 1.0));
        grain_voices.push(GrainVoice::new(0.0, 1.0, 1.0));

        Self {
            source_array: Arc::new(Mutex::new(vec![])),
            grain_env: Arc::new(Mutex::new(vec![])),
            grain_voices: Arc::new(Mutex::new(grain_voices)),
            params: Arc::new(Mutex::new(GrainParams {
                grain_start: 0,
                grain_duration: 4410,
                grain_overlap: 2.0,
                grain_pitch: 1.0,
                specs: specs,
            })),
            counter: Arc::new(Mutex::new(0)),
            should_stop: Arc::new(AtomicBool::new(false)),
            grain_sender: s,
            grain_receiver: r, 
        }
    }

    pub fn calculate_metro_time_in_ms(&self) -> f32 {
        let params = self.params.lock().unwrap();
        (params.grain_duration as f32 / 2.0) / params.grain_overlap
    }

    pub fn start_scheduler(&self) {
        let synth_clone = self.clone_for_thread(); 
        thread::spawn(move || {
            let metro_time = synth_clone.calculate_metro_time_in_ms();
            let interval = Duration::from_millis(metro_time as u64);
            let mut next_time = Instant::now();

            // mientras sea falso
            while !synth_clone.should_stop.load(Ordering::SeqCst) {
                let now = Instant::now();
                if now >= next_time {
                    synth_clone.route_to_grainvoice(
                        &synth_clone.get_source_array(),
                        &synth_clone.get_grain_envelope(),
                    );
                    synth_clone.increment_counter();
                    // Schedule next event
                    next_time = now + interval;
                } else {
                    thread::sleep(Duration::from_millis(1));
                }
            }
        });
    }

    /// We could also drop the Arc if we wanted.
    pub fn stop_scheduler(&self) {
        self.should_stop.store(true, Ordering::SeqCst);
    }

    // One detail: to move `GranularSynth` into a thread’s closure, 
    // you need to clone the Arc fields. 
    #[allow(dead_code)]
    fn clone_for_thread(&self) -> GranularSynth {
        let (new_sender, new_receiver) = crossbeam_channel::unbounded();

        GranularSynth {
            source_array: Arc::clone(&self.source_array),
            grain_env: Arc::clone(&self.grain_env),
            grain_voices: Arc::clone(&self.grain_voices),
            params: Arc::clone(&self.params),
            counter: Arc::clone(&self.counter),
            should_stop: Arc::clone(&self.should_stop),
            grain_receiver: new_receiver,
            grain_sender: new_sender,
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
        ) {
        let (r_a, r_b) = Self::generate_random_parameters();
        let counter = self.counter.lock().unwrap();

        {
            let mut voices = self.grain_voices.lock().unwrap();
            let voice = &mut voices[*counter];
            voice.mystart = r_a;
            voice.mypitch = r_b;
            voice.mydur = 1.0;
        }

        let voices = self.grain_voices.lock().unwrap();
        let params = self.params.lock().unwrap();
        let grain_data = voices[*counter]
            .process_grain(source_array, grain_env, &params);

        self.grain_sender.send(grain_data).ok();
    }

    pub fn generate_random_parameters() -> (f32, f32) {
        let mut rng = rand::thread_rng();
        let r_a = rng.gen_range(0..10000) as f32;
        let r_b = rng.gen_range(0..200) as f32 / 10000.0 + 1.0;
        (r_a, r_b)
    }
    pub fn load_audio_from_file(
        &self, 
        file_path: *const u8,
        file_path_len: usize
        ) -> i32 {
        // Safety: Convert the raw pointer and length to a Rust string slice
        let file_path_slice = unsafe {
            std::slice::from_raw_parts(file_path, file_path_len)
        };
        let file_path_str = std::str::from_utf8(file_path_slice).unwrap_or("");
        match hound::WavReader::open(file_path_str) {
            Ok(mut reader) => {
                let spec = reader.spec();
                println!(
                    "loaded audio: sample rate = {}, channels = {}",
                    spec.sample_rate, spec.channels
                );

                let samples: Vec<f32> = reader
                    .samples::<i16>()
                    .map(|s| s.unwrap_or(0) as f32 / 32768.0)
                    .collect();

                let filesize = samples.len();
                let mut params = self.params.lock().unwrap();
                params.specs.filesize = filesize;
                params.specs.sample_rate = spec.sample_rate;
                params.specs.channels = spec.channels;

                let mut buffer = self.source_array.lock().unwrap();
                *buffer = samples;
                0 // Success
                  // Return file size and format validity
            }
            Err(_) => -1,
        }
    }
    pub fn generate_grain_envelope(&self, size: usize) {
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
        overlap: f32,
        pitch: f32) {
        let mut params = self.params.lock().unwrap();
        params.grain_start = (start.clamp(0, 1) as f32 * params.specs.filesize as f32) as usize;
        params.grain_duration = duration;
        params.grain_overlap = overlap.clamp(1.0, 2.0) as f32;
        params.grain_pitch = pitch.clamp(0.1, 2.0) as f32 * params.specs.sample_rate as f32;
    }
    
    pub fn play_audio(&mut self) -> i32 {
        let active_grains = Arc::new(Mutex::new(Vec::<ActiveGrain>::new()));
        let receiver = self.grain_receiver.clone();
        let host = cpal::default_host();

        let output_device = match host.default_output_device() {
            Some(device) => device,
            None => {
                eprintln!("No output device found");
                return -1;
            }
        };

        let config = match output_device.default_output_config() {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Could not get default output config: {}", e);
                return -1;
            }
        };

        let grains_arc = Arc::clone(&active_grains);

        let stream = match output_device.build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // Add grains from the receiver to the active grains list
                while let Ok(grain_data) = receiver.try_recv() {
                    let mut grains = grains_arc.lock().unwrap();
                    grains.push(ActiveGrain::new(grain_data));
                }

                // Generate audio samples for playback
                let mut grains = grains_arc.lock().unwrap();
                for frame in data.chunks_mut(2) {
                    let mut mix_sample = 0.0;
                    for grain in grains.iter_mut() {
                        mix_sample += grain.next_sample();
                    }
                    frame[0] = mix_sample;
                    frame[1] = mix_sample;
                }
                grains.retain(|g| !g.is_finished());
            },
            move |err| {
                eprintln!("Stream error: {}", err);
            },
            None
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

        0
    }
}

// -------------------------------------
// ACTIVE GRAIN HELPER STRUCT
// -------------------------------------
pub struct ActiveGrain {
    samples: Vec<f32>,
    position: usize,
}
impl ActiveGrain {
    fn new(samples: Vec<f32>) -> Self {
        Self {
            samples,
            position: 0,
        }
    }

    fn is_finished(&self) -> bool {
        self.position >= self.samples.len()
    }

    fn next_sample(&mut self) -> f32 {
        if self.is_finished() {
            0.0
        } else {
            let s = self.samples[self.position];
            self.position += 1;
            s
        }
    }

}

#[allow(dead_code)]
fn linear_interpolation(buffer: &[f32], x: f32) -> f32 {
    // x is the fractional index. E.g. 12.3 => index0=12, index1=13, frac=0.3
    let index0 = x.floor() as usize;
    let index1 = index0 + 1;
    let frac = x - (index0 as f32);

    let s0 = if index0 < buffer.len() { buffer[index0] } else { 0.0 };
    let s1 = if index1 < buffer.len() { buffer[index1] } else { 0.0 };

    let out = s0 + (s1 - s0) * frac;
    out
}

fn four_point_interpolation(buffer: &[f32], x: f32) -> f32 {
    if buffer.is_empty() {
        return 0.0;
    }

    let i = x.floor() as isize;
    let frac = x - (i as f32);

    let x0 = i-1;
    let x1 = i;
    let x2 = i+1;
    let x3 = i+2;

    let s0 = *buffer.get(x0.max(0) as usize).unwrap_or(&0.0);
    let s1 = *buffer.get(x1.max(0) as usize).unwrap_or(&0.0);
    let s2 = *buffer.get(x2.max(0) as usize).unwrap_or(&0.0);
    let s3 = *buffer.get(x3.max(0) as usize).unwrap_or(&0.0);

    let c1 = 0.5 * (s2 - s0);
    let c2 = s0 - 2.5 * s1 + 2.0 * s2 - 0.5 * s3;
    let c3 = -0.5 * s0 + 1.5 * s1 - 1.5 * s2 + 0.5 * s3;

    let frac2 = frac * frac;
    let frac3 = frac2 * frac;

    s1 + c1 * frac + c2 * frac2 + c3 * frac3
}

// -------------------------------------
// FRONT-END SPECIFIC API
// -------------------------------------
#[allow(unused_imports)]
use std::ffi::CString;
use std::os::raw::{c_char, c_int};
#[allow(unused_imports)]
use std::ptr;

#[no_mangle]
pub extern "C" fn create_synth() -> *mut GranularSynth {
    let synth = Box::new(GranularSynth::new());
    Box::into_raw(synth)
}

#[no_mangle]
pub extern "C" fn destroy_synth(ptr: *mut GranularSynth) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(ptr);
    }
}

#[no_mangle]
pub extern "C" fn load_audio_from_file(
    synth_ptr: *mut GranularSynth,
    file_path: *const c_char,
    ) -> c_int {
    let synth = unsafe {
        assert!(!synth_ptr.is_null());
        &*synth_ptr
    };
    
    if file_path.is_null() {
        return -1;
    }
    let c_str = unsafe {std::ffi::CStr::from_ptr(file_path)};
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    let result = 
        synth.load_audio_from_file(
            path_str.as_bytes().as_ptr(), path_str.len());
    result
}

#[no_mangle]
pub extern "C" fn generate_grain_envelope(
    synth_ptr: *mut GranularSynth,
    size: usize,
    ) {
    let synth = unsafe {
        assert!(!synth_ptr.is_null());
        &*synth_ptr
    };
    synth.generate_grain_envelope(size);
}

#[no_mangle]
pub extern "C" fn set_params(
    synth_ptr: *mut GranularSynth,
    start: usize,
    duration: usize,
    overlap: f32,
    pitch: f32,
    ){
    let synth = unsafe {
        assert!(!synth_ptr.is_null());
        &*synth_ptr
    };
    synth.set_params(start, duration, overlap, pitch);
}

#[no_mangle]
pub extern "C" fn set_grain_start(
    synth_ptr: *mut GranularSynth, 
    start: usize
    ) {
    let synth = unsafe {
        assert!(!synth_ptr.is_null());
        &*synth_ptr
    };
    let mut params = synth.params.lock().unwrap();
    params.grain_start = 
        (start.clamp(0, 1) as f32 * params.specs.filesize as f32) as usize;
}

#[no_mangle]
pub extern "C" fn set_grain_duration(
    synth_ptr: *mut GranularSynth, 
    duration: usize
    ) {
    let synth = unsafe {
        assert!(!synth_ptr.is_null());
        &*synth_ptr
    };
    let mut params = synth.params.lock().unwrap();
    params.grain_duration = duration;
}

#[no_mangle]
pub extern "C" fn set_grain_pitch(
    synth_ptr: *mut GranularSynth, 
    pitch: f32
    ) {
    let synth = unsafe {
        assert!(!synth_ptr.is_null());
        &*synth_ptr
    };
    let mut params = synth.params.lock().unwrap();
    params.grain_pitch = pitch.clamp(0.1, 2.0) as f32 * params.specs.sample_rate as f32;
}

#[no_mangle]
pub extern "C" fn set_overlap(
    synth_ptr: *mut GranularSynth, 
    overlap: f32
    ) {
    let synth = unsafe {
        assert!(!synth_ptr.is_null());
        &*synth_ptr
    };
    let mut params = synth.params.lock().unwrap();
    params.grain_overlap = overlap.clamp(1.0, 2.0);
}

#[no_mangle]
pub extern "C" fn play_audio(
    synth_ptr: *mut GranularSynth
    ) -> c_int {
    let synth = unsafe {
        assert!(!synth_ptr.is_null());
        &mut *synth_ptr
    };
    synth.play_audio()
}

#[no_mangle]
pub extern "C" fn start_scheduler(
    synth_ptr: *mut GranularSynth
    ){
    let synth = unsafe {
        assert!(!synth_ptr.is_null());
        &mut *synth_ptr
    };
    synth.start_scheduler();
}

#[no_mangle]
pub extern "C" fn stop_scheduler(
    synth_ptr: *mut GranularSynth
    ) {
    let synth = unsafe {
        assert!(!synth_ptr.is_null());
        &*synth_ptr
    };
    synth.stop_scheduler();
}

// -------------------------------------
// TESTS
// -------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_four_point_interpolation() {
        let buf = vec![0.0, 1.0, 2.0, 3.0, 4.0];

        assert_eq!(four_point_interpolation(&buf, 2.0), 2.0);

        let val = four_point_interpolation(&buf, 2.5);
        assert!(val > 2.4 && val < 2.6, "val={}", val);

        let val2 = four_point_interpolation(&buf, 0.0);
        assert_eq!(val2,  0.0);

        let val3 = four_point_interpolation(&buf, 4.0);
        assert_eq!(val3, 4.0);
    }

    #[test]
    fn test_envelope_generation() {
        let synth = GranularSynth::new();
        synth.generate_grain_envelope(1024);
        let env = synth.get_grain_envelope();
        assert_eq!(env.len(), 1024);
        // Basic checks for boundaries
        assert!(env[0] >= 0.0 && env[0] < 1.0);
        assert!(env[1023] >= 0.0 && env[1023] < 1.0);
    }

    #[test]
    fn test_load_invalid_file() {
        let synth = GranularSynth::new();
        let path_str = "non_existent_file.wav";
        let bytes = path_str.as_bytes();
        let result = synth.load_audio_from_file(bytes.as_ptr(), bytes.len());
        assert_eq!(result, -1);
    }

    #[test]
    fn test_process_grain_with_4point_interpolation() {
        let synth = GranularSynth::new();
        {
            let mut buf = synth.source_array.lock().unwrap();
            *buf = (0..44100).map(|i| i as f32).collect();
        }

        synth.generate_grain_envelope(128);

        let voice = GrainVoice::new(0.0, 1.0, 1.0);
        let params = synth.params.lock().unwrap();

        let source = synth.get_source_array();
        let env = synth.get_grain_envelope();
        let out = voice.process_grain(&source, &env, &params);

        assert_eq!(out.len(), 4410);

        let max_val = out.iter().cloned().fold(f32::MIN, f32::max);
        assert!(max_val > 0.0);
    }


}