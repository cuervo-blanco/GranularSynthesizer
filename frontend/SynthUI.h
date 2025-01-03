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
#include <QDial>
#include <QSpinBox>
#include <QComboBox>
#include <vector>
#include "AudioSettingsDialog.h"

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

    GranularSynth* create_synth(unsigned int sample_rate);
    void destroy_synth(GranularSynth* ptr);

    int load_audio_from_file(GranularSynth* ptr, const char* file_path, unsigned int master_sample_rate);
    void generate_grain_envelope(GranularSynth* ptr, size_t size);

    AudioEngine* create_audio_engine(GranularSynth* ptr, unsigned int sample_rate, unsigned short channels,
            unsigned short bit_depth, const char* format, int index);
    int audio_engine_start(AudioEngine* ptr);
    void audio_engine_stop(AudioEngine* ptr);
    void destroy_audio_engine(AudioEngine* ptr);

    int get_master_sample_rate(AudioEngine* ptr);
    void set_sample_rate(AudioEngine* ptr, unsigned int sr);
    void set_file_format(AudioEngine* ptr, const char* fmt);
    void set_bit_depth(AudioEngine* ptr, unsigned short bit_depth);
    void set_bit_rate(AudioEngine* ptr, unsigned int bitrate);
    void set_flac_compression(AudioEngine* ptr, unsigned char level);

    //DeviceList get_output_devices(AudioEngine* ptr);
    //void free_device_list(DeviceList list);

    int set_output_device(AudioEngine* ptr, size_t index);
    //int set_default_output_device(AudioEngine* ptr);
    //char* get_default_output_device(AudioEngine* ptr);

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
    int get_default_output_device_index();

#ifdef __cplusplus
} // extern "C"
#endif

class QGraphicsPathItem;
class QGraphicsRectItem;

class SynthUI : public QWidget {
    Q_OBJECT
    //Q_DISABLE_COPY(SynthUI)

public:
    explicit SynthUI(QWidget *parent = nullptr);
    ~SynthUI();
    void initializeAudioEngine(const QJsonObject &settings);

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
                            std::vector<float>& outDownsampled,
                            size_t targetSize);

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
    QDial *grainDurationDial;
    QDial *grainPitchDial;
    QDial *overlapDial;

    QLabel *grainStartLabel;
    QLabel *grainDurationLabel;
    QLabel *grainPitchLabel;
    QLabel *overlapLabel;

    QString loadedFilePath;
    unsigned int globalSampleRate;
    int globalDeviceIndex;
    

    void updateWaveformDisplay();
    void updateEnvelopeDisplay();
    void drawGrainSelectionRect(QGraphicsScene* scene,
                                double sceneWidth,
                                double sceneHeight,
                                size_t grainStartSample,
                                size_t grainDuration,
                                size_t totalSamples);
    void drawFullWaveformOnce();
    void updateGrainSelectionRect();
    void resizeEvent(QResizeEvent* event) override;
    void paintEvent(QPaintEvent *event) override;
    bool isPlaying = false;
};

#endif // SYNTHUI_H

