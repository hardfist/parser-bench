use glob::glob_with;
use glob::MatchOptions;
use rayon::prelude::*;
use swc_common::Mark;
use swc_common::comments::SingleThreadedComments;
use tracing::event;
use tracing::instrument;
use tracing_chrome::ChromeLayerBuilder;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
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
use tracing::{span, Level};
use swc_ecma_transforms::{react as swc_react, resolver, compat};
use swc_ecma_visit::{as_folder, FoldWith, VisitAllWith, VisitWith};
pub fn init_tracing() -> Option<tracing_chrome::FlushGuard>{
  let is_enable_chrome_tracing:bool = std::env::var("CHROME_TRACE").ok().map_or(false, |_| true);
  
  let tracing = tracing_subscriber::registry().with(fmt::layer().pretty().with_file(false))
  .with(EnvFilter::from_env("TRACE"));
  let mut guard = None;
  if is_enable_chrome_tracing {
    let (chrome_layer,_guard) = ChromeLayerBuilder::new().build();
    tracing.with(chrome_layer).init();
    guard = Some(_guard);
  }else {
    tracing.init();
  }
  tracing::trace!("enable tracing");
  guard
}
#[instrument()]
fn test(){
    println!("test instrument");
}
fn main() {
    let guard = init_tracing();
    test();
    let cwd = env::current_dir().unwrap();
    let globs = glob_with(
        "node_modules/typescript/lib/typescript.js",
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
    let codes: Vec<_> = codes.iter().cycle().take(8 * codes.len()).collect();
    println!("len: {}", codes.len());
    let start = Instant::now();
    let cm: Lrc<SourceMap> = Arc::new(Default::default());
    let parse_enter = tracing::span!(Level::TRACE,"total_parse").entered();
    let modules: Vec<_> = codes
        .par_iter()
        .map(|(path, _code)| {
            let _guard = tracing::span!(Level::TRACE,"parse").entered();
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
    parse_enter.exit();
    let parse_duration = start.elapsed();
    println!("parse duration : {:?}", parse_duration);
    let transform_start = Instant::now();
    let transform_enter = span!(Level::TRACE, "total_transform").entered();
    let modules: Vec<_> = modules
        .into_par_iter()
        .map(|ast| {
            let _guard = span!(Level::TRACE, "transform").entered();
            swc_common::GLOBALS.set(&swc_common::Globals::default(), || {
                swc_ecma_transforms::helpers::HELPERS
                    .set(&Helpers::new(true), || {
                         let ast = ast.fold_with(&mut resolver(Mark::new(), Mark::new(), false));
                         let ast = ast.fold_with(&mut swc_react::react::<SingleThreadedComments>(cm.clone(), None, swc_react::Options { ..Default::default()},Mark::new()));
                         let ast = ast.fold_with(&mut inject_helpers());
                         ast

                    }
                   )
            })
        })
        .collect();
    transform_enter.exit();
    let transform_duration = transform_start.elapsed();
    println!("transform duration: {:?}", transform_duration);
    let codegen_start = Instant::now();
    let codegen_enter = span!(Level::TRACE, "total_codegen").entered();
    let codes: Vec<_> = modules
        .into_par_iter()
        .map(|m| {
            let _guard = span!(Level::TRACE, "codegen").entered();
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
    codegen_enter.exit();
    
    let codegen_duration = codegen_start.elapsed();
    println!("codegen duration: {:?}", codegen_duration);
    event!(Level::TRACE, "codegen_ended");
    let duration = start.elapsed();
    println!("duration: {:?}", duration);
    if let Some(guard)= guard {
        guard.flush();
    }
    tracing::info!("end");
}
