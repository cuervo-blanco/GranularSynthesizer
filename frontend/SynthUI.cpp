#include "SynthUI.h"
#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QFileDialog>
#include <QGraphicsRectItem>

SynthUI::SynthUI(QWidget *parent) : QWidget(parent) {
    QVBoxLayout *mainLayout = new QVBoxLayout(this);

    loadFileButton = new QPushButton("Load Audio File", this);
    connect(loadFileButton, &QPushButton::clicked, this 
}

void SynthUI::onFrequencyChanged(int value) {
    frequency = value;
    label->setText(QString("Frequency: %1 Hz").arg(frequency));
}

void SynthUI::onGenerateClicked() {
    rust_generate_sine_wave(frequency);
}

