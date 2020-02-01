const { createSlice } = require('@reduxjs/toolkit');

const importReducer = createSlice({
  name: 'import',
  initialState: {
    files: [],
    selectedFile: null,
    width: 256,
    height: 256,
  },
  reducers: {
    setWidth(state, action) { state.width = action.payload },
    setHeight(state, action) { state.height = action.payload },
    addFile(state, action) {
      state.files.push(action.payload);
      state.selectedFile = action.payload;
    },
    selectFile(state, action) { state.selectedFile = action.payload },
  }
})

module.exports = importReducer;
