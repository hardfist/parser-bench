all: bench_esbuild bench_swcpack

esbuild_bundle:
	@hyperfine --warmup 3 './node_modules/esbuild/bin/esbuild --bundle node_modules/three/src/Three.js'
esbuild_transform:
	@hyperfine --warmup 3 './node_modules/esbuild/bin/esbuild  node_modules/three/src/**/*'
spack:
	@hyperfine --warmup 3 './node_modules/@swc/cli/bin/spack node_modules/three/src/Three.js'