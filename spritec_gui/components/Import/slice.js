const {createSlice} = require('@reduxjs/toolkit');

const importSlice = createSlice({
  name: 'import',
  initialState: {
    selected: {
      path: null, // file path of selected model
      cameras: [], // list of camera names for this model
      camera: {
        name: null,
        position: [0, 0, 0],
        rotation: [0, 0, 0, 1], // quaternion
        scale: [1, 1, 1],
        aspect_ratio: 1,
        near_z: 0.1,
        far_z: 2000,
        fov: 50, // field of view (from bottom to top, in degrees)
      },
      width: 64,
      height: 64,
    },
    useCustomCamera: false,
    files: [],
  },
  reducers: {
    setWidth(state, action) {state.selected.width = action.payload},
    setHeight(state, action) {state.selected.height = action.payload},
    loadModel(state, action) {
      const {path, cameras} = action.payload;
      // Convert to a set and convert back to array. This way if we load the
      // same model twice we won't have duplicate entries.
      state.files = Array.from(new Set(state.files).add(path))
      state.selected.path = path;

      // If no cameras then enable custom camera
      state.useCustomCamera = (cameras.length === 0);
      state.selected.cameras = cameras;
    },
    setCamera(state, action) {
      state.selected.camera = action.payload;
    }
  }
})

module.exports = importSlice;
