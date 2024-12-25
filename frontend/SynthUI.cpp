#include "SynthUI.h"
#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QFileDialog>
#include <QMessageBox>
#include <QGraphicsRectItem>

// Constructor
SynthUI::SynthUI(QWidget *parent) : QWidget(parent) {
    synthPtr = create_synth();
    enginePtr = create_audio_engine(synthPtr);
    generate_grain_envelope(synthPtr, 2048);

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
    //waveformView->setFixedHeight(150);
    waveformView->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Expanding);
    mainLayout->addWidget(waveformView, 3);

    // Envelope
    grainEnvelopeLabel = new QLabel("Grain Envelope:", this);
    mainLayout->addWidget(grainEnvelopeLabel);

    grainEnvelopeView = new QGraphicsView(this);
    grainEnvelopeScene = new QGraphicsScene(this);
    grainEnvelopeView->setScene(grainEnvelopeScene);
    //grainEnvelopeView->setFixedHeight(100);
    grainEnvelopeView->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Expanding);
    mainLayout->addWidget(grainEnvelopeView, 1);
    
    // Sliders
    QHBoxLayout *sliderLayout = new QHBoxLayout();

    grainStartLabel = new QLabel("Grain Start", this);
    grainStartSlider = new QSlider(Qt::Horizontal, this);
    grainStartSlider->setRange(0,100);
    connect(grainStartSlider, &QSlider::sliderReleased, this, &SynthUI::onGrainStartReleased);
    sliderLayout->addWidget(grainStartLabel);
    sliderLayout->addWidget(grainStartSlider);

    grainDurationLabel = new QLabel("Grain Duration", this);
    grainDurationSlider = new QSlider(Qt::Horizontal, this);
    grainDurationSlider->setRange(100, 10000);
    connect(grainDurationSlider, &QSlider::sliderReleased, this, &SynthUI::onGrainDurationReleased);
    sliderLayout->addWidget(grainDurationLabel);
    sliderLayout->addWidget(grainDurationSlider);

    grainPitchLabel = new QLabel("Grain Pitch", this);
    grainPitchSlider = new QSlider(Qt::Horizontal, this);
    grainPitchSlider->setRange(1, 20);
    grainPitchSlider->setValue(10);
    connect(grainPitchSlider, &QSlider::valueReleased, this, &SynthUI::onGrainPitchReleased);
    sliderLayout->addWidget(grainPitchLabel);
    sliderLayout->addWidget(grainPitchSlider);

    overlapLabel = new QLabel("Overlap", this);
    overlapSlider = new QSlider(Qt::Horizontal, this);
    overlapSlider->setRange(10, 20);
    overlapSlider->setValue(20);
    connect(overlapSlider, &QSlider::valueReleased, this, &SynthUI::onOverlapReleased);
    sliderLayout->addWidget(overlapLabel);
    sliderLayout->addWidget(overlapSlider);

    mainLayout->addLayout(sliderLayout);

    // Buttons
    playButton = new QPushButton("Play Audio", this);
    connect(playButton, &QPushButton::clicked, this, &SynthUI::onPlayAudioClicked);
    mainLayout->addWidget(playButton);

    stopButton = new QPushButton("Stop Audio", this);
    connect(stopButton, &QPushButton::clicked, this, &SynthUI::onStopAudioClicked);
    mainLayout->addWidget(stopButton);

    updateEnvelopeDisplay();
    setLayout(mainLayout);
    setWindowTitle("Granular Synthesizer");
    
}

// Destructor
SynthUI::~SynthUI() {
    if (enginePtr) {
        audio_engine_stop(enginePtr);
        destroy_audio_engine(enginePtr);
        enginePtr = nullptr;
    }
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
    generate_grain_envelope(synthPtr, 2048);
    loadedFilePath = QFileDialog::getOpenFileName(
            this, "Open Audio File", "", "WAV Files (*.wav)"
    );
    grainStartSlider->setValue(0);
    grainDurationSlider->setValue(1000);  // or whatever default
    grainPitchSlider->setValue(10);      // i.e. pitch=1.0
    overlapSlider->setValue(20);         // i.e. overlap=2.0
    set_params(synthPtr, 0, 4410, 2.0f, 1.0f);

    if (!loadedFilePath.isEmpty()) {
        load_audio_from_file(synthPtr, loadedFilePath.toStdString().c_str());
        updateWaveformDisplay();
    }

}

void SynthUI::onGrainStartReleased(int value) {
    grainStartLabel->setText(QString("Grain Start: %1").arg(value));
    float normalizedStart = static_cast<float>(value) / 100.0f;
    set_grain_start(synthPtr, normalizedStart);
    updateWaveformDisplay();
}

void SynthUI::onGrainDurationReleased(int value) {
    grainDurationLabel->setText(QString("Grain Duration: %1").arg(value));
    float duration = static_cast<float>(value);
    set_grain_duration(synthPtr, duration);
    updateWaveformDisplay();
}

void SynthUI::onGrainPitchReleased(int value) {
    grainPitchLabel->setText(QString("Grain Pitch: %1").arg(value));
    float pitch = static_cast<float>(value) / 10.0f;
    set_grain_pitch(synthPtr, pitch);
    updateWaveformDisplay();
}
void SynthUI::onOverlapReleased(int value) {
    overlapLabel->setText(QString("Overlap: %1").arg(value));
    float overlap = static_cast<float>(value) / 10.0f;
    set_overlap(synthPtr, overlap);
    updateWaveformDisplay();
}

void SynthUI::onPlayAudioClicked() {
    start_scheduler(synthPtr);
    int result = audio_engine_start(enginePtr);
    if (result != 0) {
        stop_scheduler(synthPtr);
        QMessageBox::critical(this, "Audio Playback Error",
                              "Failed to start audio playback. Please check your audio device.");
    }
}

void SynthUI::onStopAudioClicked() {
    stop_scheduler(synthPtr);
    audio_engine_stop(enginePtr);
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
        double sceneWidth = waveformView->width();
        double sceneHeight = waveformView->height();
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

        size_t totalSamples = samples.size();
        double fraction = static_cast<double>(grainStartSlider->value()) / 100.0f;
        size_t grainStartSample = fraction * totalSamples;
        size_t grainDuration = grainDurationSlider->value();

        waveformScene->addPath(path, QPen(Qt::blue));
        waveformScene->setSceneRect(0, 0, sceneWidth, sceneHeight);
        drawGrainSelectionRect(waveformScene, sceneWidth, sceneHeight,
                grainStartSample, grainDuration, totalSamples);
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
        for (double x = 0; x < width; x++) {
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

void SynthUI::drawGrainSelectionRect(
        QGraphicsScene* scene,
        double sceneWidth,
        double sceneHeight,
        size_t grainStartSample,
        size_t grainDuration,
        size_t totalSamples
    ) {
    double startX = (grainStartSample / (double)totalSamples) * sceneWidth;
    double endX = 
        ((grainStartSample + grainDuration) / (double)totalSamples) * sceneWidth;

    startX = qBound(0.0, startX, sceneWidth);
    endX = qBound(0.0, endX, sceneWidth);
    double rectWidth = endX - startX;

    QGraphicsRectItem *grainRect = new QGraphicsRectItem(
            startX, 0.0,
            rectWidth, sceneHeight
    );
    grainRect->setPen(QPen(Qt::red, 1.0));
    grainRect->setBrush(Qt::NoBrush);

    scene->addItem(grainRect);
}
