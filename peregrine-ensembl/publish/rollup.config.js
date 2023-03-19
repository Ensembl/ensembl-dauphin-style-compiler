import copy from 'rollup-plugin-copy';

export default [
  // CommonJS
  {
    input: './tmp/peregrine_ensembl.js',
    output: {
      dir: './dist',
      entryFileNames: 'bundle-cjs.cjs',
      format: 'cjs',
      exports: 'named'
    },
    plugins: [
      copy({
        targets: [
          { src: 'tmp/*', dest: 'dist/' }
        ],
        verbose: true
      }),
      copy({
        targets: [
          { src: 'tmp/peregrine_ensembl.d.ts', dest: 'dist/', rename: "index.d.ts" }
        ],
        verbose: true
      })
    ]
  }
];