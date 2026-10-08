#include <QCoreApplication>
#include <QFileInfo>
#include <QDir>
#include <QThread>
// Replacement helper solely for the cleanup regression. Worker zero fails
// while its already-started siblings wait, so the host must kill/join them.
int main(int argc,char** argv){QCoreApplication app(argc,argv);if(app.arguments().size()!=3)return 2;
    if(QFileInfo(app.arguments()[1]).dir().dirName()=="0"){QThread::msleep(1000);return 23;}
    QThread::sleep(60);return 0;
}
