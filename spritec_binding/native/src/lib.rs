use neon::prelude::*;
use spritec::math::{
    Mat4,
    Milliseconds,
    Quaternion,
    Radians,
    Rgb,
    Rgba,
    Vec3,
    Vec4
};
use spritec::query3d::{
    AnimationPosition,
    AnimationQuery,
    File,
    GeometryFilter,
    GeometryQuery,
};
use spritec::renderer::{
    Camera,
    FileQuery,
    Light,
    Outline,
    RenderCamera,
    RenderJob,
    RenderLights,
    RenderNode,
    RenderedImage,
    Size,
    ThreadRenderContext,
};
use spritec::scene::{CameraType, LightType};
use std::cmp::max;
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct Spritec {
    file: Option<Arc<Mutex<File>>>,
    ctx: ThreadRenderContext,
}

declare_types! {
    pub class JsSpritec for Spritec {
        init(_cx) {
            Ok(Spritec {
                file: None,
                ctx: ThreadRenderContext::new()
                        .expect("Unable to create ThreadRenderContext"),
            })
        }

        method setFile(mut cx) {
            let path = cx.argument::<JsString>(0)?.value();
            let mut this = cx.this();
            cx.borrow_mut(&mut this, |mut spritec| {
                let file = File::open(Path::new(&path)).expect("Unable to open file");
                spritec.file = Some(Arc::new(Mutex::new(file)));
            });
            Ok(cx.undefined().upcast())
        }
        /*

        /// Saves all the sprites on one spritesheet.
        method saveSpritesheet(mut cx) {
            Ok(cx.undefined().upcast())
        }

        /// Saves each sprite as its own image.
        method saveSprites(mut cx) {
            Ok(cx.undefined().upcast())
        }

*/
        method saveGif(mut cx) {
            let mut this = cx.this();

            let file = cx.borrow(&this, |spritec| {
                spritec.file.as_ref().expect("No file to render").clone()
            });

            // Arguments from JavaScript
            let path_out = cx.argument::<JsString>(0)?.value();
            let width = cx.argument::<JsNumber>(1)?.value() as u32;
            let height = cx.argument::<JsNumber>(2)?.value() as u32;
            let scale = cx.argument::<JsNumber>(3)?.value() as u32;

            let cam_position = cx.argument::<JsArrayBuffer>(4)?;
            let cam_rotation = cx.argument::<JsArrayBuffer>(5)?;
            let cam_scale = cx.argument::<JsArrayBuffer>(6)?;
            let cam_aspect_ratio = cx.argument::<JsNumber>(7)?.value() as f32;
            let cam_near_z = cx.argument::<JsNumber>(8)?.value() as f32;
            let cam_far_z = cx.argument::<JsNumber>(9)?.value() as f32;
            let cam_fov_deg = cx.argument::<JsNumber>(10)?.value() as f32;

            // Map JavaScript null to default values (None, 0.0).
            let (animation_name, animation_duration) = cx
                .argument::<JsValue>(11)?
                .downcast::<JsObject>()
                .map_or((None, 0.0), |obj| {
                    let name = obj
                        .get(&mut cx, "name")
                        .expect("'name' property not found in animation")
                        .downcast::<JsString>().unwrap().value();
                    let duration = obj
                        .get(&mut cx, "duration")
                        .expect("'duration' property not found in animation")
                        .downcast::<JsNumber>().unwrap().value() as f32;
                    (Some(name), duration)
                });
            let animation_total_steps = cx.argument::<JsNumber>(12)?.value() as u32;

            let file_out = std::fs::File::create(
                Path::new(&path_out)
            ).expect("Can't write to destination");

            let camera = Arc::new(create_camera(
                Vec3::from(take_3(&cx, &cam_position)),
                Quaternion::from(Vec4::from(take_4(&cx, &cam_rotation))),
                Vec3::from(take_3(&cx, &cam_scale)),
                cam_aspect_ratio,
                cam_near_z,
                cam_far_z,
                cam_fov_deg,
            ));

            let lights = Arc::new(vec![Arc::new(Light {
                data: Arc::new(LightType::Directional {
                    color: Rgb::white(),
                    intensity: 1.0,
                }),
                world_transform: Mat4::rotation_x((-60.0f32).to_radians()),
            })]);

            // Delay between frames, where each unit is 10ms.
            let frame_delay = {
                let delay_ms = (animation_duration * 1000.0) / (animation_total_steps as f32);
                (delay_ms / 10.0).round() as u16
            };

            let frames = cx.borrow_mut(&mut this, |mut spritec| {
                let mut frames = Vec::<gif::Frame>::new();
                for animation_cur_step in 0..animation_total_steps {
                    let job = RenderJob {
                        scale: NonZeroU32::new(scale).expect("Scale to be a positive integer"),
                        root: RenderNode::RenderedImage(describe_sprite(
                                file.clone(),
                                width,
                                height,
                                camera.clone(),
                                lights.clone(),
                                &animation_name,
                                animation_total_steps,
                                animation_cur_step,
                        )),
                    };
                    let image = job.execute(&mut spritec.ctx).expect("Sprite creation failed");
                    let mut frame = gif::Frame::from_rgba(
                        image.width() as u16,
                        image.height() as u16,
                        &mut image.into_raw(),
                    );
                    frame.delay = frame_delay;
                    frame.dispose = gif::DisposalMethod::Background;
                    frames.push(frame);
                }

                frames
            });

            let (final_width, final_height) = {
                let frame = frames.first().unwrap();
                (frame.width, frame.height)
            };

            let mut encoder = gif::Encoder::new(file_out, final_width, final_height, &[])
                .expect("Failed to create gif encoder");
            frames.iter().for_each(|frame| {
                encoder.write_frame(&frame).expect("Failed to encode gif")
            });

            Ok(cx.undefined().upcast())
        }

        method render(mut cx) {
            let mut this = cx.this();

            let file = cx.borrow(&this, |spritec| {
                spritec.file.as_ref().expect("No file to render").clone()
            });

            // Arguments from JavaScript
            let width = cx.argument::<JsNumber>(0)?.value() as u32;
            let height = cx.argument::<JsNumber>(1)?.value() as u32;

            let cam_position = cx.argument::<JsArrayBuffer>(2)?;
            let cam_rotation = cx.argument::<JsArrayBuffer>(3)?;
            let cam_scale = cx.argument::<JsArrayBuffer>(4)?;
            let cam_aspect_ratio = cx.argument::<JsNumber>(5)?.value() as f32;
            let cam_near_z = cx.argument::<JsNumber>(6)?.value() as f32;
            let cam_far_z = cx.argument::<JsNumber>(7)?.value() as f32;
            let cam_fov_deg = cx.argument::<JsNumber>(8)?.value() as f32;

            // Map JavaScript null to Rust None. If animation_name is None then
            // we do not want animation.
            let animation_name: Option<String> = cx
                .argument::<JsValue>(9)?
                .downcast::<JsString>()
                .map_or(None, |name| Some(name.value()));
            let animation_total_steps = cx.argument::<JsNumber>(10)?.value() as u32;
            let animation_cur_step = cx.argument::<JsNumber>(11)?.value() as u32;

            let camera = Arc::new(create_camera(
                Vec3::from(take_3(&cx, &cam_position)),
                Quaternion::from(Vec4::from(take_4(&cx, &cam_rotation))),
                Vec3::from(take_3(&cx, &cam_scale)),
                cam_aspect_ratio,
                cam_near_z,
                cam_far_z,
                cam_fov_deg,
            ));

            let lights = Arc::new(vec![Arc::new(Light {
                data: Arc::new(LightType::Directional {
                    color: Rgb::white(),
                    intensity: 1.0,
                }),
                world_transform: Mat4::rotation_x((-60.0f32).to_radians()),
            })]);

            let sprite = describe_sprite(
                file,
                width,
                height,
                camera.clone(),
                lights.clone(),
                &animation_name,
                animation_total_steps,
                animation_cur_step,
            );

            let image = cx.borrow_mut(&mut this, |mut spritec| {
                RenderJob {
                    scale: unsafe { NonZeroU32::new_unchecked(1) },
                    root: RenderNode::RenderedImage(sprite),
                }.execute(&mut spritec.ctx).expect("Sprite creation failed")
            });

            let mut array_buffer = cx.array_buffer(image.width() * image.height() * 4)?;
            cx.borrow_mut(&mut array_buffer, |data| {
                let slice = data.as_mut_slice::<u8>();
                for (i, pixel) in image.pixels().enumerate() {
                    slice[i * 4 + 0] = pixel[0];
                    slice[i * 4 + 1] = pixel[1];
                    slice[i * 4 + 2] = pixel[2];
                    slice[i * 4 + 3] = pixel[3];
                }
            });
            Ok(array_buffer.upcast())
        }
    }
}

/// Describes the sprite to be rendered. This information is used by the
/// ThreadRenderContext to render the sprite.
///
/// If `animation_name` is None then the default pose is used.
fn describe_sprite(
    file: Arc<Mutex<File>>,
    width: u32,
    height: u32,
    camera: Arc<Camera>,
    lights: Arc<Vec<Arc<Light>>>,
    animation_name: &Option<String>,
    animation_total_steps: u32,
    animation_cur_step: u32,
) -> RenderedImage {
    RenderedImage {
        size: Size {
            width: NonZeroU32::new(width).expect("Width is not a u32"),
            height: NonZeroU32::new(height).expect("Height is not a u32"),
        },
        background: Rgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        },
        camera: RenderCamera::Camera(camera),
        lights: RenderLights::Lights(lights),
        ambient_light: Rgb::white() * 0.5,
        geometry: FileQuery {
            query: GeometryQuery {
                models: GeometryFilter::all_in_default_scene(),
                animation: animation_name.as_ref().map(|name| AnimationQuery {
                    name: Some(name.to_string()),
                    position: AnimationPosition::RelativeTime {
                        start_time: Milliseconds::from_sec(0.0),
                        weight: get_weight(animation_cur_step, animation_total_steps),
                    },
                }),
            },
            file,
        },
        outline: Outline {
            thickness: 0.0,
            color: Rgba::black(),
        },
    }
}

/// Returns a weight between 0 and 1 given a step and total number of steps.
/// +-----------------------------------------+
/// | total_steps | step       | weight       |
/// +=============+============+==============+
/// |           1 | 0          | 0.0          |
/// |           2 | 0, 1       | 0.0, 1.0     |
/// |           3 | 0, 1, 2    | 0.0, 0.5, 1.0|
/// +-----------------------------------------+
///
/// NOTE: step must be less than total_steps
fn get_weight(step: u32, total_steps: u32) -> f32 {
    step as f32 / max(total_steps - 1, 1) as f32
}

/// Takes the first three f32 numbers from a JavaScript array buffer
fn take_3(cx: &CallContext<JsSpritec>, array_buffer: &Handle<JsArrayBuffer>) -> [f32; 3] {
    cx.borrow(array_buffer, |data| {
        let slice = data.as_slice::<f32>();
        [slice[0], slice[1], slice[2]]
    })
}

/// Takes the first four f32 numbers from a JavaScript array buffer
fn take_4(cx: &CallContext<JsSpritec>, array_buffer: &Handle<JsArrayBuffer>) -> [f32; 4] {
    cx.borrow(array_buffer, |data| {
        let slice = data.as_slice::<f32>();
        [slice[0], slice[1], slice[2], slice[3]]
    })
}

fn create_camera(
    position: Vec3,
    rotation: Quaternion,
    scale: Vec3,
    aspect_ratio: f32,
    near_z: f32,
    far_z: f32,
    fov_deg: f32,
) -> Camera {
    let cam_type = CameraType::Perspective {
        name: None,
        aspect_ratio,
        field_of_view_y: Radians::from_degrees(fov_deg),
        near_z,
        far_z: Some(far_z),
    };

    let scale_mat = Mat4::scaling_3d(scale);
    let rot_mat = Mat4::from(rotation);
    let trans_mat = Mat4::translation_3d(position);

    Camera {
        view: (trans_mat * rot_mat * scale_mat).inverted(),
        projection: cam_type.to_projection(),
    }
}

register_module!(mut cx, {
    cx.export_class::<JsSpritec>("Spritec")?;
    Ok(())
});
