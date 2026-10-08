// OM9-FILE-012: application-owned correction for the verified SDK Read defect.
// This text replaces only Read in a generated copy; upstream source stays intact.
#include <memory>
bool
ON_CurveOnSurface::Read(ON_BinaryArchive& file)
{
  const auto deferred_reference = [](const ON_Curve* curve) {
    if (!curve || curve->Dimension() != 0)
      return false;
    const auto id = curve->ClassId();
    return id == ON_ClassId::ClassId("ON_PolyEdgeCurve")
        || id == ON_ClassId::ClassId("ON_PolyEdgeSegment");
  };
  ON_Object* raw = nullptr;
  const int parameter_status = file.ReadObject(&raw);
  std::unique_ptr<ON_Object> parameter(raw);
  auto curve2 = ON_Curve::Cast(parameter.get());
  if (parameter_status != 1 || !curve2
      || (!deferred_reference(curve2) && (!curve2->IsValid() || curve2->Dimension() != 2)))
    return false;

  int has_approximation = 0;
  if (!file.ReadInt(&has_approximation) || (has_approximation != 0 && has_approximation != 1))
    return false;
  std::unique_ptr<ON_Object> approximation;
  ON_Curve* curve3 = nullptr;
  if (has_approximation)
  {
    raw = nullptr;
    const int status = file.ReadObject(&raw);
    approximation.reset(raw);
    curve3 = ON_Curve::Cast(approximation.get());
    if (status != 1 || !curve3 || (!deferred_reference(curve3) && !curve3->IsValid()))
      return false;
  }

  raw = nullptr;
  const int surface_status = file.ReadObject(&raw);
  std::unique_ptr<ON_Object> surface(raw);
  auto native_surface = ON_Surface::Cast(surface.get());
  if (surface_status != 1 || !native_surface || !native_surface->IsValid()
      || (curve3 && !deferred_reference(curve3) && curve3->Dimension() != native_surface->Dimension()))
    return false;

  // Known serialized PolyEdge children remain unresolved until their owner
  // model/reference adapter links them; Read must not discard their payload.
  // No old pointer changes until all child records are typed and owned.
  DestroyCurveTree();
  delete m_c2;
  delete m_c3;
  delete m_s;
  m_c2 = static_cast<ON_Curve*>(parameter.release());
  m_c3 = static_cast<ON_Curve*>(approximation.release());
  m_s = static_cast<ON_Surface*>(surface.release());
  return true;
}
