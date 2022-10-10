use glob::glob_with;
use glob::MatchOptions;
use rayon::prelude::*;
use std::env;
use std::fs;
use std::sync::Arc;
use std::time::Instant;
use swc_common::SourceMap;
use swc_common::{self, sync::Lrc};
use swc_ecma_codegen::{text_writer::JsWriter, Emitter};
use swc_ecma_parser::parse_file_as_module;
use swc_ecma_transforms::helpers::inject_helpers;
use swc_ecma_transforms::helpers::Helpers;
use swc_ecma_visit::{as_folder, FoldWith, VisitAllWith, VisitWith};
fn main() {
    let cwd = env::current_dir().unwrap();
    let globs = glob_with(
        "node_modules/three/src/**/*",
        MatchOptions {
            ..Default::default()
        },
    )
    .unwrap();
    let codes: Vec<_> = globs
        .filter_map(|x| {
            let path = x.unwrap();
            let abs_path = cwd.clone().join(path);
            let metadata = fs::metadata(&abs_path).unwrap();
            if metadata.is_dir() {
                return None;
            } else {
                let code = fs::read_to_string(&abs_path).unwrap();
                return Some((abs_path, code));
            }
        })
        .collect();
    let codes: Vec<_> = codes.iter().cycle().take(10 * codes.len()).collect();
    println!("len: {}", codes.len());
    let start = Instant::now();
    let cm: Lrc<SourceMap> = Arc::new(Default::default());
    let modules: Vec<_> = codes
        .par_iter()
        .map(|(path, _code)| {
            let fm = cm.clone().load_file(&path).expect("load file failed: {}");
            let m = parse_file_as_module(
                &fm,
                Default::default(),
                Default::default(),
                Default::default(),
                &mut vec![],
            )
            .unwrap();
            m
        })
        .collect();
    let parse_duration = start.elapsed();
    println!("parse duration : {:?}", parse_duration);
    let transform_start = Instant::now();
    let modules: Vec<_> = modules
        .into_par_iter()
        .map(|ast| {
            swc_common::GLOBALS.set(&swc_common::Globals::default(), || {
                swc_ecma_transforms::helpers::HELPERS
                    .set(&Helpers::new(true), || ast.fold_with(&mut inject_helpers()))
            })
        })
        .collect();
    let transform_duration = transform_start.elapsed();
    println!("transform duration: {:?}", transform_duration);
    let codegen_start = Instant::now();
    let codes: Vec<_> = modules
        .into_par_iter()
        .map(|m| {
            let code = {
                let mut buf = vec![];
                {
                    let mut emitter = Emitter {
                        cfg: swc_ecma_codegen::Config {
                            ..Default::default()
                        },
                        cm: cm.clone(),
                        comments: None,
                        wr: JsWriter::new(cm.clone(), "\n", &mut buf, None),
                    };
                    emitter.emit_module(&m).unwrap();
                }
                String::from_utf8_lossy(&buf).to_string()
            };
            code
        })
        .collect();
    let codegen_duration = codegen_start.elapsed();
    println!("codegen duration: {:?}", codegen_duration);
    let duration = start.elapsed();
    println!("duration: {:?}", duration);
}
