#include "cadrs_kernel/src/backend/shell_sectors.rs.h"
#include <BinTools.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepGProp.hxx>
#include <BRepPrimAPI_MakePrism.hxx>
#include <BRepPrimAPI_MakeRevol.hxx>
#include <BRepTools.hxx>
#include <GC_MakeArcOfCircle.hxx>
#include <GProp_GProps.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <cmath>
#include <sstream>
#include <stdexcept>
#include <vector>

namespace cadrs_sectors {
namespace {
rust::Vec<uint8_t> write(const TopoDS_Shape& shape) {
  std::ostringstream stream(std::ios::out | std::ios::binary);
  BinTools::Write(shape, stream);
  rust::Vec<uint8_t> out;
  for (unsigned char c : stream.str()) out.push_back(c);
  return out;
}

// A closed planar face through the points, joined by lines or (where `arc` holds the middle
// point) by an arc.
TopoDS_Face polygon(const std::vector<gp_Pnt>& points, const std::vector<const gp_Pnt*>& arcs) {
  BRepBuilderAPI_MakeWire wire;
  for (size_t i = 0; i < points.size(); ++i) {
    const gp_Pnt& a = points[i];
    const gp_Pnt& b = points[(i + 1) % points.size()];
    if (a.Distance(b) < 1e-9) continue;
    if (arcs[i]) {
      wire.Add(BRepBuilderAPI_MakeEdge(GC_MakeArcOfCircle(a, *arcs[i], b).Value()).Edge());
    } else {
      wire.Add(BRepBuilderAPI_MakeEdge(a, b).Edge());
    }
  }
  if (!wire.IsDone()) throw std::runtime_error("sector profile wire");
  BRepBuilderAPI_MakeFace face(wire.Wire(), true);
  if (!face.IsDone()) throw std::runtime_error("sector profile face");
  return face.Face();
}
}  // namespace

rust::Vec<Piece> convex_sector(rust::Slice<const uint8_t> bytes, double t) {
  rust::Vec<Piece> out;
  TopoDS_Shape shape;
  std::istringstream in(std::string(reinterpret_cast<const char*>(bytes.data()), bytes.size()),
                        std::ios::in | std::ios::binary);
  BinTools::Read(shape, in);
  if (shape.IsNull() || shape.ShapeType() != TopAbs_FACE) return out;
  const TopoDS_Face face = TopoDS::Face(shape);
  BRepAdaptor_Surface s(face, false);
  double u0, u1, v0, v1;
  BRepTools::UVBounds(face, u0, u1, v0, v1);
  gp_Ax3 frame;
  double r = 0.0, big = 0.0, natural = 0.0;
  switch (s.GetType()) {
    case GeomAbs_Cylinder:
      frame = s.Cylinder().Position();
      r = s.Cylinder().Radius();
      natural = r * (u1 - u0) * (v1 - v0);
      break;
    case GeomAbs_Sphere:
      frame = s.Sphere().Position();
      r = s.Sphere().Radius();
      natural = r * r * (u1 - u0) * (std::sin(v1) - std::sin(v0));
      break;
    case GeomAbs_Torus:
      frame = s.Torus().Position();
      big = s.Torus().MajorRadius();
      r = s.Torus().MinorRadius();
      if (r >= big) return out;
      natural = r * (u1 - u0) * (big * (v1 - v0) + r * (std::sin(v1) - std::sin(v0)));
      break;
    default:
      return out;
  }
  const double tol = 1e-7 * std::max(1.0, r);
  if (r > t + tol) return out;
  // The sector is exact only for a face whose boundary is its parameter box.
  GProp_GProps props;
  BRepGProp::SurfaceProperties(face, props);
  if (std::abs(props.Mass() - natural) > 1e-6 * std::max(natural, 1e-9)) return out;
  const gp_Pnt o = frame.Location();
  const gp_Dir x = frame.XDirection(), y = frame.YDirection(), z = frame.Direction();
  auto radial = [&](double u) { return gp_Vec(x) * std::cos(u) + gp_Vec(y) * std::sin(u); };
  // The body lies on the centre side: the outward normal points away from it.
  {
    const double um = 0.5 * (u0 + u1), vm = 0.5 * (v0 + v1);
    gp_Pnt p;
    gp_Vec du, dv;
    s.D1(um, vm, p, du, dv);
    gp_Vec n = du.Crossed(dv);
    if (face.Orientation() == TopAbs_REVERSED) n.Reverse();
    gp_Pnt c = o;
    if (s.GetType() == GeomAbs_Cylinder) c = o.Translated(gp_Vec(z) * gp_Vec(o, p).Dot(gp_Vec(z)));
    if (s.GetType() == GeomAbs_Torus) c = o.Translated(radial(um) * big);
    if (n.Dot(gp_Vec(c, p)) <= 0.0) return out;
  }
  // The profile in the half plane at u0, (ρ, h) ↦ o + ρ·radial(u0) + h·z, turned through the
  // face's span of u.
  const gp_Vec e = radial(u0);
  auto at = [&](double rho, double h) { return o.Translated(e * rho + gp_Vec(z) * h); };
  TopoDS_Face profile;
  if (s.GetType() == GeomAbs_Cylinder) {
    profile = polygon({at(0, v0), at(r, v0), at(r, v1), at(0, v1)}, {nullptr, nullptr, nullptr, nullptr});
  } else {
    const double rho = s.GetType() == GeomAbs_Torus ? big : 0.0;
    const gp_Pnt c = at(rho, 0), a = at(rho + r * std::cos(v0), r * std::sin(v0)),
                 b = at(rho + r * std::cos(v1), r * std::sin(v1));
    const double vm = 0.5 * (v0 + v1);
    const gp_Pnt m = at(rho + r * std::cos(vm), r * std::sin(vm));
    profile = polygon({c, a, b}, {nullptr, &m, nullptr});
  }
  const gp_Dir turn = x.Crossed(y);
  BRepPrimAPI_MakeRevol revol(profile, gp_Ax1(o, turn), u1 - u0);
  if (!revol.IsDone()) throw std::runtime_error("sector revolve");
  const TopoDS_Shape sector = revol.Shape();
  out.push_back(Piece{write(sector)});
  if (t - r > tol) {
    for (TopExp_Explorer ex(sector, TopAbs_FACE); ex.More(); ex.Next()) {
      const TopoDS_Face side = TopoDS::Face(ex.Current());
      BRepAdaptor_Surface ps(side);
      if (ps.GetType() != GeomAbs_Plane) continue;
      gp_Dir n = ps.Plane().Axis().Direction();
      if (side.Orientation() == TopAbs_REVERSED) n.Reverse();
      BRepPrimAPI_MakePrism prism(side, gp_Vec(n) * (t - r));
      if (!prism.IsDone()) throw std::runtime_error("sector side prism");
      out.push_back(Piece{write(prism.Shape())});
    }
  }
  return out;
}
}  // namespace cadrs_sectors
