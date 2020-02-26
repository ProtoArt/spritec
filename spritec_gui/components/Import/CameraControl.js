const Component = require('../../lib/Component');
const {actions: {setCameraPosition, setCameraRotation}} = require('./slice');

const dispatchPosition = (dispatch, action, index) => {
  return (event) => {
    dispatch(action([Number(event.target.value), 0]));
  }
}

class CameraControl extends Component {
  onCreate(element) {
    const camPosX = element.querySelector('#cam-pos-x');
    const camPosY = element.querySelector('#cam-pos-y');
    const camPosZ = element.querySelector('#cam-pos-z');
    const camRotX = element.querySelector('#cam-rot-x');
    const camRotY = element.querySelector('#cam-rot-y');
    const camRotZ = element.querySelector('#cam-rot-z');
    const euler = new THREE.Euler();
    const quaternion = new THREE.Quaternion();

    [camPosX, camPosY, camPosZ].forEach((camPos, index) => {
      camPos.oninput = (event) => {
        this.dispatch(setCameraPosition([Number(event.target.value), index]));
      };
    });

    camRotX.oninput = (event) => {
      this.state.euler.x = THREE.MathUtils.degToRad(Number(event.target.value));
      this.state.quaternion.setFromEuler(this.state.euler);
      this.dispatch(setCameraRotation(this.state.quaternion.toArray()));
    }
    camRotY.oninput = (event) => {
      this.state.euler.y = THREE.MathUtils.degToRad(Number(event.target.value));
      this.state.quaternion.setFromEuler(this.state.euler);
      this.dispatch(setCameraRotation(this.state.quaternion.toArray()));
    }
    camRotZ.oninput = (event) => {
      this.state.euler.z = THREE.MathUtils.degToRad(Number(event.target.value));
      this.state.quaternion.setFromEuler(this.state.euler);
      this.dispatch(setCameraRotation(this.state.quaternion.toArray()));
    }

    this.state = {
      camPosX, camPosY, camPosZ, camRotX, camRotY, camRotZ,
      euler, quaternion,
    };

  }

  mapStateToProps() {
    return {
      camPosition: state => state.import.selected.camera.position,
      camRotation: state => state.import.selected.camera.rotation,
    };
  }

  render() {
    const [pX, pY, pZ] = this.props.camPosition;
    const {
      camPosX, camPosY, camPosZ,
      camRotX, camRotY, camRotZ,
      euler, quaternion
    } = this.state;

    camPosX.value = pX;
    camPosY.value = pY;
    camPosZ.value = pZ;

    quaternion.fromArray(this.props.camRotation);
    euler.setFromQuaternion(quaternion);

    camRotX.value = THREE.MathUtils.radToDeg(euler.x);
    camRotY.value = THREE.MathUtils.radToDeg(euler.y);
    camRotZ.value = THREE.MathUtils.radToDeg(euler.z);
  }
}

module.exports = CameraControl;
