// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include "SolidReferences.h"
#include <QJsonObject>
namespace OpenMatrix9Gui {
// Independent capture-time values; no Python/native shape owner retained.
struct SolidSourceEvidence {
    ObjectSnapshot source;
    QJsonObject snapshot;
    QByteArray contentDigest;
    std::vector<SolidPoint> points;
    std::string role,frame;
    std::size_t index=0,begin=0,end=0;
};
struct SolidEvidence {
    std::vector<ObjectSnapshot> sources;
    QJsonObject context;
    std::string units;
};
std::vector<SolidSourceEvidence> captureSolidFitSelection(App::Document&,std::size_t used);
void verifySolidFitSources(App::Document&,const std::vector<SolidSourceEvidence>&);
void validateSolidFitBudget(const std::vector<SolidSourceEvidence>& existing,const std::vector<SolidSourceEvidence>& incoming);
SolidEvidence captureSolidEvidence(App::Document&,const std::map<std::size_t,SolidCurveReference>&,
    const std::optional<SolidCurveReference>&,const std::vector<SolidSourceEvidence>&,
    const std::string& construction,const std::string& units);
// Existing solver returns a fresh private Part shape in document-world space.
// Serialize before publication; bytes retain no host owner and no display mesh.
std::string solidGeometryResult(App::Document&,PyObject*,const SolidEvidence&,const std::string& outputName);
}
