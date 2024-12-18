#include "SynthUI.h"
#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QFileDialog>
#include <QGraphicsRectItem>

SynthUI::SynthUI(QWidget *parent) : QWidget(parent) {
    QVBoxLayout *mainLayout = new QVBoxLayout(this);

    loadFileButton = new QPushButton("Load Audio File", this);
    connect(loadFileButton, &QPushButton::clicked, this, &SynthUI::onLoadFileClicked);
    mainLayout->addWidget(loadFileButton);

    waveformLabel = new QLabel("Audio Waveform:", this);
    mainLayout->addWidget(waveformLabel);

    waveformView = new QGraphicsView(this);
    waveformScene = new QGraphicsScene(this);
    waveformView->setScene(waveformScene);
    waveformView->setFixedHeight(150);

    grainEnvelopeLabel = new QLabel("Grain Envelope:", this);
    mainLayout->addWidget(grainEnvelopeLabel);

    grainEnvelopeView = new QGraphicsView(this);
    grainEnvelopeScene = new QGraphicsScene(this);
    grainEnvelopeView->setScene(grainEnvelopeScene);
    grainEnvelopeView->setFixedHeight(100);
    mainLayout->addWidget(grainEnvelopeView);

    QHBoxLayout *sliderLayout = new QHBoxLayout();

    grainStartLabel = new QLabel("Grain Start", this);
    grainStartSlider = new QSlider(Qt::Horizontal, this);
    grainStartSlider->setRange(0,1);
    connect(grainStartSlider, &QSlider::valueChanged, this, &SynthUI::onGrainStartChanged);

    sliderLayout->addWidget(grainStartLabel);
    sliderLayout->addWidget(grainStartSlider);

    grainDurationLabel = new QLabel("Grain Duration", this);
    grainDurationSlider = new QSlider(Qt::Horizontal, this);
    grainDurationSlider->setRange(100, 5000);
    connect(grainDurationSlider, &QSlider::valueChanged, this, &SynthUI::onGrainDurationChanged);

    sliderLayout->addWidget(grainDurationLabel);
    sliderLayout->addWidget(grainDurationSlider);

    grainPitchLabel = new QLabel("Grain Pitch", this);
    grainPitchSlider = new QSlider(Qt::Horizontal, this);
    grainPitchSlider->setRange(0.1, 2);
    connect(grainPitchSlider, &QSlider::valueChanged, this, &SynthUI::onGrainPitchChanged);

    sliderLayout->addWidget(grainPitchLabel);
    sliderLayout->addWidget(grainPitchSlider);

    overlapLabel = new QLabel("Overlap", this);
    overlapSlider = new QSlider(Qt::Horizontal, this);
    overlapSlider->setRange(1, 2);
    connect(overlapSlider, &QSlider::valueChanged, this, &SynthUI::onOverlapChanged);

    sliderLayout->addWidget(overlapLabel);
    sliderLayout->addWidget(overlapSlider);

    mainLayout->addLayout(sliderLayout);

    playButton = QPushButton("PlayAudio", this);
    connect(playButton, &QPushButton::clicked, this, &SynthUI::onPlayAudioClicked);
    mainLayout->addWidget(playButton);

    setLayout(mainLayout);
    setWindowTitle("Granular Synthesizer");
    
}

void onLoadFileClicked() {
    loadedFilePath = QFileDialog::getOpenFileName(this, "Open Audio File", "", "WAV Files (*.wav)");
    if (!loadedFilePath.isEmpty()) {
        load_audio_from_file(loadedFilePath.toStdString().c_str());
        updateWaveformDisplay();
    }
}

void SynthUI::onGrainStartChanged(int value) {
    generate_grain_envelope(2048); 
    updateEnvelopeDisplay();
}

void SynthUI::onGrainDurationChanged(int value) {
    // Handle grain duration update logic
}

void SynthUI::onGrainPitchChanged(int value) {
    // Handle grain pitch update logic
}

void SynthUI::onOverlapChanged(int value) {
    // Handle overlap update logic
}

void SynthUI::onPlayAudioClicked() {
    play_audio();
}

void SynthUI::updateWaveformDisplay() {
    waveformScene->clear();
    waveformScene->addRect(0, 0, 400, 100); // Placeholder for waveform data
}

void SynthUI::updateEnvelopeDisplay() {
    grainEnvelopeScene->clear();
    grainEnvelopeScene->addRect(0, 0, 400, 50); // Placeholder for envelope data
}


