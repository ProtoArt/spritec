const Component = require('../../lib/Component');
const {loadCamera} = require('./actions');

class CameraList extends Component {
  onCreate(element) {
    this.state = {
      listElement: element,
    }
    this.state.listElement.onchange = (event) => {
      this.dispatch(loadCamera(event.target.value));
    }
  }

  mapStateToProps() {
    return {
      cameras: state => state.import.selected.cameras,
      cameraName: state => state.import.selected.camera.name,
      useCustomCamera: state => state.import.useCustomCamera,
    };
  }

  render() {
    const {cameras, cameraName, useCustomCamera} = this.props;
    const {listElement} = this.state;

    // clear the list
    listElement.innerHTML = '';

    listElement.disabled = (cameras.length == 0 || useCustomCamera);
    cameras.forEach(camera => listElement.add(new Option(
      camera, // text
      camera, // value
      false,  // default selected
      camera === cameraName // selected
    )));
  }
}

module.exports = CameraList;
