# GranularSynth
Firstly I apologize, half of my commit history at the moment is Git Actions and
creating a CI/CD development environment. I will be including a ready made .dmg
file, but this one is not guaranteed to work in your environment from the
get-go. This is a noob job I admit, but it is the most complex I have developed
at the moment, including stages from dsp development, through sample-by-sample
processing, to a high-level user interface, and finalizing in a environment
setup complete for installation and run.

A ready made compiled .dmg is found in the [Bundle
directory](https://github.com/cuervo-blanco/GranularSynthesizer/frontend/build/Bundles/GranularSynthesizer.dmg)

The Synthesizer is built with a two-part structure:
- **Backend**: Developed in Rust, handling audio synthesis and real-time processing.
- **Frontend**: Written in C++ (Qt framework), providing an interactive graphical interface.

Besides being able to control Granular Synthesis parameters such as duration of
grains, pitch, overlap, and start, the most important feature of this
Application is to export your work into a .wav file.

Unfortunately, at this moment only `.wav` files can be used as source material and
as exporting format, but in the future I plan to update for more formats (mp3, flac, etc.)...
at least for importing.

This code is highly inspired by Andy Farnell's
Designing Sound book's chapter:
[Grain Techniques](https://mitp-content-server.mit.edu/books/content/sectbyfn/books_pres_0/8375/designing_sound.zip/chapter21.html)

I highly recommend reading this if you want to understand the goal of this
application.

## What Needs Work
- Handling multiple formats (mp3, flac, etc.).
- Limited to your system's default audio devices for now...
- Doesn't handle weird or broken audio files gracefully.
- Everything in the backend is in need of some cleaning, and double checking.
- Finishing the UI (uploading images and linking them through QSS)
- Add flags to the install.sh script for specified build (i.e: only build frontend)

There are a two ways to install this App: directly through an executable
provided in the [Releases](https://github.com/cuervo-blanco/GranularSynthesizer/releases) page, 
or cloning the repo and following the steps below.

At this moment only Linux and MacOS are implemented, Windows will be setup
properly soon.

## Installation (Build from Source)

### **1. Clone the Repository**
Clone the repository and navigate to the project directory:
```bash
git clone https://github.com/cuervo-blanco/GranularSynthesizer.git
cd GranularSynthesizer
```
---

### **2. Install Dependencies**

#### **macOS**
1. Install [Homebrew](https://brew.sh) if not already installed:
   ```bash
   /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
   ```
2. Install required tools and libraries:
   ```bash
   brew install rustup cmake qt
   ```
   - `rustup` for Rust and `cargo`.
   - `cmake` for building the C++ frontend.
   - `qt` for the Qt framework (`macdeployqt` is included).

3. Configure Rust:
   ```bash
   rustup install stable
   rustup default stable
   rustup target add x86_64-apple-darwin aarch64-apple-darwin
   cargo install cargo-lipo
   ```

#### **Linux**
1. Install dependencies using your package manager:
   ```bash
   sudo apt-get update
   sudo apt-get install build-essential cmake qt6-base-dev rustc cargo
   ```
   - On other distributions, replace `apt-get` with your package manager (e.g., `dnf`, `pacman`).

2. Configure Rust:
   ```bash
   rustup install stable
   rustup default stable
   rustup target add x86_64-unknown-linux-gnu
   ```
---

### **3. Build the Application**

Run the automated install script to build the application:
```bash
./install.sh
```

#### **Script Options**
- Use `--clean` to clear previous builds:
  ```bash
  ./install.sh --clean
  ```
- Use `--rebuild` to clean and rebuild everything from scratch:
  ```bash
  ./install.sh --rebuild
  ```
---

### **4. Run the Application**
After the build completes, run the app:
```bash
./GranularSynth
```
---

### **5. Notes**
- **Qt Environment Setup:**  
  Ensure the `qmake` and `macdeployqt` commands are in your PATH. 
  If you used Homebrew or a package manager, this should be configured automatically.
  
- **Linux Testing:**  
  Linux compatibility has not been thoroughly tested yet. 
  If you encounter issues, please submit them via [Issues](https://github.com/cuervo-blanco/GranularSynthesizer/issues).



## Want to Help?
Contributions are more than welcome, they are needed! 
So please, fork the repo, play some grains, and submit a pull request when you
have something to provide.

As a sound designer myself, I built this for my own usage. I've worked in 
projects before were I used the Pd patch that Andy Farnell wrote (see above),
but I knew sooner or later that a more streamlined tool that could provided me 
with high quality exports, better interpolation algorithms and ... a Rust
backend (*wink* *wink*) would be a must in my tool kit.

If this app helps you in your works, feel free to share any content. It would be
a chance to showcase what this tool is capable of doing!
