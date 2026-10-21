// OCCT's side of tools/kernel_perf.py: runs the cases of a case file (the format of
// crates/cadrs_kernel/examples/kernel_bench.rs, after its `prep`) on OCCT and prints, per case,
//   RESULT <name> <category> ok|err <wall ms> <cpu ms> <runs> [message]
// for the timed (`*`) lines' sum the median wall time and the mean CPU time over the runs (at
// least --reps, until they have taken --min-time, not past --budget; as kernel_bench). CPU time
// is the process's (CLOCK_PROCESS_CPUTIME_ID: all threads).
//
//   occt_bench <cases> [--case NAME] [--reps N] [--budget SECS] [--min-time SECS] [--parallel]
//
// --parallel turns on OCCT's own threads where it has them (Boolean operations'
// SetRunParallel, BRepMesh's parallel mode); without it OCCT runs on one thread, as DRAW does
// by default.
//
// Commands (each as the DRAW command of the same name does it, defaults as DRAW's):
//   restore FILE                 BRepTools::Read (DRAW's .rle header skipped)
//   box dx dy dz | boxat x y z dx dy dz | pcylinder r h | pcone r1 r2 h | psphere r | ptorus R r
//   ttranslate s dx dy dz | trotate s x y z dx dy dz deg | tscale s x y z f   BRepBuilderAPI_Transform
//   explode_e s | explode_f s    TopExp_Explorer order, each sub-shape once
//   nexplode_e s | nexplode_f s  sorted by centre of mass (key x*999 + y*99 + z*0.9)
//   bcut|bfuse|bcommon|bsection a b                                  BRepAlgoAPI_*
//   blend s r e…                 BRepFilletAPI_MakeFillet, DRAW's tolblend defaults
//   lprops s                     length (a number)
//   mkoffset s d                 BRepOffsetAPI_MakeOffset, arc joins
//   mkplane w                    BRepBuilderAPI_MakeFace
//   wire s… | polyline (x y z)…  BRepBuilderAPI_MakeWire, BRepBuilderAPI_MakePolygon
//   prism s dx dy dz | revol s x y z dx dy dz deg                     BRepPrimAPI_MakePrism, _MakeRevol
//   offsetperform s d arc f…     BRepOffset_MakeOffset (tol 1e-7, complete intersection);
//                                with faces MakeThickSolid, else MakeOffsetShape
//   pipe spine profile           BRepOffsetAPI_MakePipe
//   buildsweep spine mode dx dy dz M|C|R solid (section contact correction)…
//                                BRepOffsetAPI_MakePipeShell (mode CF, FR, DT or CN)
//   depouille s dx dy dz (face angle px py pz nx ny nz)…              BRepOffsetAPI_DraftAngle
//   stepread FILE | stepwrite s FILE                                  STEPControl
//   fillet_at s r (x y z)…       BRepFilletAPI_MakeFillet on the edges nearest the points
//   offset s d                   BRepOffsetAPI_MakeOffsetShape::PerformByJoin, arc joins
//   tess s rel angle             BRepMesh_IncrementalMesh, chord rel * bounding box diagonal
//   mass s [eps]                 BRepGProp volume and surface properties: OCCT's fixed Gauss
//                                rules, or adaptive to the relative error eps
#include <BRepAlgoAPI_Common.hxx>
#include <BRepAlgoAPI_Cut.hxx>
#include <BRepAlgoAPI_Fuse.hxx>
#include <BRepAlgoAPI_Section.hxx>
#include <BRepBndLib.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepBuilderAPI_Transform.hxx>
#include <BRepExtrema_DistShapeShape.hxx>
#include <BRepFilletAPI_MakeFillet.hxx>
#include <BRepGProp.hxx>
#include <BRepMesh_IncrementalMesh.hxx>
#include <BRepOffset_MakeOffset.hxx>
#include <BRepOffsetAPI_DraftAngle.hxx>
#include <BRepOffsetAPI_MakeOffset.hxx>
#include <BRepOffsetAPI_MakeOffsetShape.hxx>
#include <BRepOffsetAPI_MakePipe.hxx>
#include <BRepOffsetAPI_MakePipeShell.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCone.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakePrism.hxx>
#include <BRepPrimAPI_MakeRevol.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRepPrimAPI_MakeTorus.hxx>
#include <BRepTools.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <Bnd_Box.hxx>
#include <GProp_GProps.hxx>
#include <IFSelect_ReturnStatus.hxx>
#include <Interface_Static.hxx>
#include <Message.hxx>
#include <Message_Gravity.hxx>
#include <Message_Messenger.hxx>
#include <Message_Printer.hxx>
#include <STEPControl_Reader.hxx>
#include <STEPControl_Writer.hxx>
#include <Standard_Failure.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopTools_MapOfShape.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Iterator.hxx>
#include <gp_Ax1.hxx>
#include <gp_Pln.hxx>
#include <gp_Trsf.hxx>

#include <algorithm>
#include <chrono>
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <ctime>
#include <fstream>
#include <map>
#include <sstream>
#include <stdexcept>
#include <string>
#include <tuple>
#include <vector>

namespace {

bool g_parallel = false;

struct Line {
  bool timed = false;
  std::string out, cmd;
  std::vector<std::string> args;
};

struct Case {
  std::string name, category;
  std::vector<Line> lines;
};

struct Val {
  enum Kind { Shape, List, Num } kind = Shape;
  TopoDS_Shape s;
  std::vector<TopoDS_Shape> list;
  double num = 0;
};

std::vector<std::string> split(const std::string& l) {
  std::istringstream is(l);
  std::vector<std::string> t;
  std::string w;
  while (is >> w) t.push_back(w);
  return t;
}

std::vector<Case> parse_cases(const std::string& path) {
  std::ifstream f(path);
  if (!f) throw std::runtime_error("can't read " + path);
  std::vector<Case> cases;
  Case cur;
  bool in = false;
  std::string raw;
  while (std::getline(f, raw)) {
    auto t = split(raw);
    if (t.empty() || t[0][0] == '#') continue;
    if (t[0] == "case") {
      cur = Case{t.at(1), t.at(2), {}};
      in = true;
      continue;
    }
    if (t[0] == "end") {
      if (in) cases.push_back(cur);
      in = false;
      continue;
    }
    Line l;
    if (t[0][0] == '*') {
      l.timed = true;
      if (t[0] == "*") t.erase(t.begin());
      else t[0] = t[0].substr(1);
    }
    size_t i = 0;
    if (t.size() >= 3 && t[1] == "=") {
      l.out = t[0];
      i = 2;
    }
    l.cmd = t.at(i);
    l.args.assign(t.begin() + i + 1, t.end());
    cur.lines.push_back(l);
  }
  return cases;
}

double cpu_ms() {
  timespec ts;
  clock_gettime(CLOCK_PROCESS_CPUTIME_ID, &ts);
  return ts.tv_sec * 1e3 + ts.tv_nsec * 1e-6;
}

double wall_ms() {
  return std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now().time_since_epoch()).count();
}

struct Ctx {
  std::map<std::string, Val> vars;
  bool timing = false;
  double wall = 0, cpu = 0;
  double t_wall = 0, t_cpu = 0;

  void start() {
    if (!timing) return;
    t_cpu = cpu_ms();
    t_wall = wall_ms();
  }
  void stop() {
    if (!timing) return;
    wall += wall_ms() - t_wall;
    cpu += cpu_ms() - t_cpu;
  }

  TopoDS_Shape shape(const std::string& tok) {
    auto dot = tok.rfind('.');
    if (dot != std::string::npos) {
      std::string idx = tok.substr(dot + 1);
      if (!idx.empty() && std::all_of(idx.begin(), idx.end(), ::isdigit)) {
        auto it = vars.find(tok.substr(0, dot));
        if (it == vars.end() || it->second.kind != Val::List) throw std::runtime_error(tok + ": not a list");
        size_t i = std::stoul(idx);
        if (i >= it->second.list.size()) throw std::runtime_error(tok + ": out of range");
        return it->second.list[i];
      }
    }
    auto it = vars.find(tok);
    if (it == vars.end() || it->second.kind != Val::Shape) throw std::runtime_error(tok + " isn't a shape");
    return it->second.s;
  }

  double num(const std::string& tok) {
    auto star = tok.find('*');
    if (star == std::string::npos) return std::stod(tok);
    auto it = vars.find(tok.substr(0, star));
    if (it == vars.end() || it->second.kind != Val::Num) throw std::runtime_error(tok + ": not a number");
    return it->second.num * std::stod(tok.substr(star + 1));
  }
};

// A compound of one shape is that shape (as STEP files come in).
TopoDS_Shape unwrap(TopoDS_Shape s) {
  while (s.ShapeType() == TopAbs_COMPOUND) {
    TopoDS_Iterator it(s);
    if (!it.More()) break;
    TopoDS_Shape first = it.Value();
    it.Next();
    if (it.More()) break;
    s = first;
  }
  return s;
}

std::vector<TopoDS_Shape> explode(const TopoDS_Shape& s, TopAbs_ShapeEnum typ) {
  std::vector<TopoDS_Shape> out;
  TopTools_MapOfShape seen;
  for (TopExp_Explorer ex(s, typ); ex.More(); ex.Next())
    if (seen.Add(ex.Current())) out.push_back(ex.Current());
  return out;
}

std::vector<TopoDS_Shape> nexplode(const TopoDS_Shape& s, TopAbs_ShapeEnum typ) {
  auto v = explode(s, typ);
  std::vector<std::pair<double, size_t>> key;
  for (size_t i = 0; i < v.size(); ++i) {
    GProp_GProps g;
    BRepGProp::LinearProperties(v[i], g);
    gp_Pnt c = g.CentreOfMass();
    key.push_back({c.X() * 999 + c.Y() * 99 + c.Z() * 0.9, i});
  }
  std::stable_sort(key.begin(), key.end(), [](auto& a, auto& b) { return a.first < b.first; });
  std::vector<TopoDS_Shape> out;
  for (auto& k : key) out.push_back(v[k.second]);
  return out;
}

TopoDS_Shape transform(const TopoDS_Shape& s, const gp_Trsf& t) {
  BRepBuilderAPI_Transform trf(t);
  trf.Perform(s, Standard_False);
  if (!trf.IsDone()) throw std::runtime_error("transform failed");
  return trf.Shape();
}

TopoDS_Wire as_wire(const TopoDS_Shape& s) {
  if (s.ShapeType() == TopAbs_WIRE) return TopoDS::Wire(s);
  if (s.ShapeType() == TopAbs_EDGE) return BRepBuilderAPI_MakeWire(TopoDS::Edge(s)).Wire();
  auto ws = explode(s, TopAbs_WIRE);
  if (ws.size() == 1) return TopoDS::Wire(ws[0]);
  throw std::runtime_error("not a wire");
}

double diagonal(const TopoDS_Shape& s) {
  Bnd_Box b;
  BRepBndLib::AddOptimal(s, b, Standard_False, Standard_False);
  if (b.IsVoid()) return 1.0;
  return std::sqrt(b.SquareExtent());
}

template <class Op>
TopoDS_Shape bop(Op& op, const TopoDS_Shape& a, const TopoDS_Shape& b) {
  TopTools_ListOfShape args, tools;
  args.Append(a);
  tools.Append(b);
  op.SetArguments(args);
  op.SetTools(tools);
  op.SetRunParallel(g_parallel);
  op.Build();
  if (!op.IsDone() || op.HasErrors()) throw std::runtime_error("boolean failed");
  return op.Shape();
}

// Runs one line; returns whether it made a value (in `v`).
bool exec(Ctx& c, const Line& l, Val& v) {
  c.timing = l.timed;
  const auto& a = l.args;
  auto arg = [&](size_t i) -> const std::string& {
    if (i >= a.size()) throw std::runtime_error(l.cmd + ": an argument is missing");
    return a[i];
  };
  auto n = [&](size_t i) { return c.num(arg(i)); };
  auto ret = [&](const TopoDS_Shape& s) {
    if (s.IsNull()) throw std::runtime_error(l.cmd + ": a null result");
    v.kind = Val::Shape;
    v.s = s;
    return true;
  };
  const std::string& cmd = l.cmd;
  if (cmd == "restore") {
    std::ifstream f(arg(0));
    if (!f) throw std::runtime_error("can't read " + arg(0));
    std::stringstream ss;
    ss << f.rdbuf();
    std::string text = ss.str();
    if (text.rfind("DBRep_DrawableShape", 0) == 0) text = text.substr(text.find('\n') + 1);
    std::istringstream is(text);
    TopoDS_Shape s;
    BRep_Builder bb;
    c.start();
    BRepTools::Read(s, is, bb);
    c.stop();
    return ret(s);
  }
  if (cmd == "box" || cmd == "boxat" || cmd == "pcylinder" || cmd == "pcone" || cmd == "psphere" || cmd == "ptorus") {
    TopoDS_Shape s;
    c.start();
    if (cmd == "box") s = BRepPrimAPI_MakeBox(n(0), n(1), n(2)).Shape();
    else if (cmd == "boxat") s = BRepPrimAPI_MakeBox(gp_Pnt(n(0), n(1), n(2)), n(3), n(4), n(5)).Shape();
    else if (cmd == "pcylinder") s = BRepPrimAPI_MakeCylinder(n(0), n(1)).Shape();
    else if (cmd == "pcone") s = BRepPrimAPI_MakeCone(n(0), n(1), n(2)).Shape();
    else if (cmd == "psphere") s = BRepPrimAPI_MakeSphere(n(0)).Shape();
    else s = BRepPrimAPI_MakeTorus(n(0), n(1)).Shape();
    c.stop();
    return ret(s);
  }
  if (cmd == "ttranslate" || cmd == "trotate" || cmd == "tscale") {
    TopoDS_Shape s = c.shape(arg(0));
    gp_Trsf t;
    if (cmd == "ttranslate") t.SetTranslation(gp_Vec(n(1), n(2), n(3)));
    else if (cmd == "trotate") t.SetRotation(gp_Ax1(gp_Pnt(n(1), n(2), n(3)), gp_Dir(n(4), n(5), n(6))), n(7) * M_PI / 180.0);
    else t.SetScale(gp_Pnt(n(1), n(2), n(3)), n(4));
    c.start();
    TopoDS_Shape r = transform(s, t);
    c.stop();
    return ret(r);
  }
  if (cmd == "explode_e" || cmd == "explode_f" || cmd == "nexplode_e" || cmd == "nexplode_f") {
    TopoDS_Shape s = c.shape(arg(0));
    TopAbs_ShapeEnum typ = cmd.back() == 'e' ? TopAbs_EDGE : TopAbs_FACE;
    c.start();
    v.list = cmd[0] == 'n' ? nexplode(s, typ) : explode(s, typ);
    c.stop();
    v.kind = Val::List;
    return true;
  }
  if (cmd == "bcut" || cmd == "bfuse" || cmd == "bcommon" || cmd == "bsection") {
    TopoDS_Shape x = c.shape(arg(0)), y = c.shape(arg(1)), r;
    c.start();
    if (cmd == "bcut") { BRepAlgoAPI_Cut op; r = bop(op, x, y); }
    else if (cmd == "bfuse") { BRepAlgoAPI_Fuse op; r = bop(op, x, y); }
    else if (cmd == "bcommon") { BRepAlgoAPI_Common op; r = bop(op, x, y); }
    else { BRepAlgoAPI_Section op; r = bop(op, x, y); }
    c.stop();
    return ret(r);
  }
  if (cmd == "blend") {
    TopoDS_Shape s = c.shape(arg(0));
    double r = n(1);
    std::vector<TopoDS_Edge> es;
    for (size_t i = 2; i < a.size(); ++i) es.push_back(TopoDS::Edge(c.shape(a[i])));
    c.start();
    BRepFilletAPI_MakeFillet mk(s, ChFi3d_Rational);
    mk.SetParams(1e-2, 1e-4, 1e-5, 1e-4, 1e-5, 1e-3);
    mk.SetContinuity(GeomAbs_C1, 1e-2);
    for (auto& e : es) mk.Add(r, e);
    mk.Build();
    c.stop();
    if (!mk.IsDone()) throw std::runtime_error("blend failed");
    return ret(mk.Shape());
  }
  if (cmd == "wire") {
    std::vector<TopoDS_Shape> parts;
    for (auto& t : a) parts.push_back(c.shape(t));
    c.start();
    BRepBuilderAPI_MakeWire mk;
    for (auto& p : parts) {
      if (p.ShapeType() == TopAbs_EDGE) mk.Add(TopoDS::Edge(p));
      else if (p.ShapeType() == TopAbs_WIRE) mk.Add(TopoDS::Wire(p));
      else
        for (auto& e : explode(p, TopAbs_EDGE)) mk.Add(TopoDS::Edge(e));
    }
    c.stop();
    if (!mk.IsDone()) throw std::runtime_error("wire failed");
    return ret(mk.Wire());
  }
  if (cmd == "polyline") {
    c.start();
    BRepBuilderAPI_MakePolygon mk;
    for (size_t i = 0; i + 2 < a.size(); i += 3) mk.Add(gp_Pnt(n(i), n(i + 1), n(i + 2)));
    c.stop();
    if (!mk.IsDone()) throw std::runtime_error("polyline failed");
    return ret(mk.Wire());
  }
  if (cmd == "prism") {
    TopoDS_Shape s = c.shape(arg(0));
    c.start();
    BRepPrimAPI_MakePrism mk(s, gp_Vec(n(1), n(2), n(3)));
    c.stop();
    return ret(mk.Shape());
  }
  if (cmd == "revol") {
    TopoDS_Shape s = c.shape(arg(0));
    c.start();
    BRepPrimAPI_MakeRevol mk(s, gp_Ax1(gp_Pnt(n(1), n(2), n(3)), gp_Dir(n(4), n(5), n(6))), n(7) * M_PI / 180.0);
    c.stop();
    return ret(mk.Shape());
  }
  if (cmd == "lprops") {
    GProp_GProps g;
    BRepGProp::LinearProperties(c.shape(arg(0)), g);
    v.kind = Val::Num;
    v.num = g.Mass();
    return true;
  }
  if (cmd == "mkoffset") {
    TopoDS_Shape s = c.shape(arg(0));
    double d = n(1);
    c.start();
    BRepOffsetAPI_MakeOffset par;
    if (s.ShapeType() == TopAbs_FACE) {
      s.Orientation(TopAbs_FORWARD);
      par.Init(TopoDS::Face(s), GeomAbs_Arc);
    } else {
      par.Init(GeomAbs_Arc);
      for (TopExp_Explorer ex(s, TopAbs_WIRE); ex.More(); ex.Next()) par.AddWire(TopoDS::Wire(ex.Current()));
    }
    par.Perform(d, 0.0);
    c.stop();
    if (!par.IsDone()) throw std::runtime_error("mkoffset failed");
    return ret(par.Shape());
  }
  if (cmd == "mkplane") {
    TopoDS_Wire w = as_wire(c.shape(arg(0)));
    c.start();
    BRepBuilderAPI_MakeFace mk(w, Standard_False);
    c.stop();
    if (!mk.IsDone()) throw std::runtime_error("mkplane failed");
    return ret(mk.Face());
  }
  if (cmd == "offsetperform") {
    TopoDS_Shape s = c.shape(arg(0));
    double d = n(1);
    bool arc = arg(2) == "1";
    std::vector<TopoDS_Face> fs;
    for (size_t i = 3; i < a.size(); ++i) fs.push_back(TopoDS::Face(c.shape(a[i])));
    c.start();
    BRepOffset_MakeOffset mo;
    mo.Initialize(s, d, 1e-7, BRepOffset_Skin, Standard_True, Standard_False, arc ? GeomAbs_Arc : GeomAbs_Intersection, Standard_False, Standard_False);
    for (auto& f : fs) mo.AddFace(f);
    if (fs.empty()) mo.MakeOffsetShape();
    else mo.MakeThickSolid();
    c.stop();
    if (!mo.IsDone()) throw std::runtime_error("offsetperform failed");
    return ret(mo.Shape());
  }
  if (cmd == "pipe") {
    TopoDS_Wire sp = as_wire(c.shape(arg(0)));
    TopoDS_Shape pr = c.shape(arg(1));
    c.start();
    BRepOffsetAPI_MakePipe mk(sp, pr);
    mk.Build();
    c.stop();
    if (!mk.IsDone()) throw std::runtime_error("pipe failed");
    return ret(mk.Shape());
  }
  if (cmd == "buildsweep") {
    TopoDS_Wire sp = as_wire(c.shape(arg(0)));
    const std::string& mode = arg(1);
    gp_Dir dir(0, 0, 1);
    if (mode == "CN") dir = gp_Dir(n(2), n(3), n(4));
    char tr = arg(5)[0];
    bool solid = arg(6) == "1";
    std::vector<std::tuple<TopoDS_Shape, bool, bool>> secs;
    for (size_t i = 7; i + 2 < a.size(); i += 3) {
      secs.emplace_back(c.shape(a[i]), a[i + 1] == "1", a[i + 2] == "1");
    }
    c.start();
    BRepOffsetAPI_MakePipeShell mk(sp);
    if (mode == "FR") mk.SetMode(Standard_True);
    else if (mode == "DT") mk.SetDiscreteMode();
    else if (mode == "CN") mk.SetMode(dir);
    else if (mode == "CF") mk.SetMode(Standard_False);
    else throw std::runtime_error("buildsweep mode " + mode);
    for (auto& [s, t, r] : secs) mk.Add(s, t, r);
    mk.SetTransitionMode(tr == 'C' ? BRepBuilderAPI_RightCorner : tr == 'R' ? BRepBuilderAPI_RoundCorner : BRepBuilderAPI_Transformed);
    mk.Build();
    if (mk.IsDone() && solid) mk.MakeSolid();
    c.stop();
    if (!mk.IsDone()) throw std::runtime_error("buildsweep failed");
    return ret(mk.Shape());
  }
  if (cmd == "depouille") {
    TopoDS_Shape s = c.shape(arg(0));
    gp_Dir dir(n(1), n(2), n(3));
    struct G { TopoDS_Face f; double ang; gp_Pln pl; };
    std::vector<G> gs;
    for (size_t i = 4; i + 7 < a.size(); i += 8)
      gs.push_back({TopoDS::Face(c.shape(a[i])), c.num(a[i + 1]) * M_PI / 180.0,
                    gp_Pln(gp_Pnt(c.num(a[i + 2]), c.num(a[i + 3]), c.num(a[i + 4])), gp_Dir(c.num(a[i + 5]), c.num(a[i + 6]), c.num(a[i + 7])))});
    c.start();
    BRepOffsetAPI_DraftAngle dr(s);
    for (auto& g : gs) {
      dr.Add(g.f, dir, g.ang, g.pl);
      if (!dr.AddDone()) break;
    }
    if (dr.AddDone()) dr.Build();
    c.stop();
    if (!dr.AddDone() || !dr.IsDone()) throw std::runtime_error("depouille failed");
    return ret(dr.Shape());
  }
  if (cmd == "stepread") {
    c.start();
    STEPControl_Reader rd;
    if (rd.ReadFile(arg(0).c_str()) != IFSelect_RetDone) throw std::runtime_error("can't read " + arg(0));
    rd.TransferRoots();
    TopoDS_Shape s = rd.OneShape();
    c.stop();
    return ret(unwrap(s));
  }
  if (cmd == "stepwrite") {
    TopoDS_Shape s = c.shape(arg(0));
    c.start();
    STEPControl_Writer wr;
    if (wr.Transfer(s, STEPControl_AsIs) != IFSelect_RetDone) throw std::runtime_error("STEP transfer failed");
    if (wr.Write(arg(1).c_str()) != IFSelect_RetDone) throw std::runtime_error("can't write " + arg(1));
    c.stop();
    return false;
  }
  if (cmd == "fillet_at") {
    TopoDS_Shape s = c.shape(arg(0));
    double r = n(1);
    auto edges = explode(s, TopAbs_EDGE);
    TopTools_MapOfShape picked;
    std::vector<TopoDS_Edge> es;
    for (size_t i = 2; i + 2 < a.size(); i += 3) {
      TopoDS_Vertex p = BRepBuilderAPI_MakeVertex(gp_Pnt(n(i), n(i + 1), n(i + 2)));
      double best = 1e300;
      TopoDS_Shape be;
      for (auto& e : edges) {
        if (BRep_Tool::Degenerated(TopoDS::Edge(e))) continue;
        BRepExtrema_DistShapeShape d(p, e);
        if (d.IsDone() && d.Value() < best) { best = d.Value(); be = e; }
      }
      if (!be.IsNull() && picked.Add(be)) es.push_back(TopoDS::Edge(be));
    }
    c.start();
    BRepFilletAPI_MakeFillet mk(s);
    for (auto& e : es) mk.Add(r, e);
    mk.Build();
    c.stop();
    if (!mk.IsDone()) throw std::runtime_error("fillet failed");
    return ret(mk.Shape());
  }
  if (cmd == "offset") {
    TopoDS_Shape s = c.shape(arg(0));
    double d = n(1);
    c.start();
    BRepOffsetAPI_MakeOffsetShape mk;
    mk.PerformByJoin(s, d, 1e-7, BRepOffset_Skin, Standard_False, Standard_False, GeomAbs_Arc);
    c.stop();
    if (!mk.IsDone()) throw std::runtime_error("offset failed");
    return ret(mk.Shape());
  }
  if (cmd == "tess") {
    TopoDS_Shape s = c.shape(arg(0));
    double chord = n(1) * diagonal(s), ang = n(2);
    BRepTools::Clean(s);
    c.start();
    BRepMesh_IncrementalMesh mesh(s, chord, Standard_False, ang, g_parallel);
    c.stop();
    if (!mesh.IsDone()) throw std::runtime_error("mesh failed");
    return false;
  }
  if (cmd == "mass") {
    TopoDS_Shape s = c.shape(arg(0));
    c.start();
    GProp_GProps gv, gs;
    if (a.size() > 1) {
      // Adaptive, to a relative error (as precise as the native kernel's Gauss rules).
      double eps = n(1);
      BRepGProp::VolumeProperties(s, gv, eps);
      BRepGProp::SurfaceProperties(s, gs, eps);
    } else {
      BRepGProp::VolumeProperties(s, gv);
      BRepGProp::SurfaceProperties(s, gs);
    }
    c.stop();
    return false;
  }
  throw std::runtime_error("unknown command " + cmd);
}

void run_once(const Case& cs, double& wall, double& cpu) {
  Ctx c;
  for (auto& l : cs.lines) {
    Val v;
    if (exec(c, l, v) && !l.out.empty()) c.vars[l.out] = v;
  }
  wall = c.wall;
  cpu = c.cpu;
}

double mean(const std::vector<double>& v) {
  double s = 0;
  for (double x : v) s += x;
  return s / v.size();
}

double median(std::vector<double> v) {
  std::sort(v.begin(), v.end());
  size_t n = v.size();
  return n % 2 ? v[n / 2] : 0.5 * (v[n / 2 - 1] + v[n / 2]);
}

}  // namespace

int main(int argc, char** argv) {
  if (argc < 2) {
    std::fprintf(stderr, "occt_bench <cases> [--case NAME] [--reps N] [--budget SECS] [--min-time SECS] [--parallel]\n");
    return 2;
  }
  std::string only;
  int reps = 5;
  double budget = 20.0, min_time = 1.0;
  for (int i = 2; i < argc; ++i) {
    std::string f = argv[i];
    if (f == "--case" && i + 1 < argc) only = argv[++i];
    else if (f == "--reps" && i + 1 < argc) reps = std::atoi(argv[++i]);
    else if (f == "--budget" && i + 1 < argc) budget = std::atof(argv[++i]);
    else if (f == "--min-time" && i + 1 < argc) min_time = std::atof(argv[++i]);
    else if (f == "--parallel") g_parallel = true;
  }
  // Quiet: OCCT's messages (STEP translation reports) go to stdout otherwise.
  Message::DefaultMessenger()->RemovePrinters(STANDARD_TYPE(Message_Printer));
  std::vector<Case> cases;
  try {
    cases = parse_cases(argv[1]);
  } catch (const std::exception& e) {
    std::fprintf(stderr, "%s\n", e.what());
    return 2;
  }
  for (auto& cs : cases) {
    if (!only.empty() && cs.name != only) continue;
    std::vector<double> walls, cpus;
    std::string err;
    double t0 = wall_ms();
    for (int r = 0; r < 200; ++r) {
      double t = (wall_ms() - t0) * 1e-3;
      if ((r >= std::max(reps, 1) && t >= min_time) || (r >= 1 && t > budget)) break;
      try {
        double w, c;
        run_once(cs, w, c);
        walls.push_back(w);
        cpus.push_back(c);
      } catch (const Standard_Failure& e) {
        err = std::string("OCCT: ") + e.GetMessageString();
      } catch (const std::exception& e) {
        err = e.what();
      }
      if (!err.empty()) break;
    }
    for (auto& ch : err)
      if (ch == '\t' || ch == '\n') ch = ' ';
    if (!err.empty())
      std::printf("RESULT\t%s\t%s\terr\t0\t0\t0\t%s\n", cs.name.c_str(), cs.category.c_str(), err.c_str());
    else
      std::printf("RESULT\t%s\t%s\tok\t%.3f\t%.3f\t%zu\n", cs.name.c_str(), cs.category.c_str(), median(walls), mean(cpus), walls.size());
    std::fflush(stdout);
  }
  // Skips the static destructors (which crash in this static OCCT build).
  std::fflush(stdout);
  std::_Exit(0);
}
