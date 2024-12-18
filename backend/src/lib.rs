use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
#[allow(unused_imports)]
use cpal::{SampleRate, Stream};
use hound;
use dasp_signal::{self as signal, Signal};
use std::sync::{Arc, Mutex};
use std::f32::consts::PI;

pub struct GrainParams {
    pub grain_start: usize,
    pub grain_duration: usize,
    pub grain_overlap: usize,
    pub grain_pitch: f32,
}

pub struct GranularSynth {
    audio_buffer: Arc<Mutex<Vec<f32>>>,
    grain_env: Arc<Mutex<Vec<f32>>>,
    params: Arc<Mutex<GrainParams>>,
}

impl GranularSynth {
    pub fn new() -> Self {
        Self {
            audio_buffer: Arc::new(Mutex::new(vec![])),
            grain_env: Arc::new(Mutex::new(vec![])),
            params: Arc::new(Mutex::new(GrainParams {
                grain_start: 0,
                grain_duration: 4410,
                grain_overlap: 2,
                grain_pitch: 1.0,
            })),
        }
    }

    pub extern "C" fn load_audio_from_file(&self, file_path: &str) -> Result<(), hound::Error> {
        let mut reader = hound::WavReader::open(file_path)?;
        let spec = reader.spec();
        println!(
            "Loaded audio: sample rate = {}, channels = {}",
            spec.sample_rate, spec.channels
            );

        let samples: Vec<f32> = reader
            .samples::<i16>()
            .map(|s| s.unwrap_or(0) as f32 / 32768.0)
            .collect();

        let mut buffer = self.audio_buffer.lock().unwrap();
        *buffer = samples;
        Ok(())
    }

    pub extern "C" fn generate_grain_envelope(&self, size: usize) {
        let mut env = self.grain_env.lock().unwrap();
        env.clear();
        for i in 0..size {
            let x = (i as f32 / size as f32) * 2.0 - 1.0;
            let value = 0.5 + (0.5 * (x * PI).cos());
            env.push(value);
        }
    }

    pub fn get_audio_buffer(&self) -> Vec<f32> {
        self.audio_buffer.lock().unwrap().clone()
    }

    pub fn get_grain_envelope(&self) -> Vec<f32> {
        self.grain_env.lock().unwrap().clone()
    }

    pub fn load_audio(&self) {
        let sample_rate = 44100;
        let frequency = 440.0;
        let sine_wave: Vec<f32> = signal::rate(sample_rate as f64)
            .const_hz(frequency)
            .sine()
            .take(sample_rate * 2)
            .map(|s| s as f32)
            .collect::<Vec<f32>>();

        let mut buffer = self.audio_buffer.lock().unwrap();
        *buffer = sine_wave;
    }

    pub fn set_params(
        &self, 
        start: usize, 
        duration: usize, 
        overlap: usize,
        pitch: f32) {
        let mut params = self.params.lock().unwrap();
        params.grain_start = start;
        params.grain_duration = duration;
        params.grain_overlap = overlap;
        params.grain_pitch = pitch;
    }

    pub extern "C" fn play(&self) -> Stream {
        let audio_buffer = Arc::clone(&self.audio_buffer);
        let params = Arc::clone(&self.params);

        let host = cpal::default_host();
        let output_device = host.default_output_device().expect("No output device found");
        let config = output_device.default_output_config().unwrap();

        let stream = output_device
            .build_output_stream(
                &config.into(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let buffer = audio_buffer.lock().unwrap();
                    let params = params.lock().unwrap();

                    let grain_start = params.grain_start;
                    let grain_duration = params.grain_duration;
                    let grain_pitch = params.grain_pitch;

                    for sample in data.iter_mut() {
                        let index = (grain_start + (
                                *sample as usize % grain_duration
                                )) as f32 * grain_pitch;

                        *sample = if let Some(&value) = buffer.get(index as usize) {
                            value
                        } else {
                            0.0
                        };
                    }
                },
                |err| eprintln!("Error: {}", err),
                None,
            ).unwrap();
        stream.play().unwrap();
        stream
    }
}

#[no_mangle]
pub extern "C" fn rust_generate_sine_wave(frequency: i32) {
    println!("Generating sine wave with frequency: {} Hz", frequency);
    // Placeholder for actual DSP logic
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_granular_synth() {
        let synth = GranularSynth::new();
         synth
            .load_audio_from_file("test_audio.wav")
            .expect("Failed to load audio");

        synth.generate_grain_envelope(2048);

        let audio_buffer = synth.get_audio_buffer();
        let envelope = synth.get_grain_envelope();

        println!("Audio buffer size: {}", audio_buffer.len());
        println!("Grain envelope size: {}", envelope.len());
        
        synth.load_audio();

        synth.set_params(0, 4410, 2, 1.0);

        let _stream = synth.play();

        thread::sleep(Duration::from_secs(5));
    }
}

