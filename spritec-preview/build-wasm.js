////////////////////////////////////////////////////////////////////////////
//
// Build script for WASM code.
//
// Pass `--release` to build in release mode (for production).
//
////////////////////////////////////////////////////////////////////////////

// The directory to copy WASM files to
const WASM_DIRECTORY = 'build/wasm'

////////////////////////////////////////////////////////////////////////////

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

function runCmd(cmd) {
  console.log('$', cmd);
  return execSync(cmd, {stdio: 'inherit'});
}

const args = process.argv.slice(2);
if (args.length > 1 || (args[0] && args[0] !== '--release')) {
  console.error("Invalid arguments.");
  console.error(`Usage: ${process.argv[0]} ${process.argv[1]} [--release]`);
  process.exit();
}
const release = args[0] === '--release';

console.log(`Building wasm in ${release ? 'release' : 'debug'} mode`);
const releaseFlag = release ? '--release' : '';
runCmd(`cargo build --target wasm32-unknown-unknown ${releaseFlag}`);

// In general, you shouldn't use *Sync methods. In a simple script like this
// though, it really doesn't matter.
// Read more: https://stackoverflow.com/a/21196961/551904

if (!fs.existsSync(WASM_DIRECTORY)) {
  console.log(`Creating ${WASM_DIRECTORY}`);
  fs.mkdirSync(WASM_DIRECTORY, {recursive: true});
}

const wasmBuildDir = release ? 'release' : 'debug';
const wasmPath = path.join('..', 'target', 'wasm32-unknown-unknown',
  wasmBuildDir, 'spritec_preview.wasm');
const destPath = path.join(WASM_DIRECTORY, 'spritec_preview.wasm');
console.log(`Copying '${wasmPath}' to '${destPath}'`);
fs.copyFileSync(wasmPath, destPath);

// wasm-gc cuts down the size of the file *significantly*

console.log("Running wasm-gc...")
runCmd(`wasm-gc ${destPath}`);

console.log("Finished successfully.");
