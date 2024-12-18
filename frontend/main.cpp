#include <QApplication>
#include "SynthUI.h"

int main(int argc, char *argv[]) {
    QApplication app(argc, argv);
    SynthUI window;
    window.show();
    return app.exec();
}

