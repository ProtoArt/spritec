const Component = require('../../lib/Component');
const {loadCamera} = require('./actions');

class CameraList extends Component {
  onCreate(element) {
    this.state = {
      listElement: element,
    }
    this.state.listElement.onchange = (event) => {
      this.dispatch(loadCamera(Number(event.target.value)));
    }
  }

  mapStateToProps() {
    return {
      cameras: state => state.import.selected.cameras,
      cameraId: state => state.import.selected.camera.id,
      useCustomCamera: state => state.import.useCustomCamera,
    };
  }

  render() {
    const {cameras, cameraId, useCustomCamera} = this.props;
    const {listElement} = this.state;

    // clear the list
    listElement.innerHTML = '';

    console.log(cameras);
    listElement.disabled = (cameras.length == 0 || useCustomCamera);
    cameras.forEach(camera => listElement.add(new Option(
      camera.name, // text
      camera.id, // value
      false,  // default selected
      camera.id === cameraId // selected
    )));
  }
}

module.exports = CameraList;
