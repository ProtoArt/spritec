const {createSlice} = require('@reduxjs/toolkit');

const initialSelectedState = {
  path: null, // file path of selected model
  cameras: [], // list of camera {id, name}
  animations: [], // list of animations {name, duration}
  animation: null, // chosen animation {name, duration}, null means no animation
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
  scale: 1,
};

const importSlice = createSlice({
  name: 'import',
  initialState: {
    selected: initialSelectedState,
    files: [],
    // TODO: `submitting` is here temporarily since we are exporting from the
    // import screen. In the future we will export from the spritesheet overview
    submitting: false,
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
      state.selected.cameras = cameras;
      state.selected.animations = animations;
    },
    setCamera(state, action) {
      state.selected.camera = action.payload;
    },
    setCameraPosition(state, action) {
      const [value, index] = action.payload;
      state.selected.camera.position[index] = value;
    },
    setCameraRotation(state, action) {
      state.selected.camera.rotation = action.payload;
    },
    setAnimationSteps(state, action) {
      state.selected.animation_total_steps = action.payload
    },
    setScale(state, action) {state.selected.scale = action.payload},
    startSubmit(state, _) {state.submitting = true},
    endSubmit(state, _) {state.submitting = false},
  }
})

module.exports = importSlice;
