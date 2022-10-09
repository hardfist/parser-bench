all: bench_esbuild bench_swcpack

bench_esbuild:
	@hyperfine --warmup 3 './node_modules/esbuild/bin/esbuild --bundle node_modules/three/src/Three.js'
bench_spack:
	@hyperfine --warmup 3 './node_modules/@swc/cli/bin/spack node_modules/three/src/Three.js'