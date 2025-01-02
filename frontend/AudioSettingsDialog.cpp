#include "AudioSettingsDialog.h"
#include "SynthUI.h"
#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QPushButton>
#include <QLabel>
#include <QDebug>

AudioSettingsDialog::AudioSettingsDialog(QWidget *parent, AudioEngine *engine, GranularSynth *synth, QString *path)
    : QDialog(parent), enginePtr(engine), synthPtr(synth), loadedFilePath(path) {

    setWindowTitle("Audio Settings");
    QVBoxLayout *mainLayout = new QVBoxLayout(this);

    // Output Device
    QLabel *outputDeviceLabel = new QLabel("Output Device:", this);
    mainLayout->addWidget(outputDeviceLabel);

    outputDeviceComboBox = new QComboBox(this);
    mainLayout->addWidget(outputDeviceComboBox);

    DeviceList list{};
    list.devices = nullptr;
    list.count   = 0;

    if (enginePtr) {
        DeviceList list = get_output_devices(enginePtr);
        //qDebug() << "deviceList.count =" << deviceList.count;
        //const DeviceInfo* arr = reinterpret_cast<const DeviceInfo*>(deviceList.devices);
        if (list.devices && list.count > 0) {
            for (size_t i = 0; i < list.count; i++) {
                auto di = reinterpret_cast<const DeviceInfo*>(list.devices)[i];
                QString devName = QString::fromUtf8(di.name);
                outputDeviceComboBox->addItem(devName, (qulonglong)di.index);
            }
        }
    }
    connect(outputDeviceComboBox, QOverload<int>::of(&QComboBox::currentIndexChanged),
            this, &AudioSettingsDialog::onOutputDeviceChanged);

    if (list.devices) {
        free_device_list(list);
    }

    // Sample Rate
    QLabel *sampleRateLabel = new QLabel("Sample Rate (Hz):", this);
    mainLayout->addWidget(sampleRateLabel);
    sampleRateSpinBox = new QSpinBox(this);
    sampleRateSpinBox->setRange(22050, 192000);
    sampleRateSpinBox->setValue(44100);
    mainLayout->addWidget(sampleRateSpinBox);

    // Bit Depth
    QLabel *bitDepthLabel = new QLabel("Bit Depth:", this);
    mainLayout->addWidget(bitDepthLabel);
    bitDepthSpinBox = new QSpinBox(this);
    bitDepthSpinBox->setRange(8, 32);
    bitDepthSpinBox->setValue(16);
    mainLayout->addWidget(bitDepthSpinBox);

    // File Format
    QLabel *fileFormatLabel = new QLabel("File Format:", this);
    mainLayout->addWidget(fileFormatLabel);
    fileFormatComboBox = new QComboBox(this);
    fileFormatComboBox->addItems({"wav", "mp3", "flac"});
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
if (!enginePtr || !synthPtr) {
        reject();
        return;
    }

    unsigned int newRate = sampleRateSpinBox->value();
    unsigned short newBitDepth = static_cast<unsigned short>(bitDepthSpinBox->value());
    QString fmt = fileFormatComboBox->currentText();

    int currentIndex = outputDeviceComboBox->currentIndex();
    size_t devIndex = 0; // or default
    if (currentIndex >= 0) {
        QVariant val = outputDeviceComboBox->itemData(currentIndex);
        devIndex = val.value<qulonglong>();
    }

    if (SynthUI* mainUi = qobject_cast<SynthUI*>(parentWidget())) {
        mainUi->reinitializeAudio(newRate, newBitDepth, fmt, devIndex);
    }

    accept();
}

void AudioSettingsDialog::onOutputDeviceChanged(int index) {
    // Could do something real-time here
    // e.g., preview the device info, etc.
    // Or do nothing. We’ll apply on ‘ApplySettings’ anyway.
}

