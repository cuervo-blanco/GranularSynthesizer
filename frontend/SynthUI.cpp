#include "SynthUI.h"
#include "AudioSettingsDialog.h" 
#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QFileDialog>
#include <QPainterPath>
#include <QMessageBox>
#include <QGraphicsPathItem>
#include <QGraphicsRectItem>
#include <QPen>
#include <algorithm>
#include <cmath>

// Constructor
SynthUI::SynthUI(QWidget *parent) : QWidget(parent) {

    QVBoxLayout *mainLayout = new QVBoxLayout(this);

    // Buttons
    loadFileButton = new QPushButton("Load WAV", this);
    connect(loadFileButton, &QPushButton::clicked, 
            this, &SynthUI::onLoadFileClicked);
    mainLayout->addWidget(loadFileButton);

    playButton = new QPushButton("Play", this);
    connect(playButton, &QPushButton::clicked, 
            this, &SynthUI::onPlayAudioClicked);
    playButton->setEnabled(false); 
    mainLayout->addWidget(playButton);

    stopButton = new QPushButton("Stop", this);
    connect(stopButton, &QPushButton::clicked,
            this, &SynthUI::onStopAudioClicked);
    stopButton->setEnabled(false); 
    mainLayout->addWidget(stopButton);

    recordButton = new QPushButton("Record", this);
    connect(recordButton, &QPushButton::clicked, 
            this, &SynthUI::onRecordClicked);
    recordButton->setEnabled(false); 
    mainLayout->addWidget(recordButton);

    stopRecordingButton = new QPushButton("Stop Recording", this);
    connect(stopRecordingButton, &QPushButton::clicked,
            this, &SynthUI::onStopRecordingClicked);
    stopRecordingButton->setEnabled(false); 
    mainLayout->addWidget(stopRecordingButton);

    // Menu Bar
    QMenuBar *menuBar = new QMenuBar(this);
    QMenu *fileMenu = menuBar->addMenu("File");

    QAction *loadAction = new QAction("Load", this);
    fileMenu->addAction(loadAction);
    connect(loadAction, &QAction::triggered, 
            this, &SynthUI::onLoadFileClicked);
    mainLayout->setMenuBar(menuBar);

    QAction *settingsAction = new QAction("Audio Settings" , this);
    fileMenu->addAction(settingsAction);
    connect(settingsAction, &QAction::triggered, 
            this, &SynthUI::onAudioSettingsClicked);

    
    // Sliders
    QHBoxLayout *sliderLayout = new QHBoxLayout();
    grainStartLabel = new QLabel("Grain Start", this);
    grainStartSlider = new QSlider(Qt::Horizontal, this);
    grainStartSlider->setRange(0,100);
    grainStartSlider->setValue(0);
    connect(grainStartSlider, &QSlider::sliderReleased, 
            this, &SynthUI::onGrainStartReleased);
    connect(grainStartSlider, &QSlider::valueChanged, 
            this, &SynthUI::onGrainStartValueChanged);
    sliderLayout->addWidget(grainStartLabel);
    sliderLayout->addWidget(grainStartSlider);
    grainStartSlider->setEnabled(false); 

    grainDurationLabel = new QLabel("Grain Duration", this);
    grainDurationSlider = new QSlider(Qt::Horizontal, this);
    grainDurationSlider->setRange(50, 1000);
    grainDurationSlider->setValue(100);
    connect(grainDurationSlider, &QSlider::sliderReleased, 
            this, &SynthUI::onGrainDurationReleased);
    connect(grainDurationSlider, &QSlider::valueChanged, 
            this, &SynthUI::onGrainDurationValueChanged);
    sliderLayout->addWidget(grainDurationLabel);
    sliderLayout->addWidget(grainDurationSlider);
    grainDurationSlider->setEnabled(false); 

    grainPitchLabel = new QLabel("Grain Pitch", this);
    grainPitchSlider = new QSlider(Qt::Horizontal, this);
    grainPitchSlider->setRange(1, 20);
    grainPitchSlider->setValue(10);
    connect(grainPitchSlider, &QSlider::sliderReleased, 
            this, &SynthUI::onGrainPitchReleased);
    connect(grainPitchSlider, &QSlider::valueChanged, 
            this, &SynthUI::onGrainPitchValueChanged);
    sliderLayout->addWidget(grainPitchLabel);
    sliderLayout->addWidget(grainPitchSlider);
    grainPitchSlider->setEnabled(false); 

    overlapLabel = new QLabel("Overlap", this);
    overlapSlider = new QSlider(Qt::Horizontal, this);
    overlapSlider->setRange(10, 20);
    overlapSlider->setValue(15);
    connect(overlapSlider, &QSlider::sliderReleased, 
            this, &SynthUI::onOverlapReleased);
    connect(overlapSlider, &QSlider::valueChanged, 
            this, &SynthUI::onOverlapValueChanged);
    sliderLayout->addWidget(overlapLabel);
    sliderLayout->addWidget(overlapSlider);
    overlapSlider->setEnabled(false); 

    mainLayout->addLayout(sliderLayout);

    // Waveform 
    waveformLabel = new QLabel("Audio Waveform:", this);
    mainLayout->addWidget(waveformLabel);

    waveformView = new QGraphicsView(this);
    waveformScene = new QGraphicsScene(this);
    waveformView->setScene(waveformScene);
    //waveformView->setFixedHeight(150);
    waveformView->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Expanding);
    mainLayout->addWidget(waveformView, 3);

    m_waveformPathItem = new QGraphicsPathItem();
    m_waveformPathItem->setPen(QPen(Qt::blue, 1.0));
    waveformScene->addItem(m_waveformPathItem);

    m_grainRectItem = new QGraphicsRectItem();
    m_grainRectItem->setPen(QPen(Qt::red, 1.0));
    m_grainRectItem->setBrush(Qt::NoBrush);
    waveformScene->addItem(m_grainRectItem);

    // Envelope
    grainEnvelopeLabel = new QLabel("Grain Envelope:", this);
    mainLayout->addWidget(grainEnvelopeLabel);

    grainEnvelopeView = new QGraphicsView(this);
    grainEnvelopeScene = new QGraphicsScene(this);
    grainEnvelopeView->setScene(grainEnvelopeScene);
    //grainEnvelopeView->setFixedHeight(100);
    grainEnvelopeView->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Expanding);
    mainLayout->addWidget(grainEnvelopeView, 1);

    synthPtr = create_synth();
    enginePtr = create_audio_engine(synthPtr);
    set_default_output_device(enginePtr);

    updateEnvelopeDisplay();
    setLayout(mainLayout);
    setWindowTitle("Granular Synthesizer");
    onGrainStartValueChanged();
    onGrainDurationValueChanged();
    onGrainPitchValueChanged();
    onOverlapValueChanged();
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
}

void SynthUI::onAudioSettingsClicked() {
    AudioSettingsDialog dialog(this, enginePtr);
    if (dialog.exec() == QDialog::Accepted) {
        int ret = audio_engine_start(enginePtr);
        if (ret != 0) {
            QMessageBox::critical(this, "Engine Error", "Engine failed to start");
            return;
        }
    }
}

void SynthUI::onRecordClicked() {
    if (!enginePtr) {
        QMessageBox::warning(this, "Error", "No audio engine available!");
        return;
    }
    //stop_scheduler(synthPtr);
    //audio_engine_stop(enginePtr);
    //destroy_audio_engine(enginePtr);
    //enginePtr = nullptr;

    //enginePtr = create_audio_engine(synthPtr);
    //audio_engine_start(enginePtr);
    //start_scheduler(synthPtr);

    QString filePath = QFileDialog::getSaveFileName(
            this,
            tr("Save Recording As"), 
            QString(), 
            tr("Wav Files (*.wav)")
    );
    if (filePath.isEmpty()) {
        return;
    }

    QByteArray ba = filePath.toUtf8();
    int result = record(enginePtr, ba.constData());
    if (result == 0) {
        printf("Sucessfully started recording");
    }
    if (result != 0) {
        QMessageBox::critical(this, "Record Error", "Failed to begin recording");
    } else {
        recordButton->setEnabled(false);
        stopRecordingButton->setEnabled(true);
    }
}

void SynthUI::onStopRecordingClicked() {
    if (!enginePtr) return;
    int result = stop_recording(enginePtr);
    if (result == 0) {
        printf("Sucessfully stopped recording");
    }
    if (result != 0) {
        QMessageBox::critical(this, "Record Error", "Failed to stop recording!");
    } else {
        recordButton->setEnabled(true);
        stopRecordingButton->setEnabled(false);
    }
}

void SynthUI::onLoadFileClicked() {
    loadedFilePath = QFileDialog::getOpenFileName(
            this,
            tr("Open Audio File"),
            QString(),
            tr("WAV Files (*.wav)")
    );

    if (loadedFilePath.isEmpty()) {
        return;
    }

    if (!synthPtr) {
        QMessageBox::warning(this, "Error", "No synth created!");
        return;
    }
    int masterSampleRate = get_master_sample_rate(enginePtr);
    int result = load_audio_from_file(
            synthPtr, 
            loadedFilePath.toStdString().c_str(), 
            masterSampleRate
    );
    if (result != 0) {
        QMessageBox::critical(this, "Load Error", "Failed to load WAV");
        return;
    }

    generate_grain_envelope(synthPtr, 2048);
    
    grainStartSlider->setEnabled(true); 
    grainDurationSlider->setEnabled(true); 
    grainPitchSlider->setEnabled(true); 
    overlapSlider->setEnabled(true); 

    playButton->setEnabled(true); 
    recordButton->setEnabled(true);

    grainStartSlider->setValue(0);
    grainDurationSlider->setValue(100);
    grainPitchSlider->setValue(10);
    overlapSlider->setValue(15);

    SourceArray array = get_source_array(synthPtr);
    std::vector<float> fullSamples(array.length);
    for (size_t i = 0; i < array.length; ++i) {
        fullSamples[i] = array.data[i];
    }
    free_source_array(array);

    downsampleWaveform(fullSamples, m_downsampledWaveform, 2048);
    drawFullWaveformOnce();
    updateGrainSelectionRect();
}

void SynthUI::onGrainStartReleased() {
    int value = grainStartSlider->value();
    float normalizedStart = static_cast<float>(value) / 100.0f;
    set_grain_start(synthPtr, normalizedStart);
    if (isPlaying) {
        onPlayAudioClicked();
    }
}
void SynthUI::onGrainStartValueChanged() {
    // Convert the 0 - 100 into a clock timer... somehow.
    int value = grainStartSlider->value();
    if (value < 10) {
        grainStartLabel->setText(QString("Grain Start:  %1").arg(value));
    } else {
        grainStartLabel->setText(QString("Grain Start: %1").arg(value));
    }
    updateGrainSelectionRect();
}

void SynthUI::onGrainDurationReleased() {
    int duration = grainDurationSlider->value();
    set_grain_duration(synthPtr, duration);
    if (isPlaying) {
        onPlayAudioClicked();
    }
}
void SynthUI::onGrainDurationValueChanged() {
    int value = grainDurationSlider->value();
    if (value < 1000) {
        grainDurationLabel->setText(QString("Grain Duration:   %1").arg(value));
    } else if (value < 10000) {
        grainDurationLabel->setText(QString("Grain Duration:  %1").arg(value));
    } else {
        grainDurationLabel->setText(QString("Grain Duration: %1").arg(value));
    }
    updateGrainSelectionRect();
}

void SynthUI::onGrainPitchReleased() {
    float value = static_cast<float>(grainPitchSlider->value()) / 10.0f;
    if (!synthPtr) {
        // Do something?
        return;
    }
    set_grain_pitch(synthPtr, value);
    updateGrainSelectionRect();
    if (isPlaying == true) {
        onPlayAudioClicked();
    }
}
void SynthUI::onGrainPitchValueChanged() {
    float value = static_cast<float>(grainPitchSlider->value()) / 10.0f;
    if (value < 1) {
        grainPitchLabel->setText(QString("Grain Pitch:  %1").arg(value));
    } else {
        grainPitchLabel->setText(QString("Grain Pitch: %1").arg(value));
    }
}

void SynthUI::onOverlapReleased() {
    int value = overlapSlider->value();
    float overlap = static_cast<float>(value) / 10.0f;
    if (!synthPtr) {
        // Do something?
        return;
    }
    set_overlap(synthPtr, overlap);
    updateGrainSelectionRect();
    if (isPlaying == true) {
        onPlayAudioClicked();
    }
}
void SynthUI::onOverlapValueChanged() {
    int value = overlapSlider->value();
    int overlap = value * 10 - 100;
    if (value < 10) {
        overlapLabel->setText(QString("Overlap: %1").arg(overlap));
    } else {
        overlapLabel->setText(QString("Overlap: %1").arg(overlap));
    }
}

void SynthUI::onPlayAudioClicked() {
    if (!synthPtr) {
        synthPtr = create_synth();
    }
    if (!enginePtr) {
        QMessageBox::critical(this, "Audio Error", "Failed to create audio engine!");
        return;
    }
    start_scheduler(synthPtr);

    int result = audio_engine_start(enginePtr);
    if (result != 0) {
        QMessageBox::critical(this, "Audio Playback Error",
                "Failed to start audio playback. Please check your audio device.");
        stop_scheduler(synthPtr);
        return;
    }

    isPlaying = true;
    playButton->setEnabled(false); 
    stopButton->setEnabled(true); 

    float normalizedStart = static_cast<float>(grainStartSlider->value()) / 100.0f;
    float normalizedPitch = static_cast<float>(grainPitchSlider->value()) / 10.0f;
    float normalizedOverlap= static_cast<float>(overlapSlider->value()) / 10.0f;
    
    set_params(
        synthPtr,
        normalizedStart,
        static_cast<size_t>(grainDurationSlider->value()),
        normalizedOverlap,
        normalizedPitch
    );
}

void SynthUI::onStopAudioClicked() {
    if (!enginePtr) return;
    stop_scheduler(synthPtr);
    audio_engine_stop(enginePtr);
    isPlaying = false;
    stopButton->setEnabled(false); 
    playButton->setEnabled(true); 
}

void SynthUI::drawFullWaveformOnce(){
    if (m_downsampledWaveform.empty()) {
        m_waveformPathItem->setPath(QPainterPath());
        waveformScene->setSceneRect(0, 0, waveformView->width(), waveformView->height());
        return;
    }

    double sceneWidth  = waveformView->width();
    double sceneHeight = waveformView->height();
    double step = m_downsampledWaveform.size() / sceneWidth;

    QPainterPath path;
    path.moveTo(0, sceneHeight / 2.0);

    for (double x = 0; x < sceneWidth; x++)
    {
        size_t index = static_cast<size_t>(x * step);
        if (index >= m_downsampledWaveform.size()) break;
        double sampleVal = m_downsampledWaveform[index];
        double y = (sceneHeight / 2.0) - (sampleVal * (sceneHeight / 2.0));
        path.lineTo(x, y);
    }

    m_waveformPathItem->setPath(path);
    m_waveformPathItem->setPen(QPen(Qt::blue));
    waveformScene->setSceneRect(0, 0, sceneWidth, sceneHeight);
}
void SynthUI::updateGrainSelectionRect()
{
    if (!synthPtr) {
        m_grainRectItem->setRect(0,0,0,0);
        return;
    }
    SourceArray array = get_source_array(synthPtr);
    size_t totalSamples = array.length;
    free_source_array(array);

    if (totalSamples == 0) {
        m_grainRectItem->setRect(0,0,0,0);
        return;
    }

    double sceneWidth  = waveformView->width();
    double sceneHeight = waveformView->height();

    double fractionStart = static_cast<double>(grainStartSlider->value()) / 100.0;
    int sample_rate = get_sample_rate(synthPtr);
    double grainDurationSamples = static_cast<double>(grainDurationSlider->value()) / 1000.0 * sample_rate;

    double fractionDur = grainDurationSamples / static_cast<double>(totalSamples);
    if (fractionDur > 1.0) fractionDur = 1.0;

    double startX = fractionStart * sceneWidth;
    double widthX = fractionDur   * sceneWidth;
    if (startX < 0) startX = 0;
    if ((startX + widthX) > sceneWidth) {
        widthX = sceneWidth - startX;
    }

    m_grainRectItem->setRect(startX, 0, widthX, sceneHeight);
}

void SynthUI::updateEnvelopeDisplay() {
    grainEnvelopeScene->clear();
    if (!synthPtr) {
        grainEnvelopeScene->setSceneRect(0, 0, waveformView->width(), waveformView->height());
        return;
    }
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
    if (!synthPtr) {
        return;
    }
    int sample_rate = get_sample_rate(synthPtr);
    double startX = (grainStartSample / (double)totalSamples) * sceneWidth;
    double endX = 
        (((grainStartSample + grainDuration) / 1000) * sample_rate / (double)totalSamples) * sceneWidth;

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
void SynthUI::downsampleWaveform(
        const std::vector<float>& fullData,
        std::vector<float>& outDownsampled,
        size_t targetSize
        ) {
    outDownsampled.clear();
    if (fullData.empty() || targetSize == 0) return;

    outDownsampled.reserve(targetSize);
    size_t step = std::max<size_t>(1, fullData.size() / targetSize);

    for (size_t i = 0; i < targetSize; i++) {
        size_t index = i * step;
        if (index >= fullData.size()) break;
        outDownsampled.push_back(fullData[index]);
    }
}

void SynthUI::resizeEvent(QResizeEvent* event){
    QWidget::resizeEvent(event); // call base
    drawFullWaveformOnce();      // re-draw waveform path at new size
    updateGrainSelectionRect();  // reposition rectangle
}
