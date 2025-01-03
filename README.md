# GranularSynth
The Synthesizer is built with a two-part structure:
- **Backend**: Developed in Rust, handling audio synthesis and real-time processing.
- **Frontend**: Written in C++ (Qt framework), providing an interactive graphical interface.

The app enables you to experiment with granular synthesis using controls like 
grain start, duration, pitch, and overlap.

This code is highly inspired by Andy Farnell's
Designing Sound book's chapter:
[Grain Techniques](https://mitp-content-server.mit.edu/books/content/sectbyfn/books_pres_0/8375/designing_sound.zip/chapter21.html)

I highly recommend reading this if you want to understand the goal of this
application.

Unfortunately, at this moment only `.wav` files can be used as source material and
as exporting material, but in future I plan to update for more formats (mp3, flac, etc.)...
at least for importing.

## What Needs Work
- Handling multiple formats (mp3, flac, etc.).
- Limited to your system's default audio devices for now...
- Doesn't handle weird or broken audio files gracefully.
- Everything in the backend is in need of some cleaning, and double checking.
- Finishing the UI (uploading images and linking them through QSS)
- Add flags to the install.sh script for specified build (i.e: only build frontend)

Hopefully in the near future I will provide a .exe or a .dmg, dockfile or an AppImage,
but for now the only way to install the app is directly by cloning the repo. 
Windows is untested. You'll definately need to install Qt.

## Installation
1. Clone the repo:
   ```bash
   git clone https://github.com/cuervo-blanco/GranularSynthesizer.git
   cd GranularSynth
   ```
2. Run the automated install script:
   ```bash
   ./install.sh
   ```
   - Use `--clean` if you want to clear previous builds.
   - Use `--rebuild` to clean and rebuild everything from scratch.
3. Ensure you have Qt installed. If not, download and install it from [Qt Downloads](https://www.qt.io/download). Make sure `qmake` is available in your PATH.
4. After installation, run the app and start experimenting:
   ```bash
   ./GranularSynth
   ```

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
