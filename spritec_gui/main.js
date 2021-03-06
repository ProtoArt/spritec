const AnimationList = require('./components/Import/AnimationList');
const CameraControl = require('./components/Import/CameraControl');
const CameraList = require('./components/Import/CameraList');
const ImportCanvas = require('./components/Import/ImportCanvas');
const ImportPanel = require('./components/Import/ImportPanel');
const importSlice = require('./components/Import/slice');
const ExportModal = require('./components/Export/ExportModal');
const LightControl = require('./components/Import/LightControl');
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
  new CameraControl(document, store),
  new LightControl(document, store),
];

store.subscribe(() => {
  let state = store.getState();
  components.forEach((component) => {
    if (component._updateComponent(state)) {
      component.render();
    }
  });
});

// TODO: implement proper error handling in the app
// Loudly announce all errors. Reload the app so it's not left in a dead state.
window.onerror = (message) => {alert(message); location.reload();}
window.onunhandledrejection = ({reason}) => {alert(reason); location.reload();}
