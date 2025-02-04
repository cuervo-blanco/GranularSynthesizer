#include <QJsonObject>
#include <QApplication>
#include <QDebug>
#include <QFile>
#include "SynthUI.h"
#include "SettingsManager.h"

int main(int argc, char *argv[]) {
    QApplication app(argc, argv);

    QFile file(":/styles/styles.qss");
    if (file.open(QFile::ReadOnly)) {
        QString styleSheet = QLatin1String(file.readAll());
        app.setStyleSheet(styleSheet);
    }

    QJsonObject settings = SettingsManager::loadSettings();
    unsigned int sampleRate = settings.value("sample_rate").toInt(48000);
    unsigned short bitDepth = settings.value("bit_depth").toInt(16);
    size_t deviceIndex = settings.value("output_device_index").toInt(0);
    QString fileFormat = settings.value("file_format").toString("wav"); 

    SynthUI window;
    //window.initializeAudioEngine(sampleRate, bitDepth, fileFormat, deviceIndex);
    window.show();

    return app.exec();
}

