#![allow(clippy::precedence)]

use assert_no_alloc::*;
use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample};
use fundsp::hacker::*;

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

// Karplus-Strong acoustic guitar synthesis
fn acoustic_guitar_hz(freq: f32) -> An<impl AudioNode<Inputs = U0, Outputs = U1>> {
    // Generate excitation pulse with noise
    let excitation = white() * envelope(|t| if t < 0.002 { 1.0 } else { 0.0 });

    // Karplus-Strong plucked string synthesis
    // Parameters: frequency, gain per second (decay), high frequency damping
    let plucked_string = excitation >> pluck(freq, 0.996, 0.3);

    // Add body resonance with bandpass filters for acoustic guitar character
    let body_resonance = plucked_string
        >> (pass() &
        bandpass_hz(110.0, 1.5) * 0.15 &  // Low body resonance
        bandpass_hz(200.0, 2.0) * 0.25 &  // Primary body resonance
        bandpass_hz(400.0, 2.5) * 0.2 &   // Mid body resonance
        bandpass_hz(800.0, 3.0) * 0.1); // High frequency brightness

    // Apply natural guitar envelope and final filtering
    body_resonance
        * envelope(|t| (-t * 1.5).exp()) // Natural decay envelope
        >> lowpass_hz(6000.0, 1.0)      // Remove harsh high frequencies
        >> dcblock() // Remove DC offset
}

// Create a single guitar note with timing control
fn guitar_note_timed(
    freq: f32,
    start_time: f64,
    duration: f64,
) -> An<impl AudioNode<Inputs = U0, Outputs = U1>> {
    let guitar = acoustic_guitar_hz(freq)
        * envelope(move |t| {
            if t >= start_time && t < start_time + duration {
                1.0
            } else {
                0.0
            }
        });

    guitar * 0.5
}

fn create_audio_graph() -> An<impl AudioNode<Inputs = U0, Outputs = U2>> {
    // BPM 120 = 0.5 seconds per quarter note
    let note_duration = 0.5;

    // C major scale starting from C4 (MIDI note 60)
    let c_major_scale = [
        60.0, // C4
        62.0, // D4
        64.0, // E4
        65.0, // F4
        67.0, // G4
        69.0, // A4
        71.0, // B4
        72.0, // C5
    ];

    // Create sequence of notes
    let scale_sequence = guitar_note_timed(midi_hz(c_major_scale[0]), 0.0, note_duration)
        + guitar_note_timed(midi_hz(c_major_scale[1]), note_duration, note_duration)
        + guitar_note_timed(
            midi_hz(c_major_scale[2]),
            note_duration * 2.0,
            note_duration,
        )
        + guitar_note_timed(
            midi_hz(c_major_scale[3]),
            note_duration * 3.0,
            note_duration,
        )
        + guitar_note_timed(
            midi_hz(c_major_scale[4]),
            note_duration * 4.0,
            note_duration,
        )
        + guitar_note_timed(
            midi_hz(c_major_scale[5]),
            note_duration * 5.0,
            note_duration,
        )
        + guitar_note_timed(
            midi_hz(c_major_scale[6]),
            note_duration * 6.0,
            note_duration,
        )
        + guitar_note_timed(
            midi_hz(c_major_scale[7]),
            note_duration * 7.0,
            note_duration,
        );

    // Convert to stereo
    let stereo_scale = scale_sequence >> pan(0.0);

    // Add subtle chorus for natural string detuning and width
    let with_chorus = stereo_scale >> (chorus(0, 0.0, 0.002, 0.1) | chorus(1, 0.0, 0.002, 0.1));

    // Add acoustic space with reverb
    let with_reverb = with_chorus >> reverb_stereo(3.0, 2.5, 0.4);

    // Final processing chain
    with_reverb >> (declick() | declick()) >> limiter_stereo(0.9, 2.0)
}

fn save_to_wav(filename: &str) {
    let sample_rate = 44100.0;
    // Duration for 8 quarter notes at BPM 120 = 8 * 0.5 = 4 seconds + some extra for decay
    let duration = 6.0;

    let mut c = create_audio_graph();

    let wave = Wave::render(sample_rate, duration, &mut c);
    let path = std::path::Path::new(filename);
    wave.save_wav32(path)
        .expect(&format!("Could not save {}", filename));

    println!("Saved C major scale to {}", filename);
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

    // Play for 6 seconds to hear the complete C major scale
    std::thread::sleep(std::time::Duration::from_millis(6000));

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
