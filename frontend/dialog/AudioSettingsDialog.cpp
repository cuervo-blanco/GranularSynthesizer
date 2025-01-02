#include "AudioSettingsDialog.h"
#include "SynthUI.h"
#include "SettingsManager.h"
#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QPushButton>
#include <QLabel>
#include <QDebug>
#include <QProcess>
#include <QApplication>

AudioSettingsDialog::AudioSettingsDialog(QWidget *parent, AudioEngine *engine, GranularSynth *synth, QString *path)
    : QDialog(parent), enginePtr(engine), synthPtr(synth), loadedFilePath(path) {

    setWindowTitle("Audio Settings");
    QVBoxLayout *mainLayout = new QVBoxLayout(this);

    if (!enginePtr) {
    qDebug() << "Engine pointer is null!";
    return;
}

    // Output Device
    QLabel *outputDeviceLabel = new QLabel("Output Device:", this);
    mainLayout->addWidget(outputDeviceLabel);

    outputDeviceComboBox = new QComboBox(this);
    mainLayout->addWidget(outputDeviceComboBox);

    UserSettings settings = get_user_settings(enginePtr);
    QString formatStr;
    if (settings.format) {
        formatStr = QString::fromUtf8(settings.format);
    } else {
        qDebug() << "Format string is null!";
        formatStr = "wav"; // Default format
    }

    DeviceList list{};
    list.devices = nullptr;
    list.count   = 0;

    if (enginePtr) {
        DeviceList list = get_output_devices(enginePtr);
        if (list.devices && list.count > 0) {
            for (size_t i = 0; i < list.count; i++) {
                auto di = reinterpret_cast<const DeviceInfo*>(list.devices)[i];
                QString devName = QString::fromUtf8(di.name);
                outputDeviceComboBox->addItem(devName, (qulonglong)di.index);
            }
            free_device_list(list);
        } else {
            qDebug() << "No devices found!";
        }
    }
    connect(outputDeviceComboBox, QOverload<int>::of(&QComboBox::currentIndexChanged),
            this, &AudioSettingsDialog::onOutputDeviceChanged);

    // Get default settings function

    // Sample Rate
    QLabel *sampleRateLabel = new QLabel("Sample Rate (Hz):", this);
    mainLayout->addWidget(sampleRateLabel);
    sampleRateSpinBox = new QSpinBox(this);
    sampleRateSpinBox->setRange(22050, 192000);
    sampleRateSpinBox->setValue(settings.sample_rate);
    mainLayout->addWidget(sampleRateSpinBox);

    // Bit Depth
    QLabel *bitDepthLabel = new QLabel("Bit Depth:", this);
    mainLayout->addWidget(bitDepthLabel);
    bitDepthSpinBox = new QSpinBox(this);
    bitDepthSpinBox->setRange(8, 32);
    bitDepthSpinBox->setValue(settings.bit_depth);
    mainLayout->addWidget(bitDepthSpinBox);

    // File Format
    QLabel *fileFormatLabel = new QLabel("File Format:", this);
    mainLayout->addWidget(fileFormatLabel);
    fileFormatComboBox = new QComboBox(this);
    fileFormatComboBox->addItems({"wav", "mp3", "flac"});
    int index = fileFormatComboBox->findText(formatStr);
    if (index != -1) {
        fileFormatComboBox->setCurrentIndex(index);
    } else {
        qDebug() << "Unknown format: " << formatStr;
    }
    mainLayout->addWidget(fileFormatComboBox);

    // Bit Rate (For MP3)
    bitRateLabel = new QLabel("Bit Rate (kbps):", this);
    mainLayout->addWidget(bitRateLabel);
    bitRateSpinBox = new QSpinBox(this);
    bitRateSpinBox->setRange(96, 320);
    bitRateSpinBox->setValue(128);
    mainLayout->addWidget(bitRateSpinBox);

    // FLAC Compression Level
    flacCompressionLabel = new QLabel("FLAC Compression Level:", this);
    mainLayout->addWidget(flacCompressionLabel);
    flacCompressionSlider = new QSlider(Qt::Horizontal, this);
    flacCompressionSlider->setRange(0, 8);
    flacCompressionSlider->setValue(5);
    mainLayout->addWidget(flacCompressionSlider);

    // Apply and Cancel Buttons
    QHBoxLayout *buttonLayout = new QHBoxLayout();

    QPushButton *applyButton = new QPushButton("Apply", this);
    buttonLayout->addWidget(applyButton);
    connect(applyButton, &QPushButton::clicked, this, &AudioSettingsDialog::applySettings);

    QPushButton *cancelButton = new QPushButton("Cancel", this);
    buttonLayout->addWidget(cancelButton);
    connect(cancelButton, &QPushButton::clicked, this, &QDialog::reject);

    mainLayout->addLayout(buttonLayout);
    setLayout(mainLayout);
}

AudioSettingsDialog::~AudioSettingsDialog() {}

void AudioSettingsDialog::applySettings() {

    unsigned int newSampleRate = sampleRateSpinBox->value();
    unsigned short newBitDepth = static_cast<unsigned short>(bitDepthSpinBox->value());
    QString format = fileFormatComboBox->currentText();
    int selectedDeviceIndex = outputDeviceComboBox->currentData().toInt();
    size_t devIndex = 0; // or default
    if (selectedDeviceIndex >= 0) {
        QVariant val = outputDeviceComboBox->itemData(selectedDeviceIndex);
        devIndex = val.value<qulonglong>();
    }

    QMessageBox::StandardButton reply = QMessageBox::warning(
        this,
        "Apply Settings",
        "Applying these settings will restart the application. Any unsaved changes will be lost.\n\nDo you wish to proceed?",
        QMessageBox::Yes | QMessageBox::No
    );
    if (reply == QMessageBox::No) {
        return;
    }
    if (reply == QMessageBox::Yes) {
        SettingsManager::saveSettings(newSampleRate, newBitDepth, selectedDeviceIndex, format);
        QProcess::startDetached(QApplication::applicationFilePath(), QStringList());
        QApplication::quit();
    }

}

void AudioSettingsDialog::onOutputDeviceChanged(int index) {
    // Could do something real-time here
    // e.g., preview the device info, etc.
    // Or do nothing. We’ll apply on ‘ApplySettings’ anyway.
}

