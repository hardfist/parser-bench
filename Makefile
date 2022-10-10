all: bench_esbuild bench_swcpack

esbuild_bundle:
	@hyperfine --warmup 3 './node_modules/esbuild/bin/esbuild --bundle node_modules/three/src/Three.js'
esbuild_transform:
	@hyperfine --warmup 3 './node_modules/esbuild/bin/esbuild  node_modules/three/src/**/*'
spack:
	@hyperfine --warmup 3 './node_modules/@swc/cli/bin/spack node_modules/three/src/Three.js'
swc:
	@cargo build --release
	@RAYON_NUM_THREADS=1 hyperfine --warmup 3 './target/release/swc_single'
	@RAYON_NUM_THREADS=1 hyperfine --warmup 3 './target/release/swc_parallel'
	@RAYON_NUM_THREADS=2 hyperfine --warmup 3 './target/release/swc_single'
	@RAYON_NUM_THREADS=2 hyperfine --warmup 3 './target/release/swc_parallel'
	@RAYON_NUM_THREADS=4 hyperfine --warmup 3 './target/release/swc_single'
	@RAYON_NUM_THREADS=4 hyperfine --warmup 3 './target/release/swc_parallel'
	@RAYON_NUM_THREADS=8 hyperfine --warmup 3 './target/release/swc_single'
	@RAYON_NUM_THREADS=8 hyperfine --warmup 3 './target/release/swc_parallel'

swc_1:
	@RAYON_NUM_THREADS=1 samply record ./target/release/swc
swc_8:
	@RAYON_NUM_THREADS=8 samply record ./target/release/swc
swc_trace_1:
	@RAYON_NUM_THREADS=1 TRACE=TRACE CHROME_TRACE=1  ./target/release/swc
swc_trace_2:
	@RAYON_NUM_THREADS=2 TRACE=TRACE CHROME_TRACE=1  ./target/release/swc
swc_trace_4:
	@RAYON_NUM_THREADS=4 TRACE=TRACE CHROME_TRACE=1  ./target/release/swc
swc_trace_8:
	@RAYON_NUM_THREADS=8 TRACE=TRACE CHROME_TRACE=1  ./target/release/swc
