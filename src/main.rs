#![allow(clippy::precedence)]

use assert_no_alloc::*;
use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample};
use fundsp::hacker::*;
use once_cell::race::OnceBox;
use std::sync::Arc;

#[cfg(debug_assertions)] // required when disable_release is set (default)
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

#[derive(Parser)]
#[command(name = "sound-test")]
#[command(about = "A Rust audio synthesis program that generates acoustic guitar sounds")]
struct Args {
    #[arg(short = 'o', long = "output", help = "Output WAV file path")]
    output: Option<String>,
}

fn main() {
    let args = Args::parse();
    
    if let Some(output_file) = args.output {
        save_to_wav(&output_file);
        return;
    }
    
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .expect("Failed to find a default output device");
    let config = device.default_output_config().unwrap();

    match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into()).unwrap(),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into()).unwrap(),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into()).unwrap(),
        _ => panic!("Unsupported format"),
    }
}

fn guitar_table() -> Arc<Wavetable> {
    static INSTANCE: OnceBox<Arc<Wavetable>> = OnceBox::new();
    INSTANCE
        .get_or_init(|| {
            let table = Wavetable::new(
                20.0,
                20_000.0,
                4.0,
                // Set phase to enable interpolation with saw, triangle and soft saw wavetables.
                &|i| {
                    if (i & 3) == 3 {
                        0.5
                    } else if (i & 1) == 1 {
                        0.0
                    } else {
                        0.5
                    }
                },
                // Guitar-like harmonic series with decay
                &|_, i| {
                    let decay = (-0.3 * i as f64).exp();
                    let amplitude = if i % 2 == 1 {
                        // Odd harmonics are stronger for guitar-like sound
                        1.0 / (i as f64).sqrt()
                    } else {
                        // Even harmonics are weaker
                        0.5 / (i as f64).sqrt()
                    };
                    amplitude * decay
                },
            );
            Box::new(Arc::new(table))
        })
        .clone()
}

fn guitar() -> An<WaveSynth<U1>> {
    An(WaveSynth::new(guitar_table()))
}

fn guitar_hz(f: f32) -> An<Pipe<Constant<U1>, WaveSynth<U1>>> {
    constant(f) >> guitar()
}

fn create_audio_graph() -> An<impl AudioNode<Inputs = U0, Outputs = U2>> {
    let c = 0.3 * guitar_hz(midi_hz(57.0)); // A3 single note
    let c = c >> pan(0.0);
    // Add reverb for more natural guitar sound
    let c = c >> reverb_stereo(4.0, 3.0, 0.5);
    c >> (declick() | declick()) >> (dcblock() | dcblock()) >> limiter_stereo(1.0, 5.0)
}

fn save_to_wav(filename: &str) {
    let sample_rate = 44100.0;
    let duration = 10.0;
    
    let mut c = create_audio_graph();
    
    let wave = Wave::render(sample_rate, duration, &mut c);
    let path = std::path::Path::new(filename);
    wave.save_wav32(path).expect(&format!("Could not save {}", filename));
    
    println!("Saved audio to {}", filename);
}

fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<(), anyhow::Error>
where
    T: SizedSample + FromSample<f32>,
{
    let sample_rate = config.sample_rate.0 as f64;
    let channels = config.channels as usize;

    let mut c = create_audio_graph();

    c.set_sample_rate(sample_rate);
    c.allocate();

    let mut next_value = move || assert_no_alloc(|| c.get_stereo());

    let err_fn = |err| eprintln!("an error occurred on stream: {err}");

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_value)
        },
        err_fn,
        None,
    )?;
    stream.play()?;

    std::thread::sleep(std::time::Duration::from_millis(10000));

    Ok(())
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> (f32, f32))
where
    T: SizedSample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let sample = next_sample();
        let left = T::from_sample(sample.0);
        let right: T = T::from_sample(sample.1);

        for (channel, sample) in frame.iter_mut().enumerate() {
            if channel & 1 == 0 {
                *sample = left;
            } else {
                *sample = right;
            }
        }
    }
}
