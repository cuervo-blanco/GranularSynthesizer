#ifndef AUDIOSETTINGSDIALOG_H
#define AUDIOSETTINGSDIALOG_H

#include <QDialog>
#include <QComboBox>
#include <QSpinBox>
#include <QSlider>
#include <QLabel>
#include <QPushButton>
#include <QVBoxLayout>
#include <QHBoxLayout>
#include "SynthUI.h"

extern "C" {
    typedef struct AudioEngine AudioEngine;
    typedef struct GranularSynth GranularSynth;

    typedef struct DeviceList {
        const void* devices;
        size_t count;
    } DeviceList;

    typedef struct DeviceInfo {
        size_t index;
        const char* name;
    } DeviceInfo;

    typedef struct UserSettings {
        unsigned int sample_rate;
        unsigned short bit_depth;
        const char* format;
    } UserSettings;

    DeviceList get_output_devices(AudioEngine* ptr);
    UserSettings get_user_settings(AudioEngine* ptr);
    void free_device_list(DeviceList list);
    int set_output_device(AudioEngine* ptr, size_t index);

    int set_default_output_device(AudioEngine* ptr);

    void set_sample_rate(AudioEngine* ptr, unsigned int sr);
    void set_bit_depth(AudioEngine* ptr, unsigned short bit_depth);
    void set_file_format(AudioEngine* ptr, const char* fmt);
    void set_bit_rate(AudioEngine* ptr, unsigned int bitrate);
    void set_flac_compression(AudioEngine* ptr, unsigned char level);
}

class AudioSettingsDialog : public QDialog {
    Q_OBJECT

public:
    explicit AudioSettingsDialog(
            QWidget *parent = nullptr,
            AudioEngine *engine = nullptr,
            GranularSynth *synth = nullptr,
            QString *path = nullptr);
    ~AudioSettingsDialog();

private slots:
    void applySettings();
    void onOutputDeviceChanged(int index);

private:
    AudioEngine *enginePtr;
    GranularSynth *synthPtr;
    QString *loadedFilePath;

    QComboBox *outputDeviceComboBox;
    QSpinBox *sampleRateSpinBox;
    QSpinBox *bitDepthSpinBox;
    QComboBox *fileFormatComboBox;
    QSpinBox *bitRateSpinBox;
    QSlider *flacCompressionSlider;

    QLabel *flacCompressionLabel;
    QLabel *bitRateLabel;
    QString formatStr;
    // Maybe there should be more labels
};

#endif // AUDIOSETTINGSDIALOG_H

