const AnimationList = require('./components/Import/AnimationList');
const CameraList = require('./components/Import/CameraList');
const ImportCanvas = require('./components/Import/ImportCanvas');
const ImportPanel = require('./components/Import/ImportPanel');
const importSlice = require('./components/Import/slice');
const ExportModal = require('./components/Export/ExportModal');
const ModelList = require('./components/Import/ModelList');
const {configureStore} = require('@reduxjs/toolkit');

const store = configureStore({
  reducer: {
    import: importSlice.reducer,
  },
});

let components = [
  new ImportCanvas(document.getElementById('spritec-container'), store),
  new ImportPanel(document.getElementById('import-panel'), store),
  new ModelList(document.getElementById('spritec-model-list'), store),
  new CameraList(document.getElementById('spritec-camera-list'), store),
  new AnimationList(document.getElementById('spritec-animation-list'), store),
  new ExportModal(document, store),
];

store.subscribe(() => {
  let state = store.getState();
  components.forEach((component) => {
    if (component._updateComponent(state)) {
      component.render();
    }
  });
});
