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