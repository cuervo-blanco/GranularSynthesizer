#ifndef SYNTHUI_H
#define SYNTHUI_H

#include <QWidget>
#include <QPushButton>
#include <QSlider>
#include <QLabel>
#include <QGraphicsView>
#include <QFileDialog>
#include <QGraphicsScene>

extern "C" {
    void load_audio_from_file(const char* file_path);
    void generate_grain_envelope(int frequency);
    void play_audio();
}

class SynthUI : public QWidget {
    Q_OBJECT
    Q_DISABLE_COPY(SynthUI)

public:
    explicit SynthUI(QWidget *parent = nullptr);
    ~SynthUI();

private slots:
    void onLoadFileClicked();
    void onGrainStartChanged(int value);
    void onGrainDurationChanged(int value);
    void onGrainPitchChanged(int value);
    void onOverlapChanged(int value);
    void onPlayAudioClicked();

private:
    QLabel *waveformLabel;
    QGraphicsView *waveformView;
    QGraphicsScene *waveformScene;

    QLabel *grainEnvelopeLabel;
    QGraphicsView *grainEnvelopeView;
    QGraphicsScene *grainEnvelopeScene;
    
    QPushButton *loadFileButton;
    QPushButton *playButton;

    QSlider *grainStartSlider;
    QSlider *grainDurationSlider;
    QSlider *grainPitchSlider;
    QSlider *overlapSlider;

    QLabel *grainStartLabel;
    QLabel *grainDurationLabel;
    QLabel *grainPitchLabel;
    QLabel *overlapLabel;

    QString loadedFilePath;

    void updateWaveformDisplay();
    void updateEnvelopeDisplay();
};

#endif // SYNTHUI_H

