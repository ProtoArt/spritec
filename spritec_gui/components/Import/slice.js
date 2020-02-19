const {createSlice} = require('@reduxjs/toolkit');

const initialSelectedState = {
  path: null, // file path of selected model
  cameras: [], // list of camera {id, name}
  animations: [], // list of animations {name, duration}
  animation: {
    name: null, // null means no animation
    duration: 1, // duration of animation in seconds
  },
  animation_total_steps: 16, // number of frames to generate for the animation
  camera: {
    id: null,
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
};

const importSlice = createSlice({
  name: 'import',
  initialState: {
    selected: initialSelectedState,
    useCustomCamera: false,
    files: [],
  },
  reducers: {
    stopAnimation(state, _) {state.selected.animation = null},
    setAnimation(state, action) {state.selected.animation = action.payload},
    setWidth(state, action) {state.selected.width = action.payload},
    setHeight(state, action) {state.selected.height = action.payload},
    clearSelected(state, _) {state.selected = initialSelectedState},
    loadModel(state, action) {
      const {path, cameras, animations} = action.payload;
      // Convert to a set and convert back to array. This way if we load the
      // same model twice we won't have duplicate entries.
      state.files = Array.from(new Set(state.files).add(path))
      state.selected.path = path;

      // If no cameras then enable custom camera
      state.useCustomCamera = (cameras.length === 0);
      state.selected.cameras = cameras;

      state.selected.animations = animations;
    },
    setCamera(state, action) {
      state.selected.camera = action.payload;
    }
  }
})

module.exports = importSlice;
