# spritec-preview

This is the GUI frontend to spritec. It's purpose is to provide a convenient way
to preview the spritesheets you generate with spritec. This tool is great for
quickly iterating as you update your character's design.

# Building & Running

Make sure you have the following installed:

* [Rust & Cargo](https://rustup.rs/)
* [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
* [Node.js](https://nodejs.org)
* [npm](https://www.npmjs.com/)
* [Yarn Package Manager](https://yarnpkg.com)

Start in the `spritec-preview` directory if you aren't there already.

Do the following installation steps once:

```bash
$ rustup target add wasm32-unknown-unknown
$ yarn
$ yarn run build-wasm
```

Then, to build and run the UI, use the following commands:

```bash
$ yarn run build-wasm
$ yarn start
```
