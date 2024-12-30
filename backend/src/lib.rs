use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
#[allow(unused_imports)]
use cpal::{SampleRate, Stream};
use hound;
use hound::WavWriter;
use crossbeam_channel::{Sender, Receiver};
#[allow(unused_imports)]
use dasp_signal::{self as signal, Signal};
use rand::Rng;
use std::{
    sync::{Arc, Mutex},
    f32::consts::PI,
    time::{Duration, Instant},
    sync::atomic::AtomicBool,
    thread,
    sync::atomic::Ordering,
    fs::File,
    io::BufWriter,
};


// -------------------------------------
// SPECS, PARAMS
// -------------------------------------
pub struct Specs {
    pub sample_rate: u32,
    pub channels: u16,
    pub filesize: usize,
}

pub struct GrainParams {
    pub grain_start: f32,
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
    interpolation: Interpolation
}
#[derive(Clone)]
pub enum Interpolation {
    FourPoint,
    Sinc,
    Cubic,
    Linear
}

impl GrainVoice {
    pub fn new(mystart: f32, mypitch: f32, mydur: f32) -> Self {
        Self {
            mystart,
            mypitch,
            mydur,
            interpolation: Interpolation::Sinc,
        }
    }

    pub fn process_grain(
        &self,
        source_array: &[f32],
        grain_env: &[f32],
        grain_params: &GrainParams,
    ) -> Vec<f32> {
        // How many samples in this grain?
        let (sample_rate, _channels) = (
            grain_params.specs.sample_rate,
            grain_params.specs.channels
            );
        // total # of samples for this grain
        let duration_in_samples =
            (self.mydur * grain_params.grain_duration as f32) 
            / 1000.0 
            * sample_rate as f32;
        
        let base_source_start = grain_params.grain_start + self.mystart;
        let playback_rate = self.mypitch * grain_params.grain_pitch;
        // let total_duration_samples = duration_in_samples / playback_rate;
        let mut output = vec![0.0; duration_in_samples as usize];

        for i in 0..duration_in_samples as usize {
            // ----------------------------
            // 1) Envelope ramp
            // ----------------------------
            // env_pos goes from 0..1 across duration_in_samples
            let env_pos = i as f32 / (duration_in_samples as f32);
            // Map env_pos [0..1] -> [0..grain_env.len()-1] 
            let env_index_float = env_pos * (grain_env.len() as f32 - 1.0);
            // let env_index = env_index_float.floor() as usize;
            // Later do linear interpolation for a smoother read
            let envelope_value = match self.interpolation {
                Interpolation::FourPoint => {
                        four_point_interpolation(grain_env, env_index_float)
                },
                Interpolation::Sinc => {
                        sinc_interpolation(grain_env, env_index_float)
                },
                Interpolation::Cubic => {
                        cubic_interpolation(grain_env, env_index_float)
                },
                Interpolation::Linear => {
                        linear_interpolation(grain_env, env_index_float)
                },
            };
            // ----------------------------
            // 2) Source read ramp
            // ----------------------------
            // Each sample, we move forward by `playback_rate` (set by pitch)
            // starting from `base_source_start`.
            let source_index_float = base_source_start + (i as f32 * playback_rate);
            let source_value = match self.interpolation {
                Interpolation::FourPoint => {
                        four_point_interpolation(source_array, source_index_float)
                },
                Interpolation::Sinc => {
                        sinc_interpolation(source_array, source_index_float)
                },
                Interpolation::Cubic => {
                        cubic_interpolation(grain_env, env_index_float)
                },
                Interpolation::Linear => {
                        linear_interpolation(grain_env, env_index_float)
                }
            };
            output[i] = source_value * envelope_value;
        }
        output
    }
}
// -------------------------------------
// RECORDING FORMATS
// -------------------------------------
pub struct ExportSettings {
    pub channels: u16,
    pub sample_rate: u32,
    pub bit_depth: u16,
    pub sample_format: hound::SampleFormat,
    pub format: String, // "wav", "mp3", "flac", etc.
    pub format_specific: Option<FormatSpecificSettings>,
}

pub enum FormatSpecificSettings {
    WavSettings {},               // WAV doesn't need extra parameters
    Mp3Settings { bitrate: u32 }, // MP3-specific
    FlacSettings { compression: u8 }, // FLAC-specific
}

pub enum Writers {
    WavWriter(hound::WavWriter<BufWriter<File>>),
    // In the future, we could add Mp3Writer(...), FlacWriter(...), etc.
}

fn dummy_placeholder() -> WavWriter<BufWriter<File>> {
    let temp_file = std::fs::File::create("/dev/null")
        .expect("Failed to create a dummy file");
    let spec = hound::WavSpec {
        bits_per_sample: 16,
        channels: 2,
        sample_format: hound::SampleFormat::Int,
        sample_rate: 16000,
    };
    let buf_writer = BufWriter::new(temp_file);
    WavWriter::new(buf_writer, spec)
        .expect("Failed to create a dummy WavWriter")
}
// -------------------------------------
// AUDIO ENGINE STRUCT
// -------------------------------------
pub struct AudioEngine {
    synth: Arc<GranularSynth>,
    output_device: Option<cpal::platform::Device>,
    stream: Option<cpal::Stream>,
    export_settings: ExportSettings,
    is_recording: bool,
    writer: Option<Arc<Mutex<Writers>>>,
}

impl AudioEngine {
    pub fn new(
        synth: Arc<GranularSynth>,
        export_settings: ExportSettings
        ) -> Self {
        let host = cpal::default_host();
        let default_output_device = match host.default_output_device() {
            Some(device) => Some(device),
            None => {
                eprintln!("No default output device found!");
                None
            }
        };
        AudioEngine {
            synth,
            output_device: default_output_device,
            stream: None,
            export_settings,
            is_recording: false,
            writer: None,
        }
    }
    // ---------------
    // SETTINGS
    // ---------------
    pub fn set_sample_rate(&mut self, sample_rate: u32){
        self.export_settings.sample_rate = sample_rate;
    }

    pub fn set_bit_depth(&mut self, bit_depth: u16) {
        self.export_settings.bit_depth = bit_depth;
    }

    pub fn set_buffer_size(&mut self, _buffer_size: usize) {
        // Add buffer size logic here
        // TODO: possibly set custom buffer size in the cpal config
        // pub struct StreamConfig {
            // pub channels: ChannelCount,
            // pub sample_rate: SampleRate,
            // **pub buffer_size: BufferSize,
        // } 
    }

    pub fn set_file_format(&mut self, fmt: &str){
        self.export_settings.format = fmt.to_string();
    }

    pub fn set_bit_rate(&mut self, bitrate: u32){
        if let Some(FormatSpecificSettings::Mp3Settings { bitrate: ref mut b }) =
            self.export_settings.format_specific
        {
            *b = bitrate;
        } else {
            // Possibly override or create new FormatSpecificSettings::Mp3Settings
            self.export_settings.format_specific = 
                Some(FormatSpecificSettings::Mp3Settings { bitrate });
        }
    }
    
    pub fn set_flac_compression(&mut self, level: u8) {
        if let Some(FormatSpecificSettings::FlacSettings { ref mut compression }) =
            self.export_settings.format_specific
        {
            *compression = level;
        } else {
            self.export_settings.format_specific =
                Some(FormatSpecificSettings::FlacSettings { compression: level });
        }
    }

    pub fn get_output_devices(&self) -> Vec<(usize, String)> {
        let host = cpal::default_host();
        let mut results = Vec::new();
        let devices = match host.output_devices() {
            Ok(devs) => devs,
            Err(e) => {
                eprintln!("Failed to get output devices: {}", e);
                return results;
            }
        };
        for (index, device) in devices.enumerate() {
            let name = device.name().unwrap_or("Unknown".to_string());
            results.push((index, name));
        }
        results
    }

    pub fn set_output_device_by_index(
        &mut self,
        index: usize
    ) -> Result<(), String> {
        let host = cpal::default_host();
        let devices = host.output_devices().map_err(|e| e.to_string())?;
        let dev = devices.skip(index).next().ok_or("Invalid device index")?;
        self.output_device = Some(dev);
        Ok(())
    }

    pub fn set_default_output_device(&mut self) -> Result<(), String> {
        let host = cpal::default_host();
        let default_dev = host
            .default_output_device()
            .ok_or("No default output device found!")?;
        self.output_device = Some(default_dev);
        Ok(())
    }

    pub fn get_default_output_device(&self) -> String {
        self.output_device
            .as_ref()
            .map(|device| device
                .name()
                .unwrap_or_else(|_| "Unknown device".to_string()))
            .unwrap_or_else(|| "No device available".to_string())
    }

    // ----------------------
    // PLAYBACK
    // ----------------------
    pub fn start(&mut self) -> i32 {
        // if Stream already exists, drop it and re-build
        if let Some(existing) = self.stream.take() {
            drop(existing);
        }
        let output_device = match &self.output_device {
            Some(device) => device,
            None => {
                println!("No output device available");
                return -1;
            }
        };
        // Perhaps here build the config based on the user settings?
        let config = match output_device.default_output_config() {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Could not get default output config: {}", e);
                return -1;
            }
        };

        let active_grains = Arc::new(Mutex::new(Vec::<ActiveGrain>::new()));
        let grains_arc = Arc::clone(&active_grains);
        
        let receiver_for_callback = Arc::clone(&self.synth.grain_receiver);

        // For recording
        let writer_for_callback = self.writer.clone();
        let is_recording_for_callback = self.is_recording;

        let stream = match output_device.build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // 1. gather any newly scheduled grains
                while let Ok(grain_data) = receiver_for_callback.try_recv() {
                    let mut vec = grains_arc.lock().unwrap();
                    vec.push(ActiveGrain::new(grain_data));
                }

                // 2. fill the audio buffer
                let mut grains = grains_arc.lock().unwrap();
                for frame in data.chunks_mut(2) {
                    let mut mix_sample = 0.0;
                    for g in grains.iter_mut() {
                        mix_sample += g.next_sample();
                    }
                    for sample in frame.iter_mut() {
                        *sample = mix_sample;
                    }
                    //frame[0] = mix_sample;
                    //frame[1] = mix_sample;
                }
                grains.retain(|g| !g.is_finished());

                // Recording
                if is_recording_for_callback {
                    if let Some(writer_arc) = &writer_for_callback {
                        let mut writer_lock = writer_arc.lock().unwrap();
                        match &mut *writer_lock {
                            Writers::WavWriter(wav_writer) => {
                                for &sample in data.iter() {
                                    // Convert f32 -> i16. 
                                    // Could do float WAV directly if you want "SampleFormat::Float"
                                    let sample_i16 = (sample * i16::MAX as f32) as i16;
                                    if let Err(e) = wav_writer.write_sample(sample_i16) {
                                        eprintln!("Failed to write sample: {}", e);
                                    }
                                }
                            }
                        }
                    }
                }
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

        // Store it so it doesn't drop
        self.stream = Some(stream);

        0
    }

    pub fn stop(&mut self) {
        self.stream.take();
    }

    // ----------------------
    // RECORDING
    // ----------------------
    pub fn record(&mut self, output_path: &str) -> Result<(), String> { 
         if self.is_recording {
             return Err("Already recording!".to_string());
         }
         match self.export_settings.format.as_str() {
             "wav" => {
                let spec = hound::WavSpec {
                    channels: self.export_settings.channels,
                    sample_rate: self.export_settings.sample_rate,
                    bits_per_sample: self.export_settings.bit_depth,
                    sample_format: match self.export_settings.sample_format {
                        hound::SampleFormat::Float => hound::SampleFormat::Float,
                        hound::SampleFormat::Int => hound::SampleFormat::Int,
                    },
                };
                let file = File::create(output_path).map_err(|e| e.to_string())?;
                let bw = BufWriter::new(file);
                let wav_writer = hound::WavWriter::new(bw, spec)
                    .map_err(|e| e.to_string())?;
                self.writer = Some(Arc::new(Mutex::new(Writers::WavWriter(wav_writer))));
            },
            "mp3" => {
                return Err("MP3 recording not implemented yet.".to_string());
            },
            "flac" => {
                return Err("FLAC recording not implemented yet.".to_string());
            },
            other => {
                return Err(format!("Unsupported format for recording: {}", other));
            },
         };

         self.is_recording = true;

         Ok(())
    }

    pub fn stop_recording(&mut self) -> Result<(), String> {
        if !self.is_recording {
            return Err("Not currently recording!".to_string());
        }
        self.is_recording = false;

        // Finalize the writer
        if let Some(writer_arc) = self.writer.take() {
            let mut writer_lock = writer_arc.lock().unwrap();
            let writer = std::mem::replace(&mut *writer_lock, Writers::WavWriter(dummy_placeholder()));

            match writer {
                Writers::WavWriter(wav_writer) => {
                    wav_writer.finalize().map_err(|e| e.to_string())?;
                }
            }
        }

        Ok(())
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
    pub grain_sender: Arc<Sender<Vec<f32>>>,
    pub grain_receiver: Arc<Receiver<Vec<f32>>>,
}
impl GranularSynth {
    // Maybe add a function to set the numbrt of grain_voices
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
                grain_start: 0.0,
                grain_duration: 4410,
                grain_overlap: 2.0,
                grain_pitch: 1.0,
                specs: specs,
            })),
            counter: Arc::new(Mutex::new(0)),
            should_stop: Arc::new(AtomicBool::new(false)),
            grain_sender: Arc::new(s),
            grain_receiver: Arc::new(r), 
        }
    }

    pub fn calculate_metro_time_in_ms(&self) -> f32 {
        let params = self.params.lock().unwrap();
        let interval_ms = params.grain_duration as f32 / params.grain_overlap;
        interval_ms
    }

    pub fn start_scheduler(&self) {
        let synth_clone = self.clone_for_thread(); 
        self.should_stop.store(false, Ordering::SeqCst);
        thread::spawn(move || {
            let metro_time = synth_clone.calculate_metro_time_in_ms();
            let interval = Duration::from_millis(metro_time as u64);
            let mut next_time = Instant::now();
            println!("synth_clone.should_stop: {:?}", synth_clone.should_stop);
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
        // join or handle the thread
    }

    // One detail: to move `GranularSynth` into a thread’s closure, 
    // you need to clone the Arc fields. 
    fn clone_for_thread(&self) -> GranularSynth {
        //let (new_sender, new_receiver) = crossbeam_channel::unbounded();
        GranularSynth {
            source_array: Arc::clone(&self.source_array),
            grain_env: Arc::clone(&self.grain_env),
            grain_voices: Arc::clone(&self.grain_voices),
            params: Arc::clone(&self.params),
            counter: Arc::clone(&self.counter),
            should_stop: Arc::clone(&self.should_stop),
            grain_receiver: Arc::clone(&self.grain_receiver),
            grain_sender: Arc::clone(&self.grain_sender),
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
        //eprintln!("Grain data length = {}", grain_data.len());

        self.grain_sender.send(grain_data).ok();
    }

    pub fn generate_random_parameters() -> (f32, f32) {
        // For added variability
        // Perhaps add a new parameter for varying the randomness
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
                // Change bit rate
                let samples: Vec<f32> = reader
                    .samples::<i16>()
                    .map(|s| s.unwrap_or(0) as f32 / 32768.0)
                    .collect();

                let filesize = samples.len();
                let mut params = self.params.lock().unwrap();
                params.specs.filesize = filesize;
                // Resample? 
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
        start: f32 ,
        duration: usize, 
        overlap: f32,
        pitch: f32) {
        let mut params = self.params.lock().unwrap();
        params.grain_start = start.clamp(0.0, 1.0) as f32 * params.specs.filesize as f32;
        params.grain_duration = duration;
        params.grain_overlap = overlap.clamp(1.0, 2.0) as f32;
        params.grain_pitch = pitch.clamp(0.1, 2.0) as f32;
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
// -------------------------------------
// HELPER FUNCTIONS
// -------------------------------------

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

fn sinc(x: f32) -> f32 {
    if x == 0.0 {
        1.0
    } else {
        (x * std::f32::consts::PI).sin() / (x * std::f32::consts::PI)
    }
}

fn sinc_interpolation(buffer: &[f32], x: f32) -> f32 {
    let n = buffer.len() as isize;
    let i = x.floor() as isize;
    let frac = x - i as f32;

    let mut result = 0.0;
    let mut normalization = 0.0;

    for j in -2..=2 {
        let index = i + j;
        if index >= 0 && index < n {
            let sample = buffer[index as usize];
            let sinc_value = sinc(frac - j as f32);
            result += sample * sinc_value;
            normalization += sinc_value;
        }
    }

    if normalization != 0.0 {
        result / normalization
    } else {
        0.0 // Handle the case where normalization is zero
    }
}

fn cubic_interpolation(buffer: &[f32], x: f32) -> f32 {
    let n = buffer.len() as isize;
    let i = x.floor() as isize;
    let frac = x - i as f32;

    let mut p = [0.0; 4];

    for j in -1..=2 {
        let index = i + j;
        if index >= 0 && index < n {
            p[(j + 1) as usize] = buffer[index as usize];
        } else {
            p[(j + 1) as usize] = 0.0; // Out of bounds, use 0.0
        }
    }

    let a = (p[2] - p[0]) * 0.5;
    let b = p[1] - p[0] - a;
    let c = p[2] - p[1] - a;
    let d = p[1];

    // Cubic polynomial: a * frac^3 + b * frac^2 + c * frac + d
    let frac2 = frac * frac;
    let frac3 = frac2 * frac;

    a * frac3 + b * frac2 + c * frac + d
}

fn four_point_interpolation(buffer: &[f32], x: f32) -> f32 {
    if buffer.is_empty() {
        eprintln!("Buffer is empty [interpolation error]");
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
// C API
// -------------------------------------
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

#[no_mangle]
pub extern "C" fn set_params(
    synth_ptr: *mut GranularSynth,
    start: f32,
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
    start: f32
) {
    let synth = unsafe {
        assert!(!synth_ptr.is_null());
        &*synth_ptr
    };
    let mut params = synth.params.lock().unwrap();
    params.grain_start = 
        start.clamp(0.0, 1.0) as f32 * params.specs.filesize as f32;
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
    params.grain_pitch = pitch.clamp(0.1, 2.0) as f32;
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
pub extern "C" fn create_audio_engine(
    synth_ptr: *mut GranularSynth
) -> *mut AudioEngine {
    unsafe {
        assert!(!synth_ptr.is_null());
        let synth_ref = &*synth_ptr;
        let arc_synth = Arc::new(synth_ref.clone_for_thread());

        // Provide a default ExportSettings
        let default_export_settings = ExportSettings {
            channels: 2,
            sample_rate: 44100,
            bit_depth: 16,
            sample_format: hound::SampleFormat::Int,
            format: "wav".to_string(),
            format_specific: Some(FormatSpecificSettings::WavSettings {}),
        };

        let engine = AudioEngine::new(arc_synth, default_export_settings);
        Box::into_raw(Box::new(engine))
    }
}

#[no_mangle]
pub extern "C" fn audio_engine_start(engine_ptr: *mut AudioEngine) -> c_int {
    let engine = unsafe {
        assert!(!engine_ptr.is_null());
        &mut *engine_ptr
    };
    engine.start() as c_int
}

#[no_mangle]
pub extern "C" fn audio_engine_stop(engine_ptr: *mut AudioEngine) {
    let engine = unsafe {
        assert!(!engine_ptr.is_null());
        &mut *engine_ptr
    };
    engine.stop();
}


#[no_mangle]
pub extern "C" fn destroy_audio_engine(engine_ptr: *mut AudioEngine) {
    if engine_ptr.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(engine_ptr);
    }
}

#[no_mangle]
pub extern "C" fn set_sample_rate(
    engine_ptr: *mut AudioEngine,
    sample_rate: u32,
){
    let engine = unsafe {
        assert!(!engine_ptr.is_null());
        &mut *engine_ptr
    };
    engine.set_sample_rate(sample_rate);
}

#[no_mangle]
pub extern "C" fn set_file_format(
    engine_ptr: *mut AudioEngine,
    fmt: *const c_char,
) {
    let engine = unsafe {
        assert!(!engine_ptr.is_null());
        &mut *engine_ptr
    };
    if fmt.is_null() {
        eprintln!("set_file_format received null string pointer");
        return;
    }
    let c_str = unsafe {std::ffi::CStr::from_ptr(fmt)};
    let format_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Invalid UTF-8 in set_file_format");
            return;
        }
    };
    engine.set_file_format(format_str);
}

#[no_mangle]
pub extern "C" fn set_bit_depth(
    engine_ptr: *mut AudioEngine,
    bitdepth: u16,
){
    let engine = unsafe {
        assert!(!engine_ptr.is_null());
        &mut *engine_ptr
    };
    engine.set_bit_depth(bitdepth);
}

#[no_mangle]
pub extern "C" fn set_bit_rate(
    engine_ptr: *mut AudioEngine,
    bitrate: u32,
) {
    let engine = unsafe {
        assert!(!engine_ptr.is_null());
        &mut *engine_ptr
    };
    engine.set_bit_rate(bitrate);
}

#[no_mangle]
pub extern "C" fn set_flac_compression(
    engine_ptr: *mut AudioEngine,
    level: u8,
) {
    let engine = unsafe {
        assert!(!engine_ptr.is_null());
        &mut *engine_ptr
    };
    engine.set_flac_compression(level);
}

#[no_mangle]
pub extern "C" fn set_output_device(
    engine_ptr: *mut AudioEngine,
    index: usize,
) -> c_int {
    let engine = unsafe {
        assert!(!engine_ptr.is_null());
        &mut *engine_ptr
    };
    match engine.set_output_device_by_index(index) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[repr(C)]
pub struct DeviceInfo {
    pub index: usize,
    pub name: *const c_char,
}

#[repr(C)]
pub struct DeviceList {
    pub devices: *const DeviceInfo,
    pub count: usize,
}

#[no_mangle]
pub extern "C" fn get_output_devices(
    engine_ptr: *mut AudioEngine,
    ) -> DeviceList {
    let engine = unsafe {
        assert!(!engine_ptr.is_null());
        &mut *engine_ptr
    };
    let device_vec = engine.get_output_devices();
    let mut device_infos: Vec<DeviceInfo> = device_vec
        .iter()
        .map(|(idx, name)| {
            let c_string = CString::new(name.clone())
                .unwrap_or_else(|_| CString::new("Unknown").unwrap());
            DeviceInfo {
                index: *idx,
                name: c_string.into_raw(),
            }
        }).collect();
    let ptr = device_infos.as_mut_ptr();
    let count = device_infos.len();
    std::mem::forget(device_infos);

    DeviceList { devices: ptr, count }
}

#[no_mangle]
pub extern "C" fn free_device_list(device_list: DeviceList) {
    if device_list.devices.is_null() {
        return;
    }

    let devices = unsafe { 
        Vec::from_raw_parts(
            device_list.devices as *mut DeviceInfo, 
            device_list.count, 
            device_list.count
        ) 
    };

    for device in devices {
        if !device.name.is_null() {
            unsafe {
                let _ = CString::from_raw(device.name as *mut c_char);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn get_default_output_device(
    engine_ptr: *mut AudioEngine
) -> *mut c_char {
    let engine = unsafe {
        assert!(!engine_ptr.is_null());
        &mut *engine_ptr
    };
    let dev_str = engine.get_default_output_device();
    let c_str = std::ffi::CString::new(dev_str).unwrap();
    c_str.into_raw()
}

#[no_mangle]
pub extern "C" fn set_default_output_device(
    engine_ptr: *mut AudioEngine,
    ) -> c_int {
    let engine = unsafe {
        assert!(!engine_ptr.is_null());
        &mut *engine_ptr
    };
    let result = engine.set_default_output_device();
    match result {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn record (
    engine_ptr: *mut AudioEngine,
    output_path: *const c_char,
    ) -> c_int {
    let engine = unsafe {
        assert!(!engine_ptr.is_null());
        &mut *engine_ptr
    };
    if output_path.is_null() {
        return -1;
    }
    let c_str = unsafe { std::ffi::CStr::from_ptr(output_path) };
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    match engine.record(path_str) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn stop_recording (
    engine_ptr: *mut AudioEngine,
    ) -> c_int {
    let engine = unsafe {
        assert!(!engine_ptr.is_null());
        &mut *engine_ptr
    };
    match engine.stop_recording() {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[repr(C)]
pub struct GrainEnvelope {
    data: *const f32,
    length: usize,
}

#[no_mangle]
pub extern "C" fn get_grain_envelope(
    synth_ptr: *mut GranularSynth
    ) -> GrainEnvelope {
    let synth = unsafe {
        assert!(!synth_ptr.is_null());
        &*synth_ptr
    };
    let grain_envelope = synth.get_grain_envelope();
    let envelope = GrainEnvelope {
        data: grain_envelope.as_ptr(),
        length: grain_envelope.len(),
    };
    std::mem::forget(grain_envelope);
    envelope
}

#[no_mangle]
pub extern "C" fn free_grain_envelope(envelope: GrainEnvelope) {
    unsafe {
        if !envelope.data.is_null() {
            let _ = Vec::from_raw_parts(
                envelope.data as *mut f32,
                envelope.length,
                envelope.length,
            );
        }
    }
}

#[repr(C)]
pub struct SourceArray{
    data: *const f32,
    length: usize,
}

#[no_mangle]
pub extern "C" fn get_source_array(
    synth_ptr: *mut GranularSynth
    ) -> SourceArray {
    let synth = unsafe {
        assert!(!synth_ptr.is_null());
        &*synth_ptr
    };
    let source_array = synth.get_source_array();
    let array = SourceArray {
        data: source_array.as_ptr(),
        length: source_array.len(),
    };
    std::mem::forget(source_array);
    array
}

#[no_mangle]
pub extern "C" fn free_source_array(array: SourceArray) {
    unsafe {
        if !array.data.is_null() {
            let _ = Vec::from_raw_parts(
                array.data as *mut f32,
                array.length,
                array.length,
            );
        }
    }
}

#[no_mangle]
pub extern "C" fn get_sample_rate(synth_ptr: *mut GranularSynth) -> c_int {
    let synth = unsafe {
        assert!(!synth_ptr.is_null());
        &mut *synth_ptr
    };
    let params = synth.params.lock().unwrap();
    params.specs.sample_rate as c_int
}

#[no_mangle]
pub extern "C" fn get_total_channels(synth_ptr: *mut GranularSynth) -> c_int {
    let synth = unsafe {
        assert!(!synth_ptr.is_null());
        &mut *synth_ptr
    };
    let params = synth.params.lock().unwrap();
    params.specs.channels as c_int
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
    #[test]
    fn test_grain_params_edge_cases() {
        let synth = GranularSynth::new();

        synth.set_params(999_999, 0, 0.0, 0.0);

        let params = synth.params.lock().unwrap();
        assert_eq!(params.grain_overlap, 1.0, "Overlap must clamp to 1.0");
        assert_eq!(
            params.grain_pitch,
            params.specs.sample_rate as f32 * 0.1,
            "Pitch must clamp to 0.1 * sample_rate" 
        );
        assert_eq!(params.grain_duration, 0);
        drop(params);

        synth.set_params(0, 44100, 9.0, 999.0);
        let params2 = synth.params.lock().unwrap();
        assert_eq!(params2.grain_overlap, 2.0, "Overlap must clamp to 2.0");
        assert_eq!(
            params2.grain_pitch,
            params2.specs.sample_rate as f32 * 2.0,
            "Pitch must clamp to 2.0 * sample_rate"
        );
    }

    #[test]
    fn test_empty_source_array() {
        let synth = GranularSynth::new();
        {
            let mut buf = synth.source_array.lock().unwrap();
            buf.clear();
        }

        let voice = GrainVoice::new(0.0, 1.0, 1.0);
        let env = synth.get_grain_envelope();
        let params = synth.params.lock().unwrap();
        let out = voice.process_grain(&[], &env, &params);

        assert_eq!(out.len(), 4410);
        assert!(out.iter().all(|&sample| sample == 0.0));
    }

    #[test]
    fn test_tiny_source_array() {
        let synth = GranularSynth::new();
        {
            let mut buf = synth.source_array.lock().unwrap();
            *buf = vec![0.25, 0.5];
        }

        synth.generate_grain_envelope(8);

        let voice = GrainVoice::new(0.0, 1.0, 1.0);
        let params = synth.params.lock().unwrap();
        let out = voice.process_grain(
            &synth.get_source_array(),
            &synth.get_grain_envelope(),
            &params
        );

        assert_eq!(out.len(), 4410);
    }
    #[test]
    fn test_load_stereo_file() {
        let synth = GranularSynth::new();

        // With an actual stereo file, we could test it like so:
        // let path_str = "stereo_test.wav";
        // let bytes = path_str.as_bytes();
        // let result = synth.load_audio_from_file(bytes.as_ptr(), bytes.len());
        // For demonstration, we'll just check that it returns -1 for non-existent file:
        let path_str = "fake_stereo_test.wav";
        let bytes = path_str.as_bytes();
        let result = synth.load_audio_from_file(bytes.as_ptr(), bytes.len());
        assert_eq!(result, -1, "Expected failure for a non-existent stereo file");
    }

    #[test]
    fn test_calculate_metro_time_in_ms() {
        let synth = GranularSynth::new();
        let mut params = synth.params.lock().unwrap();
        params.grain_duration = 4410;
        params.grain_overlap = 1.5;
        drop(params);

        // 1) (grain_duration / 2) / overlap
        // default = (4410/2) / 1.5 = 1470 ms
        let time_ms = synth.calculate_metro_time_in_ms();
        assert!((time_ms - 1470.0).abs() < 1.0, "Expected around 1470 ms");
    }

    #[test]
    fn test_generate_random_parameters() {
        // Just check that (r_a, r_b) are in some plausible range
        // r_a ~ 0..10000, r_b ~ 1.0..(1.0 + 200/10000.0)
        // i.e. r_b ~ 1.0..1.02
        for _ in 0..100 {
            let (r_a, r_b) = GranularSynth::generate_random_parameters();
            assert!(r_a >= 0.0 && r_a < 10001.0, "r_a out of range");
            assert!(r_b >= 1.0 && r_b <= 1.02, "r_b out of range");
        }
    }
    //
    // 6) Simple Smoke Test for Real-Time + Scheduler
    //
    // This demonstrates a "no panic" test. In practice, real-time tests
    // might be excluded in CI or done manually, because they depend on 
    // an audio device, threads, timing, etc.
    //
    #[test]
    fn test_smoke_test_play_audio() {
        let mut synth = GranularSynth::new();

        // Provide a small envelope; or, if you have a test WAV file, load it:
        //   let file_path = "path/to/small_test.wav";
        //   let bytes = file_path.as_bytes();
        //   synth.load_audio_from_file(bytes.as_ptr(), bytes.len());
        synth.generate_grain_envelope(256);

        synth.start_scheduler();

        let play_result = synth.play_audio();
        // If your environment doesn't have an output device, might fail
        // So we won't assert 0 here, just verify no panic:
        eprintln!("play_audio() returned: {}", play_result);

        // Let it run a moment
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Stop scheduler
        synth.stop_scheduler();

        // If we got here without panics, consider it a success
        assert!(true);
    }

    //
    // 7) (Optional) Checking Overlap & Timing in Real-Time
    //    This is more of an integration test. Shown here as a sketch:
    //
    // #[test]
    // fn test_grain_overlap_timing() {
    //     let mut synth = GranularSynth::new();
    //     synth.generate_grain_envelope(128);
    //     // ... load short file, or set up source_array with known data
    //
    //     let (tx, rx) = crossbeam_channel::unbounded::<std::time::Instant>();
    //
    //     // Instead of using the default route_to_grainvoice, you could 
    //     // modify it temporarily to record a timestamp each time a new 
    //     // grain is started:
    //     // e.g., tx.send(Instant::now()).unwrap();
    //
    //     synth.start_scheduler();
    //     synth.play_audio();
    //
    //     // Let it run for a while, collecting timestamps
    //     std::thread::sleep(std::time::Duration::from_secs(2));
    //     synth.stop_scheduler();
    //
    //     // Now analyze the timestamps from rx to see if they're 
    //     // spaced approximately at calculate_metro_time_in_ms() intervals.
    //     // For instance:
    //     let times: Vec<_> = rx.try_iter().collect();
    //     // assert that intervals are close to the expected ms
    // }

}
