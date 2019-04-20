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

const program = require('commander');
const chokidar = require('chokidar');

program
  .version('0.1.0', '-v, --version')
  .option('--release', 'compile to WASM in --release mode')
  .option('-w, --watch', 'watch for changes and recompile')
  .parse(process.argv);

if (program.watch) {
  watchBuild(program.release)
} else {
  buildWasm(program.release)
}

function runCmd(cmd) {
  console.log('$', cmd);
  return execSync(cmd, {stdio: 'inherit'});
}

function buildWasm(release = false) {
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
}

function watchBuild(release) {
  let build = () => {
    try {
      buildWasm(release);
    } catch (e) {
      console.error(e);
      console.log('Build failed!');
    }
    console.log(`[${new Date(Date.now()).toLocaleString()}] Watching for changes...`);
  };

  // https://thisdavej.com/how-to-watch-for-files-changes-in-node-js/
  let fsWait = false;
  const watcher = (event, filename) => {
    if (!filename || fsWait) {
      return;
    }

    // debounce to make sure we don't re-build too many times too quickly
    fsWait = setTimeout(() => {
      fsWait = false;
    }, 400);

    console.log('==========================');
    build();
  };

  const watchedDirs = [
    'src',
    '../src',
  ];

  chokidar.watch(watchedDirs, {ignored: /(^|[\/\\])\../}).on('all', watcher);
}
