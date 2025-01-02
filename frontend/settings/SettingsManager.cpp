#include "SettingsManager.h"
#include <QFile>
#include <QJsonDocument>
#include <QJsonObject>
#include <QStandardPaths>
#include <QDir>

QString SettingsManager::getSettingsFilePath() {
    QString configDir = QStandardPaths::writableLocation(QStandardPaths::AppConfigLocation);
    QDir().mkpath(configDir);
    return configDir + "/settings.json";
}

void SettingsManager::saveSettings(int sampleRate, int bitDepth, int outputDeviceIndex, const QString &fileFormat) {
    QJsonObject settings;
    settings["sample_rate"] = sampleRate;
    settings["bit_depth"] = bitDepth;
    settings["output_device_index"] = outputDeviceIndex;
    settings["file_format"] = fileFormat;

    QString settingsPath = getSettingsFilePath();
    QFile file(settingsPath);
    if (file.open(QIODevice::WriteOnly)) {
        file.write(QJsonDocument(settings).toJson());
    }
}

QJsonObject SettingsManager::loadSettings() {
    QString settingsPath = getSettingsFilePath();
    QFile file(settingsPath);
    if (file.open(QIODevice::ReadOnly)) {
        QByteArray data = file.readAll();
        QJsonDocument doc = QJsonDocument::fromJson(data);
        return doc.object();
    }
    return {};
}

