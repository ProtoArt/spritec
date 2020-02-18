const {dialog} = require('electron').remote;
const {basename} = require('path');

const {actions} = require('./slice');
const {loadGLTF} = require('./actions');
const Component = require('../../lib/Component');

class ModelList extends Component {
  onCreate(element) {
    this.state = {
      selectElement: element.querySelector('#spritec-model-select'),
      files: [],
    }

    this.state.selectElement.onchange = (event) => {
      this.dispatch(loadGLTF(event.target.value));
    }

    element.querySelector('#spritec-model-add').onclick = (event) => {
      // Prevent form from sending and refreshing the page
      event.preventDefault();
      // get file
      dialog.showOpenDialog({
        properties: ['openFile'],
        filters: [{name: 'Models', extensions: ['glb', 'gltf']}]
      }).then(({filePaths}) => {
        if (filePaths.length === 0) return;
        this.dispatch(loadGLTF(filePaths[0]));
      });
    }
  }

  mapStateToProps() {
    return {
      files: (state) => state.import.files,
      selectedFile: (state) => state.import.selected.path,
    }
  }

  render() {
    let {files, selectedFile} = this.props;
    let {selectElement} = this.state;

    // Clear the list first
    selectElement.innerHTML = '';

    files.forEach(file => selectElement.add(new Option(
      basename(file), // text shown to user
      file, // value when the option is selected
      false, // default selected
      file === selectedFile // selected
    )));

    selectElement.disabled = (files.length === 0);
    if (files.length === 0) {
      selectElement.add(new Option('Click + to add model'));
    }
  }
}

module.exports = ModelList;
