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

class AudioSettingsDialog : public QDialog {
    Q_OBJECT

public:
    explicit AudioSettingsDialog(QWidget *parent = nullptr, AudioEngine *engine = nullptr);
    ~AudioSettingsDialog();

private slots:
    void applySettings();
    void onOutputDeviceChanged(int index);

private:
    AudioEngine *enginePtr;

    QComboBox *outputDeviceComboBox;
    QSpinBox *sampleRateSpinBox;
    QSpinBox *bitDepthSpinBox;
    QComboBox *fileFormatComboBox;
    QSpinBox *bitRateSpinBox; // For MP3
    QSlider *flacCompressionSlider; // For FLAC

    QLabel *flacCompressionLabel;
    QLabel *bitRateLabel;
};

#endif // AUDIOSETTINGSDIALOG_H

