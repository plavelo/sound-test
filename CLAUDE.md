# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust audio synthesis project that uses real-time audio processing to generate organ sounds. The project demonstrates real-time audio synthesis using the `fundsp` audio framework and `cpal` for cross-platform audio I/O.

## Architecture

- **Main binary**: Single executable in `src/main.rs` that sets up audio pipeline and runs synthesis
- **Audio processing**: Uses `fundsp` DSP framework for synthesis and effects
- **Real-time constraints**: Uses `assert_no_alloc` to ensure no allocations in audio callback
- **Wavetable synthesis**: Custom organ wavetable generated with harmonic series
- **Audio pipeline**: Synthesis → Pan → Chorus → Declick → DC Block → Limiter

## Common Commands

### Build and Run
```bash
cargo build --release  # Build optimized version
cargo run              # Run the application (plays organ sound for 50 seconds)
```

### Development
```bash
cargo check            # Quick syntax/type checking
cargo clippy           # Linting
cargo fmt              # Code formatting
```

### Testing Audio
The application runs a single audio test that plays an organ chord (A3, C#4, E4) for 50 seconds with stereo chorus effects.

## Key Dependencies

- `fundsp`: Audio synthesis framework providing DSP primitives
- `cpal`: Cross-platform audio library for real-time I/O
- `assert_no_alloc`: Ensures real-time safety by preventing allocations in audio callback
- `once_cell`: Thread-safe lazy initialization for wavetable

## Real-time Audio Considerations

- Audio callback must be allocation-free (enforced by `assert_no_alloc`)
- All DSP objects are pre-allocated before starting audio stream
- Wavetable is lazily initialized once and cached globally
- Sample rate is configured dynamically based on audio device capabilities
- Call `allocate()` method before sending components to real-time context
- Use block processing (`process` method) for maximum performance

## FunDSP Graph Notation

FunDSP features a powerful inline graph notation for describing audio processing networks using algebraic operators.

### Core Operators

| Operator | Meaning | Example |
|----------|---------|---------|
| `>>` | Pipe (serial chain) | `A >> B` - output of A feeds input of B |
| `&` | Bus (parallel mix) | `A & B` - same input to both, outputs mixed |
| `^` | Branch (parallel split) | `A ^ B` - same input to both, separate outputs |
| `\|` | Stack (independent) | `A \| B` - independent parallel processing |
| `*` | Multiply/amplify | `A * 0.5` - amplify by 0.5 |
| `+` | Add/mix | `A + B` - mix signals together |

### Example Expressions

```rust
// FM oscillator
sine_hz(f) * f * m + f >> sine()

// Stereo organ with chorus and limiter
organ_hz(440.0) >> pan(0.0) >> chorus(0, 0.015, 0.005, 0.5) >> limiter(0.01, 0.1)

// Parallel filter bank
noise() >> (lowpass_hz(1000.0, 1.0) & bandpass_hz(2000.0, 2.0) & highpass_hz(4000.0, 1.0))
```

## Key FunDSP Components

### Oscillators
- `sine()`, `sine_hz(f)` - Sine wave oscillator
- `saw()`, `saw_hz(f)` - Bandlimited sawtooth
- `square()`, `square_hz(f)` - Bandlimited square wave
- `organ()`, `organ_hz(f)` - Organ-style waveform (emphasizes octave partials)
- `hammond()`, `hammond_hz(f)` - Hammond-style waveform (emphasizes first three partials)

### Filters
- `lowpass_hz(freq, q)` - 2nd order lowpass filter
- `highpass_hz(freq, q)` - 2nd order highpass filter
- `bandpass_hz(freq, q)` - 2nd order bandpass filter
- `bell_hz(freq, q, gain)` - Peaking EQ filter
- `dcblock_hz(freq)` - DC blocking filter

### Effects
- `chorus(seed, separation, variation, modulation)` - Chorus effect
- `reverb_stereo(room_size, reverb_time, damping)` - Stereo reverb
- `limiter(attack, release)` - Look-ahead limiter
- `delay(time)` - Simple delay line

### Noise Sources
- `white()` - White noise
- `pink()` - Pink noise  
- `brown()` - Brown noise

### Utilities
- `pan(position)` - Mono to stereo panner (-1 to 1)
- `declick()`, `declick_s(time)` - Fade-in to prevent clicks
- `pass()` - Pass signal through unchanged
- `dc(value)` - Constant signal generator

## Sample Rate Independence

- Default sample rate: 44.1 kHz
- Use `set_sample_rate(rate)` to change sample rate for component and children
- Parameters use natural units (Hz for frequency, seconds for time)
- Enables easy oversampling with `oversample()` component
