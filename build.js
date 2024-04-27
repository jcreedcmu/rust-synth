import * as esbuild from 'esbuild';

const args = process.argv.slice(2);

const watch = args[0] == 'watch';

async function go () {
  const ctx = await esbuild.context({
	 entryPoints: ['./gui/main.ts'],
	 minify: false,
	 sourcemap: true,
	 bundle: true,
	 outfile: './public/bundle.js',
	 format: 'cjs',
	 logLevel: 'info',
  })

  if (watch) {
	 await ctx.watch();
  }
  else {
	 const result = await ctx.rebuild()
	 ctx.dispose();
  }
}

go();