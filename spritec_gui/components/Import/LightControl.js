const Component = require('../../lib/Component');
const {actions: {
  setLightRotation,
  setLightColor,
  setLightIntensity
}} = require('./slice');

// TODO: support all spritec lights, not just directional
class LightControl extends Component {
  onCreate(element) {
    const colorEle = element.querySelector('#light-color');
    const intensityEle = element.querySelector('#light-intensity');

    const rotX = element.querySelector('#light-rot-x');
    const rotY = element.querySelector('#light-rot-y');
    const rotZ = element.querySelector('#light-rot-z');

    this.state = {
      colorEle, intensityEle, rotX, rotY, rotZ,
      euler: new THREE.Euler(),
      quaternion: new THREE.Quaternion(),
    };

    colorEle.addEventListener('input',
      event => {this.dispatch(setLightColor(event.target.value));},
      false
    );
    intensityEle.oninput = event => {
      this.dispatch(setLightIntensity(Number(event.target.value)));
    };

    rotX.oninput = (event) => {
      this.state.euler.x = THREE.MathUtils.degToRad(Number(event.target.value));
      this.state.quaternion.setFromEuler(this.state.euler);
      this.dispatch(setLightRotation(this.state.quaternion.toArray()));
    };
    rotY.oninput = (event) => {
      this.state.euler.y = THREE.MathUtils.degToRad(Number(event.target.value));
      this.state.quaternion.setFromEuler(this.state.euler);
      this.dispatch(setLightRotation(this.state.quaternion.toArray()));
    };
    rotZ.oninput = (event) => {
      this.state.euler.z = THREE.MathUtils.degToRad(Number(event.target.value));
      this.state.quaternion.setFromEuler(this.state.euler);
      this.dispatch(setLightRotation(this.state.quaternion.toArray()));
    };
  }

  mapStateToProps() {
    return {
      rotation: state => state.import.selected.light_rotation,
      color: state => state.import.selected.light_color,
      intensity: state => state.import.selected.light_intensity,
    };
  }

  render() {
    const {rotation, color, intensity} = this.props;
    const {
      quaternion,
      euler,
      colorEle,
      intensityEle,
      rotX, rotY, rotZ
    } = this.state;

    quaternion.fromArray(rotation);
    euler.setFromQuaternion(quaternion);
    rotX.value = THREE.MathUtils.radToDeg(euler.x);
    rotY.value = THREE.MathUtils.radToDeg(euler.y);
    rotZ.value = THREE.MathUtils.radToDeg(euler.z);

    colorEle.value = color;
    intensityEle.value = intensity;
  }
}

module.exports = LightControl;
