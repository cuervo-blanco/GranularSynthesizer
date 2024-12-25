#ifndef SYNTHUI_H
#define SYNTHUI_H

#include <cstddef>
#include <QWidget>
#include <QPushButton>
#include <QSlider>
#include <QLabel>
#include <QGraphicsView>
#include <QFileDialog>
#include <QGraphicsScene>
#include <QMessageBox>

#ifdef __cplusplus
extern "C" {
#endif

    typedef struct GranularSynth GranularSynth;
    typedef struct SourceArray {
        const float* data;
        size_t length;
    } SourceArray;

    typedef struct GrainEnvelope {
        const float* data;
        size_t length;
    } GrainEnvelope;

    GranularSynth* create_synth();
    void destroy_synth(GranularSynth* ptr);

    int load_audio_from_file(GranularSynth* ptr, const char* file_path);
    void generate_grain_envelope(GranularSynth* ptr, size_t size);
    int play_audio(GranularSynth* ptr);

    void start_scheduler(GranularSynth* ptr);
    void stop_scheduler(GranularSynth* ptr);

    void set_params(
            GranularSynth* ptr, 
            size_t start, 
            size_t duration, 
            float overlap, 
            float pitch);
    void set_grain_start(GranularSynth* ptr, size_t start);
    void set_grain_duration(GranularSynth* ptr, size_t duration);
    void set_grain_pitch(GranularSynth* ptr, float pitch);
    void set_overlap(GranularSynth* ptr, float overlap);

    SourceArray get_source_array(GranularSynth* ptr);
    void free_source_array(SourceArray array);

    GrainEnvelope get_grain_envelope(GranularSynth* ptr);
    void free_grain_envelope(GrainEnvelope env);
#ifdef __cplusplus
} // extern "C"
#endif

class SynthUI : public QWidget {
    Q_OBJECT
    //Q_DISABLE_COPY(SynthUI)

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
    void onStopAudioClicked();

private:
    GranularSynth* synthPtr = nullptr;

    QLabel *waveformLabel;
    QGraphicsView *waveformView;
    QGraphicsScene *waveformScene;

    QLabel *grainEnvelopeLabel;
    QGraphicsView *grainEnvelopeView;
    QGraphicsScene *grainEnvelopeScene;
    
    QPushButton *loadFileButton;
    QPushButton *playButton;
    QPushButton *stopButton;

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

