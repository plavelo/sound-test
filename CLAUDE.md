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

Operators in order of precedence (highest to lowest):

| Operator | Meaning | Example | Notes |
|----------|---------|---------|-------|
| `-A` | Negate | `-noise()` - inverted noise | Equivalent to `0.0 - A` |
| `!A` | Thru | `!lowpass() >> lowpass()` | Passes through missing outputs for chaining |
| `A * B` | Multiply/Ring mod | `sine() * noise()` | Ring modulation when both are audio |
| `A * constant` | Amplify | `A * 0.5` | Broadcasts constant to all channels |
| `A + B` | Add/Mix | `A + B` - mix signals | Outputs must match |
| `A + constant` | Add offset | `A + 0.1` | Broadcasts constant to all channels |
| `A - B` | Subtract | `A - B` - subtract B from A | Outputs must match |
| `A - constant` | Subtract offset | `A - 0.1` | Broadcasts constant to all channels |
| `A >> B` | Pipe (serial) | `A >> B` | Output of A → input of B |
| `A & B` | Bus (parallel mix) | `A & B` | Same input to both, outputs mixed |
| `A ^ B` | Branch (parallel split) | `A ^ B` | Same input to both, separate outputs |
| `A \| B` | Stack (independent) | `A \| B` | Independent parallel processing |

### Broadcasting Rules

- Arithmetic with constants broadcasts to any number of channels
- Arithmetic between components requires matching channel counts
- Example: `A * 2.0` works with any A, but `A * B` requires same output count

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

## Input Modalities And Ranges

Standard parameter ranges for FunDSP components:

| Parameter Type | Range/Units | Notes |
|---------------|-------------|-------|
| frequency | Hz | Natural frequency units |
| phase | 0...1 | All oscillators use this normalized range |
| time | seconds | Natural time units |
| audio data | -1...1 | Standard audio range (internal processing may exceed) |
| stereo pan | -1...1 | Left to right positioning |
| pulse width | 0...1 | For pulse wave oscillators |
| control amount | 0...1 | Generic control parameter range |

## Deterministic Pseudorandom Phase

FunDSP uses a deterministic pseudorandom phase system for audio generators:

- Generator phases are seeded from network structure and node location
- Identical networks sound the same separately but different when combined
- `noise() | noise()` creates stereo noise (different phases per channel)
- Helps decorrelate channels and adds "warmth" to envelopes

### Customizing Phase

**Noise components** (white, pink, brown, mls):
```rust
noise().seed(42)  // Custom seed for noise
```

**Oscillators**:
```rust
sine_hz(440.0).phase(0.0)  // Start at zero phase
```

**Runtime phase changes**:
```rust
oscillator.set(Setting::phase(0.5));  // Takes effect on next reset
```

## Builder Methods

FunDSP components support builder pattern methods for customization:

- `.seed(value)` - Set pseudorandom seed for noise generators
- `.phase(value)` - Set initial phase (0...1) for oscillators  
- `.interval(seconds)` - Set sampling interval for envelope functions

Example:
```rust
envelope(|t| exp(-t)).interval(0.01)  // Sample every 10ms
```

## Waveshaping Modes

The `shape(mode)` opcode applies waveshaping distortion. Available modes:

- `Tanh(hardness)` - Hyperbolic tangent distortion
- `Atan(hardness)` - Arctangent distortion  
- `Softsign(hardness)` - Polynomial alternative to tanh
- `Clip(hardness)` - Hard clipping distortion
- `ClipTo(min, max)` - Clip to custom range
- `Crush(levels)` - Bit crushing effect
- `SoftCrush(levels)` - Smooth bit crushing
- `Adaptive::new(timescale, inner)` - Adaptive normalizing distortion

All shapes with hardness parameter have slope of 1 at origin when hardness = 1.
