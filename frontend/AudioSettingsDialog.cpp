#include "AudioSettingsDialog.h"
#include "SynthUI.h"

AudioSettingsDialog::AudioSettingsDialog(QWidget *parent, AudioEngine *engine)
    : QDialog(parent), enginePtr(engine) {

    setWindowTitle("Audio Settings");
    QVBoxLayout *mainLayout = new QVBoxLayout(this);

    // Output Device
    QLabel *outputDeviceLabel = new QLabel("Output Device:", this);
    mainLayout->addWidget(outputDeviceLabel);

    outputDeviceComboBox = new QComboBox(this);
    mainLayout->addWidget(outputDeviceComboBox);

    if (enginePtr) {
        DeviceList deviceList = get_output_devices(enginePtr);
        for (size_t i = 0; i < deviceList.count; i++) {
            DeviceInfo device = deviceList.devices[i];
            outputDeviceComboBox->addItem(
                    QString::fromUtf8(device.name), 
                    static_cast<int>(device.index)
                    );
        }
    }

    free_device_list(deviceList);

    connect(outputDeviceComboBox, QOverload<int>::of(&QComboBox::currentIndexChanged),
            this, &AudioSettingsDialog::onOutputDeviceChanged);

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
    fileFormatComboBox->addItems({"WAV", "MP3", "FLAC"});
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
    if (enginePtr) {
        set_sample_rate(enginePtr, sampleRateSpinBox->value());
        set_bit_depth(enginePtr, static_cast<uint16_t>(bitDepthSpinBox->value()));
        
        QString format = fileFormatComboBox->currentText();
        set_file_format(enginePtr, format.toStdString().c_str());

        if (format == "MP3") {
            set_bit_rate(enginePtr, static_cast<uint32_t>(bitRateSpinBox->value()));
        } else if (format == "FLAC") {
            set_flac_compression(enginePtr, static_cast<uint8_t>(flacCompressionSlider->value()));
        }

        int selectedIndex = outputDeviceComboBox->currentIndex();
        if (selectedIndex >= 0) {
            set_output_device(enginePtr, static_cast<size_t>(selectedIndex));
        }
    }
    accept();
}

void AudioSettingsDialog::onOutputDeviceChanged(int index) {
}

