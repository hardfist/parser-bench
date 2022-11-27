use glob::glob_with;
use glob::MatchOptions;
use mimalloc_rust::GlobalMiMalloc;
use rayon::prelude::*;
use swc::BoolOrDataConfig;
use swc::TransformOutput;
use swc::config::SourceMapsConfig;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;
use swc::Compiler as swcCompiler;
use swc_common::comments::SingleThreadedComments;
use swc_common::Mark;
use swc_common::SourceMap;
use swc_common::{self, sync::Lrc};
use swc_ecma_codegen::{text_writer::JsWriter, Emitter};
use swc_ecma_parser::parse_file_as_module;
use swc_ecma_transforms::helpers::inject_helpers;
use swc_ecma_transforms::helpers::Helpers;
use swc_ecma_transforms::{compat, react as swc_react, resolver};
use swc_ecma_visit::{as_folder, FoldWith, VisitAllWith, VisitWith};
use tracing::event;
use tracing::instrument;
use tracing::{span, Level};
use tracing_chrome::ChromeLayerBuilder;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
// #[global_allocator]
// static GLOBAL: GlobalMiMalloc = GlobalMiMalloc;
pub fn init_tracing() -> Option<tracing_chrome::FlushGuard> {
    let is_enable_chrome_tracing: bool = std::env::var("CHROME_TRACE").ok().map_or(false, |_| true);

    let tracing = tracing_subscriber::registry()
        .with(fmt::layer().pretty().with_file(false))
        .with(EnvFilter::from_env("TRACE"));
    let mut guard = None;
    if is_enable_chrome_tracing {
        let (chrome_layer, _guard) = ChromeLayerBuilder::new().build();
        tracing.with(chrome_layer).init();
        guard = Some(_guard);
    } else {
        tracing.init();
    }
    tracing::trace!("enable tracing");
    guard
}
#[instrument()]
fn test() {
    println!("test instrument");
}
fn main() {
    let mut args = std::env::args().skip(1);
    let pattern: String = args.next().unwrap_or("node_modules/three/src/**/*".into());
    println!("pattern: {}", pattern);
    let guard = init_tracing();
    test();
    let cwd = env::current_dir().unwrap();
    let globs = glob_with(
        &pattern,
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
    let codes: Vec<_> = codes.iter().cycle().take(1).collect();
    println!("len: {}", codes.len());
    
    /*-----------------------minify start ----------------*/
    let cm: Lrc<SourceMap> =Arc::new(Default::default());
    let compiler = swcCompiler::new(cm.clone());
    let start = Instant::now();
    let result:Vec<_> = codes.iter().map(|(path, code)| {
        let fm = cm.clone().load_file(&path).expect("load file failed: {}");
        swc_common::GLOBALS.set(&swc_common::Globals::new(), || {
         let res = swc::try_with_handler(cm.clone(), Default::default(), |handler| {
            compiler.minify(
                fm,
                handler,
                &swc::config::JsMinifyOptions {
                    source_map: BoolOrDataConfig::from_bool(true),
                     emit_source_map_columns: true,
                    ..Default::default()
                },
            )
        });
        let TransformOutput{code, map}  = res.unwrap();
        fs::write("/Users/yangjian/github/parser-bench/a.js", code).unwrap();;
        fs::write("/Users/yangjian/github/parser-bench/a.js.map", map.unwrap()).unwrap();
        return ();
        });

    }).collect();

    let minify_duration = start.elapsed();
    println!("minify duration : {:?}", minify_duration);
    return;
   

}
