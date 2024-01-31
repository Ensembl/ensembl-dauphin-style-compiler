# Description
This is a setup for publishing Ensembl genome browser (see the parent directory) as an npm package.

## Requirements
- Rust
- The `wasm-pack` library

See the Dockerfile in the parent directory for more details about the setup for the Rust build.

Installation in MacOS:

```sh
brew install rustup
rustup-init
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

## Details
- The script (see `package.json` > `scripts:build`) visits the parent directory, builds the Rust code into webassembly using `wasm-pack`, and copies the build output into a temporary directory in the current directory.
- Since `wasm-pack` generates an ES-module build, the script also produces a commonjs build for better compatibility (see the settings in the `rollup.config.js` file).
- Both the ES-module build and the commonjs build are then copied to the `dist` folder, and an npm package is generated and published. See below on further instructions on how to publish the package.

## How to publish a package
Note that this repository has been configured to use a custom package registry set up in an EBI Gitlab instance. The full list of published packages can be seen at at https://gitlab.ebi.ac.uk/ensembl-web/package-registry/-/packages. For configuration details, see the `publishConfig` field of the `package.json`, and the `.npmrc` file.

In order to publish a new package to the registry:
- Make sure that the version of the package you are about to publish [has not been published already](https://gitlab.ebi.ac.uk/ensembl-web/package-registry/-/packages). If it has, then update the version in `package.json`.
- Get the access token from the vault, where it is stored under the key `package-publish-token`
- Use this token to run the following command: `PUBLISH_TOKEN=<access_token> npm publish`

## Custom builds
- To build the genome browser in a chatty mode, when it logs what it's doing to the browser console, build the webassembly (in the parent directory) with the following command: `RUSTFLAGS="--cfg console_noisy" wasm-pack build --target web --release`. Such a build may be useful for debugging communication between the genome browser and the browser chrome.
- To build the genome browser in the development mode, build the webassembly with the following command: `RUSTFLAGS="--cfg console_noisy" wasm-pack build --target web --dev`. Note that development builds are HUGE, and slow. You might need to build one during Rust development or debugging; but normally, it will be rare that you'll need one.