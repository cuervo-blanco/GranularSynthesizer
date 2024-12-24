# GranularSynth: Rust Backend 

---

## STRUCTURES AND PARAMETERS

### `Specs`
**Purpose**: Defines audio specifications. It is set by the loaded audio file.
Maybe in future can be set by user.
- `sample_rate`: The number of audio samples per second (Hz).
- `channels`: Number of audio channels (e.g., mono = 1, stereo = 2).
- `filesize`: Total number of samples in the loaded audio file.

### `GrainParams`
**Purpose**: Configures granular synthesis parameters. This is modifiable by
the user.
- `grain_start`: Starting position of the grain (in samples).
- `grain_duration`: Length of the grain (in samples).
- `grain_overlap`: Overlap factor for scheduling grains.
- `grain_pitch`: Pitch adjustment multiplier.
- `specs`: Embeds `Specs` structure for sample rate and channel information.

---

## CORE COMPONENTS

### `GrainVoice`
**Purpose**: Represents a single grain voice.
- **Methods**:
  - `new`: Constructs a new `GrainVoice` with starting position, pitch, and duration.
  - `process_grain`: Generates audio data for a grain by applying an envelope 
  and pitch scaling to a source audio array.

---

## SYNTHESIS ENGINE

### `GranularSynth`
**Purpose**: Manages granular synthesis and audio playback.
- **Components**:
  - `source_array`: Stores the source audio samples.
  - `grain_env`: Stores the grain envelope (amplitude shaping).
  - `grain_voices`: Maintains active grain voices.
  - `params`: Holds synthesis parameters (e.g., grain duration, pitch).
  - `counter`: Tracks the active grain voice index.
  - `should_stop`: Atomic flag to control the scheduler thread.
  - `grain_sender` and `grain_receiver`: Channels for communicating grain data.

- **Methods**:
  - `new`: Initializes the `GranularSynth` instance with default settings.
  - `calculate_metro_time_in_ms`: Computes the interval between grain triggers in milliseconds.
  - `start_scheduler`: Begins a thread to schedule grains based on the computed interval.
  - `stop_scheduler`: Requests the scheduler to stop execution.
  - `increment_counter`: Cycles through grain voices in a round-robin fashion.
  - `route_to_grainvoice`: Assigns new parameters to a grain voice and processes its output.
  - `generate_random_parameters`: Produces randomized starting positions and pitch adjustments.
  - `load_audio_from_file`: Loads audio data from a WAV file into the source array.
  - `generate_grain_envelope`: Creates a cosine-based amplitude envelope for shaping grains.
  - `play_audio`: Plays the synthesized audio using the `cpal` library.

---

## UTILITY FUNCTIONS

### `ActiveGrain`
**Purpose**: Tracks grain playback state.
- **Methods**:
  - `new`: Creates a new active grain from a buffer of samples.
  - `is_finished`: Indicates if all grain samples have been played.
  - `next_sample`: Outputs the next sample and increments the playback position.

### `four_point_interpolation`
**Purpose**: Performs advanced interpolation for smooth audio sample reading.
- Uses four adjacent points for accuracy and smoothness.

### `linear_interpolation`
**Purpose**: Simplifies interpolation using two adjacent points for smooth transitions.

---

## TESTS
**Purpose**: Validates core functionality and ensures robustness.
- `test_four_point_interpolation`: Confirms the accuracy of four-point interpolation.
- `test_envelope_generation`: Verifies the envelope generation process.
- `test_load_invalid_file`: Ensures correct handling of invalid audio files.
- `test_process_grain_with_4point_interpolation`: Tests the grain processing pipeline.

---

END OF FUNCTIONAL SPECIFICATION.

