// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer Qt - Main entry point

#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQuickStyle>

int main(int argc, char *argv[])
{
    QGuiApplication app(argc, argv);
    app.setOrganizationName("Gosh");
    app.setApplicationName("Gosh Transfer");
    app.setApplicationVersion("2.0.3");

    // Use Fusion style for consistent cross-platform look
    QQuickStyle::setStyle("Fusion");

    QQmlApplicationEngine engine;

    // Load main QML
    const QUrl url(QStringLiteral("qrc:/qml/main.qml"));
    QObject::connect(&engine, &QQmlApplicationEngine::objectCreated,
                     &app, [url](QObject *obj, const QUrl &objUrl) {
        if (!obj && url == objUrl)
            QCoreApplication::exit(-1);
    }, Qt::QueuedConnection);

    engine.load(url);

    return app.exec();
}
