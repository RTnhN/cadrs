//! The kernel in a process of its own (`rebuild::process`): rebuilds, exports, tasks and the files the app has come back the same as on the kernel
//! thread, and a feature that crashes the kernel process fails alone (the rest of the list
//! rebuilds, and the app goes on).
//!
//! One test, with a `main` of its own (`harness = false`): the worker is the process's, and it
//! starts with the first job; and this executable is the kernel process it starts
//! (`--kernel-process`, with `--crash-on <feature name>` before it to crash on that feature).

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(at) = args
        .iter()
        .position(|a| a == cadrs_core::rebuild::process::ARG)
    {
        if let Some(i) = args[..at].iter().position(|a| a == "--crash-on")
            && let Some(name) = args.get(i + 1)
        {
            // SAFETY: nothing else runs yet (no threads).
            unsafe { std::env::set_var("CADRS_KERNEL_CRASH_ON", name) };
        }
        std::process::exit(cadrs_core::rebuild::process::main(&args[at + 1..]));
    }
    #[cfg(feature = "brep")]
    {
        tests::the_kernel_process_builds_and_survives_a_crash();
        println!("test the_kernel_process_builds_and_survives_a_crash ... ok");
    }
}

#[cfg(feature = "brep")]
mod tests {
    use cadrs_core::commands::{AddExtrude, AddFeature, AddSketch, EditSketch, SetExtrude};
    use cadrs_core::document::{BooleanOp, Document, ExtrudeFeature, FeatureKind};
    use cadrs_core::rebuild::{self, process};
    use cadrs_core::{ElementId, Feature, FeatureId, History, samples};
    use cadrs_sketch::{PlaneRef, SketchOp, Vec2};

    #[track_caller]
    fn close(a: f64, b: f64, tol: f64) {
        assert!((a - b).abs() <= tol, "got {a}, expected {b} ± {tol}");
    }

    struct Doc {
        d: Document,
        h: History,
        el: ElementId,
    }

    impl Doc {
        fn new() -> Self {
            let d = Document::new("Kernel process");
            let el = d.elements[0].id;
            Self {
                d,
                h: History::default(),
                el,
            }
        }

        fn features(&self) -> Vec<Feature> {
            self.d.element(self.el).unwrap().features().to_vec()
        }

        fn name(&self, f: FeatureId) -> String {
            self.d
                .element(self.el)
                .unwrap()
                .feature(f)
                .unwrap()
                .name
                .clone()
        }

        /// A box x0..x1 × y0..y1 on Top, `h` high, a new part.
        fn block(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, h: f64) -> FeatureId {
            let s = FeatureId::new();
            self.h
                .execute(
                    &mut self.d,
                    &AddSketch {
                        element: self.el,
                        feature: s,
                        plane: Some(PlaneRef::Top),
                    },
                )
                .unwrap();
            let v = Vec2::new;
            let op = SketchOp::AddPolyline {
                points: vec![v(x0, y0), v(x1, y0), v(x1, y1), v(x0, y1)],
                closed: true,
                construction: false,
                label: "Add rectangle",
            };
            self.h
                .execute(
                    &mut self.d,
                    &EditSketch {
                        element: self.el,
                        feature: s,
                        op,
                    },
                )
                .unwrap();
            let g = self
                .d
                .element(self.el)
                .unwrap()
                .feature(s)
                .unwrap()
                .sketch()
                .unwrap()
                .geometry
                .clone();
            let regions =
                samples::region_refs(s, &g, &[Vec2::new((x0 + x1) / 2.0, (y0 + y1) / 2.0)]);
            let mut e = ExtrudeFeature {
                op: BooleanOp::New,
                ..samples::extrude_of(regions, h)
            };
            e.depth_expr = format!("{h} mm");
            let f = FeatureId::new();
            self.h
                .execute(
                    &mut self.d,
                    &AddExtrude {
                        element: self.el,
                        feature: f,
                        extrude: ExtrudeFeature::default(),
                    },
                )
                .unwrap();
            self.h
                .execute(
                    &mut self.d,
                    &SetExtrude {
                        element: self.el,
                        feature: f,
                        extrude: e,
                        label: "Extrude".into(),
                    },
                )
                .unwrap();
            f
        }
    }

    fn volume(b: &rebuild::Build) -> f64 {
        b.parts.iter().map(|p| p.mass.unwrap().volume).sum()
    }

    pub fn the_kernel_process_builds_and_survives_a_crash() {
        // Three blocks; the kernel process crashes computing the second.
        let mut d = Doc::new();
        d.block(0.0, 0.0, 10.0, 10.0, 10.0);
        let crashing = d.block(20.0, 0.0, 30.0, 10.0, 10.0);
        d.block(40.0, 0.0, 60.0, 10.0, 10.0);
        process::use_process(
            std::env::current_exe().unwrap(),
            vec!["--crash-on".into(), d.name(crashing).into()],
        );
        assert!(process::enabled());

        // The crashing block fails with why; the others build (1000 + 2000 mm³).
        let b = rebuild::build(&d.features());
        assert_eq!(b.errors, vec![(crashing, process::CRASHED.to_string())]);
        assert_eq!(b.parts.len(), 2);
        close(volume(&b), 3000.0, 1e-6);

        // The next rebuild doesn't crash again (a new kernel process knows the poisoned feature):
        // the same result, from its cache.
        let again = rebuild::build(&d.features());
        assert_eq!(again.errors, b.errors);
        close(volume(&again), 3000.0, 1e-6);

        // A STEP export (a job of its own) of the first part.
        let first = b
            .parts
            .iter()
            .find(|p| p.mass.unwrap().volume < 1500.0)
            .unwrap();
        let request = cadrs_core::export::StepRequest {
            parts: vec![(first.id, "Cube".into())],
            y_up: false,
            individual: false,
        };
        let files = rebuild::export_step(d.features(), request)
            .wait()
            .expect("the export");
        assert_eq!(files.len(), 1);
        let step = files[0].bytes.clone();
        assert!(String::from_utf8_lossy(&step).contains("ISO-10303-21"));

        // A task: the import plan of that file (one part, the cube).
        let plan =
            rebuild::exchange::plan_import(cadrs_core::import::ImportFormat::Step, step.clone())
                .wait()
                .expect("a result")
                .expect("a plan");
        assert_eq!(plan.parts.len(), 1);

        // An Import of it: its bytes are a blob of this process, sent to the kernel process.
        let mut imported = Doc::new();
        let x = cadrs_core::import::ImportFeature::from_file("cube.step", step).unwrap();
        let f = FeatureId::new();
        imported
            .h
            .execute(
                &mut imported.d,
                &AddFeature {
                    element: imported.el,
                    feature: f,
                    base_name: "Import".into(),
                    kind: FeatureKind::Import(x),
                },
            )
            .unwrap();
        let b = rebuild::build(&imported.features());
        assert!(b.errors.is_empty(), "{:?}", b.errors);
        close(volume(&b), 1000.0, 1e-6);
    }
}
