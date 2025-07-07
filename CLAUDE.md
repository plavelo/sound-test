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
