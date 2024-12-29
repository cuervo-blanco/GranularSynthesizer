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
#include <QMenuBar>
#include <QMenu>
#include <QDialog>
#include <QSpingBox>
#include <QComboBox>

#ifdef __cplusplus
extern "C" {
#endif
    typedef struct GranularSynth GranularSynth;
    typedef struct AudioEngine AudioEngine;
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

    AudioEngine* create_audio_engine(GranularSynth* ptr);
    int audio_engine_start(AudioEngine* ptr);
    void audio_engine_stop(AudioEngine* ptr);
    void destroy_audio_engine(AudioEngine* ptr);

    void set_sample_rate(AudioEngine* ptr, uint32_t sr);
    void set_file_format(AudioEngine* ptr, const char* fmt);
    void set_bit_depth(AudioEngine* ptr, uint16_t bit_depth);
    void set_bit_rate(AudioEngine* ptr, uint32_t bitrate);
    void set_flac_compression(AudioEngine* ptr, uint8_t level);

    int set_output_device(AudioEngine* ptr, size_t index);
    void get_output_devices(AudioEngine* ptr);
    int set_default_output_device(AudioEngine* ptr);
    char* get_default_output_device(AudioEngine* ptr);

    int record(AudioEngine* ptr, const char* output_path);
    int stop_recording(AudioEngine* ptr);

    void start_scheduler(GranularSynth* ptr);
    void stop_scheduler(GranularSynth* ptr);

    void set_params(
            GranularSynth* ptr, 
            float start, 
            size_t duration, 
            float overlap, 
            float pitch);
    void set_grain_start(GranularSynth* ptr, float start);
    void set_grain_duration(GranularSynth* ptr, size_t duration);
    void set_grain_pitch(GranularSynth* ptr, float pitch);
    void set_overlap(GranularSynth* ptr, float overlap);

    SourceArray get_source_array(GranularSynth* ptr);
    void free_source_array(SourceArray array);

    GrainEnvelope get_grain_envelope(GranularSynth* ptr);
    void free_grain_envelope(GrainEnvelope env);

    int get_sample_rate(GranularSynth* ptr);
    int get_total_channels(GranularSynth* ptr);

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
    void onGrainStartReleased();
    void onGrainStartValueChanged();
    void onGrainDurationReleased();
    void onGrainDurationValueChanged();
    void onGrainPitchReleased();
    void onGrainPitchValueChanged();
    void onOverlapReleased();
    void onOverlapValueChanged();
    void onPlayAudioClicked();
    void onStopAudioClicked();
    void onAudioSettingsClicked();
    void onRecordClicked();
    void onStopRecordingClicked();

private:
    GranularSynth* synthPtr = nullptr;
    AudioEngine* enginePtr = nullptr;

    std::vector<float> m_downsampledWaveform;
    void downsampleWaveform(const std::vector<float>& fullData,
                        std::vector<float>& outDownsampled, size_t targetSize);
    QGraphicsPathItem* m_waveformPathItem = nullptr;
    QGraphicsRectItem* m_grainRectItem = nullptr;

    QLabel *waveformLabel;
    QGraphicsView *waveformView;
    QGraphicsScene *waveformScene;

    QLabel *grainEnvelopeLabel;
    QGraphicsView *grainEnvelopeView;
    QGraphicsScene *grainEnvelopeScene;

    QMenuBar *menuBar;
    QMenu *fileMenu;
    QAction *loadAction;
    QAction *settingsAction;
    
    QPushButton *recordButton;
    QPushButton *stopRecordingButton;
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
    void drawGrainSelectionRect(
        QGraphicsScene* scene,
        double sceneWidth,
        double sceneHeight,
        size_t grainStartSample,
        size_t grainDuration,
        size_t totalSamples
    );
    void drawFullWaveformOnce();
    void updateGrainSelectionRect();
    void resizeEvent(QResizeEvent* event);

    bool isPlaying;
};

#endif // SYNTHUI_H

