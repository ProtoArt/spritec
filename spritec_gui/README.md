# Spritec GUI

## Running the App

This project relies on [neon](https://neon-bindings.com) so make sure you have
everything needed to build neon projects
(see [Getting Started](https://neon-bindings.com/docs/getting-started)).

To install the dependencies of this app, run `npm install` (in this directory).

To run the app, run `npm start` (in this directory).

## Packaging the App

To get a packaged app ready for distribution, run `npm run dist`. The output will be in the `dist/` folder.
It uses [electron-builder](https://github.com/electron-userland/electron-builder) behind the scenes to generate the package for the OS that runs the command.

### Azure Pipelines

Azure pipeline is set up to package the app for all 3 operating systems (Windows, macOS, Linux).
To run the pipeline, either request access to the Azure Pipeline project and use their website to manually kick off a run, or push a branch with a `qa/` prefix (i.e. `qa/new-feature`).

The generated apps will be uploaded to the artifacts for that run.
