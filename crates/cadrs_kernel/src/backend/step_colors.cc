#include "cadrs_kernel/src/backend/step_colors.rs.h"
#include <BinTools.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <ShapeFix_Shape.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <STEPCAFControl_Reader.hxx>
#include <STEPCAFControl_Writer.hxx>
#include <TDataStd_Name.hxx>
#include <TCollection_AsciiString.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopoDS.hxx>
#include <XCAFApp_Application.hxx>
#include <XCAFDoc_DocumentTool.hxx>
#include <XCAFDoc_ShapeTool.hxx>
#include <XCAFDoc_ColorTool.hxx>
#include <Quantity_ColorRGBA.hxx>
#include <cmath>
#include <fstream>
#include <sstream>
#include <stdexcept>
#include <vector>

namespace cadrs_colors {
namespace {
struct Document {
  Handle(TDocStd_Document) doc;
  Document() {
    XCAFApp_Application::GetApplication()->NewDocument("MDTV-XCAF", doc);
    XCAFDoc_DocumentTool::SetLengthUnit(doc, 0.001);
  }
  ~Document() { XCAFApp_Application::GetApplication()->Close(doc); }
};
std::string name(const TDF_Label& label) {
  Handle(TDataStd_Name) attr;
  if (!label.FindAttribute(TDataStd_Name::GetID(), attr)) return {};
  const auto& text = attr->Get();
  std::string result(text.LengthOfCString() + 1, '\0');
  char* ptr = &result[0];
  const auto length = text.ToUTF8CString(ptr);
  result.resize(length);
  return result;
}
Color color(const Handle(XCAFDoc_ColorTool)& tool, const TDF_Label& label) {
  Quantity_ColorRGBA value;
  if (!tool->GetColor(label, XCAFDoc_ColorSurf, value) && !tool->GetColor(label, XCAFDoc_ColorGen, value)) return Color{{0,0,0,0}, false};
  double red, green, blue;
  value.GetRGB().Values(red, green, blue, Quantity_TOC_sRGB);
  auto byte = [](double x) { return static_cast<uint8_t>(std::lround(std::max(0.0, std::min(1.0, x)) * 255.0)); };
  return Color{{byte(red), byte(green), byte(blue), byte(value.Alpha())}, true};
}
Color surface_color(const Handle(XCAFDoc_ColorTool)& tool, const TopoDS_Shape& shape, Color fallback) {
  Quantity_ColorRGBA value;
  if (!tool->GetColor(shape, XCAFDoc_ColorSurf, value) && !tool->GetColor(shape, XCAFDoc_ColorGen, value)) return fallback;
  double red, green, blue;
  value.GetRGB().Values(red, green, blue, Quantity_TOC_sRGB);
  auto byte = [](double x) { return static_cast<uint8_t>(std::lround(std::max(0.0, std::min(1.0, x)) * 255.0)); };
  return Color{{byte(red), byte(green), byte(blue), byte(value.Alpha())}, true};
}
void walk(const TDF_Label& label, const gp_Trsf& at, const std::string& occurrence_name, Color override_color,
          const Handle(XCAFDoc_ShapeTool)& shapes, const Handle(XCAFDoc_ColorTool)& colors,
          Model& out, std::vector<TDF_Label>& seen, int depth) {
  if (depth > 64) throw std::runtime_error("STEP assembly is nested too deeply");
  if (shapes->IsAssembly(label)) {
    TDF_LabelSequence components;
    shapes->GetComponents(label, components, false);
    Color parent = color(colors, label);
    if (!override_color.present) override_color = parent;
    for (int i = 1; i <= components.Length(); ++i) {
      const auto component = components.Value(i);
      TDF_Label reference;
      if (!shapes->GetReferredShape(component, reference)) continue;
      Color local = color(colors, component);
      if (!local.present) local = override_color;
      walk(reference, at * shapes->GetLocation(component).Transformation(), name(component), local, shapes, colors, out, seen, depth + 1);
    }
    return;
  }
  size_t index = 0;
  for (; index < seen.size(); ++index) if (seen[index].IsEqual(label)) break;
  if (index == seen.size()) {
    auto shape = shapes->GetShape(label);
    if (shape.IsNull()) return;
    Part part;
    part.name = name(label);
    TopTools_IndexedMapOfShape faces;
    TopExp::MapShapes(shape, TopAbs_FACE, faces);
    Color base = surface_color(colors, shape, color(colors, label));
    std::vector<Color> inherited(faces.Extent(), base);
    // Apply solid and shell defaults once, then the more specific face styles.
    for (auto kind : {TopAbs_SOLID, TopAbs_SHELL}) {
      for (TopExp_Explorer e(shape, kind); e.More(); e.Next()) {
        Color style = surface_color(colors, e.Current(), Color{{0,0,0,0},false});
        if (!style.present) continue;
        for (TopExp_Explorer f(e.Current(), TopAbs_FACE); f.More(); f.Next()) {
          const int index = faces.FindIndex(f.Current());
          if (index > 0) inherited[index - 1] = style;
        }
      }
    }
    for (int i = 1; i <= faces.Extent(); ++i) {
      part.faces.push_back(surface_color(colors, faces(i), inherited[i - 1]));
    }
    // Keep the existing import healing behavior, remapping styles through the repair history.
    if (!BRepCheck_Analyzer(shape).IsValid()) {
      Handle(ShapeFix_Shape) fix = new ShapeFix_Shape(shape);
      fix->Perform();
      auto repaired = fix->Shape();
      if (!repaired.IsNull() && BRepCheck_Analyzer(repaired).IsValid()) {
        TopTools_IndexedMapOfShape repaired_faces;
        TopExp::MapShapes(repaired, TopAbs_FACE, repaired_faces);
        rust::Vec<Color> repaired_colors;
        for (int i = 1; i <= repaired_faces.Extent(); ++i) repaired_colors.push_back(base);
        for (int i = 1; i <= faces.Extent(); ++i) {
          auto mapped = fix->Context()->Apply(faces(i));
          for (TopExp_Explorer f(mapped, TopAbs_FACE); f.More(); f.Next()) {
            int index = repaired_faces.FindIndex(f.Current());
            if (index > 0) repaired_colors[index - 1] = part.faces[i - 1];
          }
        }
        shape = repaired;
        part.faces = std::move(repaired_colors);
      }
    }
    // Binary serialization preserves topology and the face traversal order used for styles.
    std::ostringstream stream(std::ios::out | std::ios::binary);
    BinTools::Write(shape, stream);
    for (unsigned char c : stream.str()) part.brep.push_back(c);
    out.parts.push_back(std::move(part));
    seen.push_back(label);
  }
  Occurrence occurrence;
  occurrence.part = index;
  occurrence.name = occurrence_name;
  occurrence.color = override_color;
  for (int r = 0; r < 3; ++r) for (int c = 0; c < 4; ++c) occurrence.matrix[r*4+c] = at.Value(r+1,c+1);
  out.occurrences.push_back(std::move(occurrence));
}
}
Model read_step_colors(rust::Str path) {
  try {
    Document doc;
    STEPCAFControl_Reader reader;
    reader.SetColorMode(true);
    reader.SetNameMode(true);
    reader.SetLayerMode(false);
    reader.SetPropsMode(false);
    reader.SetGDTMode(false);
    reader.SetMatMode(false);
    reader.SetViewMode(false);
    std::string file(path);
    if (reader.ReadFile(file.c_str()) != IFSelect_RetDone || !reader.Transfer(doc.doc)) throw std::runtime_error("STEP file could not be read");
    auto shapes = XCAFDoc_DocumentTool::ShapeTool(doc.doc->Main());
    auto colors = XCAFDoc_DocumentTool::ColorTool(doc.doc->Main());
    TDF_LabelSequence roots;
    shapes->GetFreeShapes(roots);
    Model result;
    std::vector<TDF_Label> seen;
    for (int i = 1; i <= roots.Length(); ++i) {
      if (result.name.empty()) result.name = name(roots.Value(i));
      walk(roots.Value(i), gp_Trsf(), name(roots.Value(i)), Color{{0,0,0,0},false}, shapes, colors, result, seen, 0);
    }
    return result;
  } catch (const Standard_Failure& error) { throw std::runtime_error(error.GetMessageString()); }
}

// A real STEP fixture: a red part with one green face, used twice with a blue instance override.
rust::Vec<uint8_t> step_color_fixture() {
  try {
    Document doc;
    auto shapes = XCAFDoc_DocumentTool::ShapeTool(doc.doc->Main());
    auto colors = XCAFDoc_DocumentTool::ColorTool(doc.doc->Main());
    auto box = BRepPrimAPI_MakeBox(10.0,20.0,30.0).Shape();
    auto part = shapes->AddShape(box, false);
    TDataStd_Name::Set(part, "Colored box");
    colors->SetColor(part, Quantity_Color(1,0,0,Quantity_TOC_sRGB), XCAFDoc_ColorSurf);
    TopExp_Explorer face(box,TopAbs_FACE);
    auto face_label = shapes->AddSubShape(part, face.Current());
    colors->SetColor(face_label, Quantity_Color(0,1,0,Quantity_TOC_sRGB), XCAFDoc_ColorSurf);
    auto assembly = shapes->NewShape();
    TDataStd_Name::Set(assembly,"Colored assembly");
    shapes->AddComponent(assembly,part,TopLoc_Location());
    gp_Trsf move; move.SetTranslation(gp_Vec(100,0,0));
    auto second = shapes->AddComponent(assembly,part,TopLoc_Location(move));
    colors->SetColor(second,Quantity_Color(0,0,1,Quantity_TOC_sRGB),XCAFDoc_ColorSurf);
    shapes->UpdateAssemblies();
    STEPCAFControl_Writer writer;
    writer.SetColorMode(true);
    writer.SetNameMode(true);
    if (!writer.Transfer(doc.doc,STEPControl_AsIs)) throw std::runtime_error("STEP fixture transfer failed");
    std::ostringstream stream;
    if (writer.ChangeWriter().WriteStream(stream) != IFSelect_RetDone) throw std::runtime_error("STEP fixture write failed");
    rust::Vec<uint8_t> bytes;
    for (unsigned char c : stream.str()) bytes.push_back(c);
    return bytes;
  } catch (const Standard_Failure& error) { throw std::runtime_error(error.GetMessageString()); }
}
}
