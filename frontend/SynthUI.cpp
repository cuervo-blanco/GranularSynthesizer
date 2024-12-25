#include "SynthUI.h"
#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QFileDialog>
#include <QGraphicsRectItem>

// Constructor
SynthUI::SynthUI(QWidget *parent) : QWidget(parent) {
    synthPtr = create_synth();

    QVBoxLayout *mainLayout = new QVBoxLayout(this);

    loadFileButton = new QPushButton("Load Audio File", this);
    connect(loadFileButton, &QPushButton::clicked, this, &SynthUI::onLoadFileClicked);
    mainLayout->addWidget(loadFileButton);

    // Waveform 
    waveformLabel = new QLabel("Audio Waveform:", this);
    mainLayout->addWidget(waveformLabel);

    waveformView = new QGraphicsView(this);
    waveformScene = new QGraphicsScene(this);
    waveformView->setScene(waveformScene);
    waveformView->setFixedHeight(150);

    // Envelope
    grainEnvelopeLabel = new QLabel("Grain Envelope:", this);
    mainLayout->addWidget(grainEnvelopeLabel);

    grainEnvelopeView = new QGraphicsView(this);
    grainEnvelopeScene = new QGraphicsScene(this);
    grainEnvelopeView->setScene(grainEnvelopeScene);
    grainEnvelopeView->setFixedHeight(100);
    mainLayout->addWidget(grainEnvelopeView);
    
    // Sliders
    QHBoxLayout *sliderLayout = new QHBoxLayout();

    grainStartLabel = new QLabel("Grain Start", this);
    grainStartSlider = new QSlider(Qt::Horizontal, this);
    grainStartSlider->setRange(0,100);
    connect(grainStartSlider, &QSlider::valueChanged, this, &SynthUI::onGrainStartChanged);
    sliderLayout->addWidget(grainStartLabel);
    sliderLayout->addWidget(grainStartSlider);

    grainDurationLabel = new QLabel("Grain Duration", this);
    grainDurationSlider = new QSlider(Qt::Horizontal, this);
    grainDurationSlider->setRange(100, 10000);
    connect(grainDurationSlider, &QSlider::valueChanged, this, &SynthUI::onGrainDurationChanged);
    sliderLayout->addWidget(grainDurationLabel);
    sliderLayout->addWidget(grainDurationSlider);

    grainPitchLabel = new QLabel("Grain Pitch", this);
    grainPitchSlider = new QSlider(Qt::Horizontal, this);
    grainPitchSlider->setRange(1, 20);
    grainPitchSlider->setValue(10);
    connect(grainPitchSlider, &QSlider::valueChanged, this, &SynthUI::onGrainPitchChanged);
    sliderLayout->addWidget(grainPitchLabel);
    sliderLayout->addWidget(grainPitchSlider);

    overlapLabel = new QLabel("Overlap", this);
    overlapSlider = new QSlider(Qt::Horizontal, this);
    overlapSlider->setRange(10, 20);
    overlapSlider->setValue(20);
    connect(overlapSlider, &QSlider::valueChanged, this, &SynthUI::onOverlapChanged);
    sliderLayout->addWidget(overlapLabel);
    sliderLayout->addWidget(overlapSlider);

    mainLayout->addLayout(sliderLayout);

    // Buttons
    playButton = new QPushButton("Play Audio", this);
    connect(playButton, &QPushButton::clicked, this, &SynthUI::onPlayAudioClicked);
    mainLayout->addWidget(playButton);

    stopButton = new QPushButton("Stop Audio", this);
    connect(playButton, &QPushButton::clicked, this, &SynthUI::onStopAudioClicked);
    mainLayout->addWidget(stopButton);

    setLayout(mainLayout);
    setWindowTitle("Granular Synthesizer");
    
}

// Destructor
SynthUI::~SynthUI() {
    if (synthPtr) {
        destroy_synth(synthPtr);
        synthPtr = nullptr;
    }

    delete grainEnvelopeScene;
    delete grainEnvelopeView;
    delete grainPitchSlider;
    delete overlapLabel;
    delete playButton;
}

void SynthUI::onLoadFileClicked() {
    loadedFilePath = QFileDialog::getOpenFileName(
            this, "Open Audio File", "", "WAV Files (*.wav)"
    );
    if (!loadedFilePath.isEmpty()) {
        load_audio_from_file(synthPtr, loadedFilePath.toStdString().c_str());
        updateWaveformDisplay();
    }
}

void SynthUI::onGrainStartChanged(int value) {
    float normalizedStart = static_cast<float>(value) / 100.0f;
    size_t duration = grainDurationSlider->value();
    float overlap = overlapSlider->value() / 10.0f;
    float pitch = static_cast<float>(grainPitchSlider->value()) / 10.0f;

    set_params(synthPtr,
            static_cast<size_t>(normalizedStart),
            duration,
            overlap,
            pitch);
    generate_grain_envelope(synthPtr, 2048);
    updateEnvelopeDisplay();
}

void SynthUI::onGrainDurationChanged(int value) {
    size_t start = grainDurationSlider->value() / 100.0f;
    float duration = static_cast<float>(value);
    float overlap = overlapSlider->value() / 10.0f;
    float pitch = static_cast<float>(grainPitchSlider->value()) / 10.0f;

    set_params(synthPtr,
            static_cast<size_t>(start),
            duration,
            overlap,
            pitch);
    generate_grain_envelope(synthPtr, 2048);
    updateEnvelopeDisplay();
}

void SynthUI::onGrainPitchChanged(int value) {
    size_t start = grainDurationSlider->value() / 100.0f;
    size_t duration = grainDurationSlider->value();
    float overlap = overlapSlider->value() / 10.0f;
    float pitch = static_cast<float>(value) / 10.0f;

    set_params(synthPtr,
            static_cast<size_t>(start),
            duration,
            overlap,
            pitch);
    generate_grain_envelope(synthPtr, 2048);
    updateEnvelopeDisplay();
}

void SynthUI::onOverlapChanged(int value) {
    size_t start = grainDurationSlider->value() / 100.0f;
    size_t duration = grainDurationSlider->value();
    float overlap = static_cast<float>(value) / 10.0f;
    float pitch = static_cast<float>(grainPitchSlider->value()) / 10.0f;

    set_params(synthPtr,
            static_cast<size_t>(start),
            duration,
            overlap,
            pitch);
    generate_grain_envelope(synthPtr, 2048);
    updateEnvelopeDisplay();
}

void SynthUI::onPlayAudioClicked() {
    start_scheduler(synthPtr);
    int result = play_audio(synthPtr);
    if (result != 0) {
        stop_scheduler(synthPtr);
        QMessageBox::critical(this, "Audio Playback Error",
                              "Failed to start audio playback. Please check your audio device.");
    }
}

void SynthUI::onStopClicked() {
    stop_scheduler(synthPtr);
    // Make a Rust function to stop the audio stream!
}

void SynthUI::updateWaveformDisplay() {
    waveformScene->clear();
    
    SourceArray array = get_source_array(synthPtr);
    std::vector<float> samples(array.length);
    for (size_t i = 0; i < array.length; ++i) {
        samples[i] = array.data[i];
    }
    free_source_array(array);

    if (!samples.empty()) {
        double sceneWidth = 400.0;
        double sceneHeight = 100.0;
        double step = samples.size() / sceneWidth;

        QPainterPath path;
        path.moveTo(0, sceneHeight / 2.0);
        for (double x = 0; x < sceneWidth; x++) {
            size_t index = static_cast<size_t>(x * step);
            if (index >= samples.size()) break;
            double sampleVal = samples[index];
            double y = (sceneHeight / 2.0) - (sampleVal * (sceneHeight / 2.0));
            path.lineTo(x, y);
        }

        waveformScene->addPath(path, QPen(Qt::blue));
        waveformScene->setSceneRect(0, 0, sceneWidth, sceneHeight);
    } else {
        // fallback if no samples
        waveformScene->addText("No samples loaded!");
    }
    // This is a fairly minimal approach. If the audio is very large, 
    // we can consider downsampling more aggressively or only showing a portion 
    // to avoid performance issues in the UI thread.

}

void SynthUI::updateEnvelopeDisplay() {
    grainEnvelopeScene->clear();

    GrainEnvelope env = get_grain_envelope(synthPtr);
    std::vector<float> envelope(env.length);
    for (size_t i = 0; i < env.length; ++i) {
        envelope[i] = env.data[i];
    }
    free_grain_envelope(env);

    if (!envelope.empty()) {
        double width = 400.0;
        double height = 50.0;
        double step = static_cast<double>(envelope.size()) / width;

        QPainterPath envPath;
        envPath.moveTo(0, height);
        for (double x = 0; x = width; x++) {
            size_t index = static_cast<size_t>(x * step);
            if (index >= envelope.size()) break;

            double envVal = envelope[index];
            double y = height - (envVal * height);
            envPath.lineTo(x, y);
        }
        
        grainEnvelopeScene->addPath(envPath, QPen(Qt::red));
        grainEnvelopeScene->setSceneRect(0, 0, width, height);
    } else {
        grainEnvelopeScene->addText("No envelope data!");
    }
}


