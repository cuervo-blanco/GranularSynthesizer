#ifndef SETTINGSMANAGER_H
#define SETTINGSMANAGER_H

#include <QString>
#include <QJsonObject>

class SettingsManager {
public:
    static void saveSettings(int sampleRate, 
            int bitDepth, int outputDeviceIndex, const QString &fileFormat);
    static QJsonObject loadSettings();
private:
    static QString getSettingsFilePath();

};

#endif // SETTINGSMANAGER_H
