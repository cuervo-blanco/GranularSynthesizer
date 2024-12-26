# GranularSynth

## NAME
**GranularSynth** – A granular audio synthesizer designed for real-time 
experimentation and manipulation of audio files.

## SYNOPSIS
`SynthApp` is an evolving project aimed at providing an intuitive interface 
for granular synthesis. Currently, the app supports basic audio file loading, 
waveform visualization, and parameter adjustments. However, it remains under 
development, with various backend and frontend features requiring refinement.
In the future we hope to make it a plugin to be used during production.

## DESCRIPTION
SynthApp is built with a two-part structure:
- **Backend**: Developed in Rust, handling audio synthesis and real-time processing.
- **Frontend**: Written in C++ (Qt framework), providing an interactive graphical interface.

The app enables users to experiment with granular synthesis through parameterized 
controls like grain start, duration, pitch, and overlap, displayed in a 
dynamically updating UI. 

This code is highly inspired by Andy Farnell's
Designing Sound book's chapter:
[Grain Techniques](https://mitp-content-server.mit.edu/books/content/sectbyfn/books_pres_0/8375/designing_sound.zip/chapter21.html)

I highly recommend reading this if you want to understand the goal of this
application.

You may ask : why a Rust backend and a C++ frontend? Why not all in JUCE?
To which I answer: Where’s the spice? Where’s the drama? JUCE? Too mainstream, too predictable.
All jokes aside, JUCE could have saved me a lot of headaches, but I wanted 
to explore a different way of writing software through a hybrid approach.


## FEATURES
### **Current Functionality**
- Load `.wav` files for granular synthesis.
- Adjustable synthesis parameters:
  - **Grain Start**: Controls the starting point of grains within the audio file.
  - **Grain Duration**: Sets the duration of individual grains.
  - **Grain Pitch**: Adjusts the playback pitch of grains.
  - **Overlap**: Determines the overlap between consecutive grains.
- Visual feedback:
  - **Waveform View**: Displays the loaded audio's waveform.
  - **Envelope View**: Shows the grain envelope.
  - **Selection Rectangles**: Highlights selected grain areas on the waveform.
- Playback control: Play and stop audio playback.

### **Known Issues**
1. UI scaling inconsistencies when resizing the application window.
2. Limited error handling for invalid audio files.
3. Backend grain generation and scheduling need further optimization and edge case handling.
4. Limited audio device support (dependent on system defaults).

## STRUCTURE
### **Backend** (`/backend`)
- Entire backend functionality is encapsulated in `lib.rs`.
- Key components:
  - **GranularSynth**: Core structure for handling synthesis parameters and audio processing.
  - **AudioEngine**: Manages the audio output stream and real-time scheduling.
  - Utilities for waveform interpolation and envelope generation.

### **Frontend** (`/frontend`)
- Core UI logic in:
  - `SynthUI.h`
  - `SynthUI.cpp`
  - `main.cpp`
- Major UI features:
  - Parameter sliders and buttons for interaction.
  - Dynamic waveform and envelope rendering using Qt's `QGraphicsScene`.

## REQUIREMENTS
### **Backend**
- Rust with dependencies: `cpal`, `hound`, `crossbeam-channel`, `dasp-signal`, `rand`.

### **Frontend**
- C++ with Qt framework (version 5 or higher).

### **System**
- Compatible with Linux, macOS, and Windows.

## INSTALLATION
### Backend
1. Install Rust and Cargo from [rust-lang.org](https://www.rust-lang.org/).
2. Build the Rust library:
   ```bash
   cd backend
   cargo build --release
   ```

### Frontend
1. Install Qt (e.g., via [Qt Creator](https://www.qt.io/)).
2. Compile the frontend application:
   ```bash
   cd frontend
   qmake && make
   ```

## USAGE
1. Launch the application from the compiled binary.
2. Load an audio file using the "Load Audio File" button.
3. Adjust synthesis parameters using the sliders.
4. Click "Play Audio" to hear the processed output.
5. Stop playback with the "Stop Audio" button.

## CONTRIBUTING
Contributions are welcome! Please:
1. Fork the repository.
2. Submit pull requests with detailed descriptions.
3. Ensure changes are tested.

## TODO
1. **Backend**
   - Enhance grain scheduling and interpolation techniques.
   - Improve performance for large audio files.
   - Add support for additional audio formats.

2. **Frontend**
   - Improve UI scaling and responsiveness.
   - Enhance visualizations for waveform and envelope displays.
   - Implement more granular control over playback.

3. **General**
   - Comprehensive testing across platforms.
   - Add documentation and tutorials.

## LICENSE
MIT License. See `LICENSE` file for details.

## AUTHORS
cuervo-blanco (Jaime Osvaldo)
