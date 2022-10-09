use glob::glob_with;
use glob::MatchOptions;
use swc_common::SourceMap;
use rayon::prelude::*;
use std::env;
use std::fs;
use std::sync::Arc;
use std::time::Instant;
use swc_common::{self, sync::Lrc};
use swc_ecma_parser::{parse_file_as_module};
fn main() {
    let cwd = env::current_dir().unwrap();
    let globs = glob_with(
        "node_modules/three/src/**/*",
        MatchOptions {
            ..Default::default()
        },
    )
    .unwrap();
    let codes: Vec<_> = globs.filter_map(|x| {
        let path = x.unwrap();
        let abs_path = cwd.clone().join(path);    
        let metadata = fs::metadata(&abs_path).unwrap();
        if metadata.is_dir(){
            return None;
        }else {
            let code = fs::read_to_string(&abs_path).unwrap();
            return Some((abs_path,code));
        }
        
    }).collect();
    let codes:Vec<_> = codes.iter().cycle().take(10*codes.len()).collect();
    let cm: Lrc<SourceMap> = Arc::new(Default::default());
    let start = Instant::now();
    
    for (path,_code) in codes.iter() {
        let fm = cm.load_file(&path).expect("load file failed: {}");
        let _program = parse_file_as_module(&fm, Default::default(), Default::default(), Default::default(), &mut vec![]).unwrap();
    }
    let duration = start.elapsed();
    println!("serialize duration: {:?}", duration);
    let cm: Lrc<SourceMap> = Default::default();
    let start = Instant::now();
    let _result:Vec<_> = codes.par_iter().map(|(path,_code)| {
        let fm = cm.load_file(&path).expect("load file failed: {}");
        let program = parse_file_as_module(&fm, Default::default(), Default::default(), Default::default(), &mut vec![]).unwrap();
        program
    }).collect();
    let duration = start.elapsed();
    println!("parallel duration: {:?}", duration);
}
