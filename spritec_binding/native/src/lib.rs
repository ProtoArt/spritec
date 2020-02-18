use neon::prelude::*;
use spritec::math::{Mat4, Quaternion, Radians, Rgb, Rgba, Vec3, Vec4};
use spritec::query3d::{File, GeometryFilter, GeometryQuery};
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
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Returns the rendered sprite given parameters from JavaScript
fn render_sprite(mut cx: FunctionContext) -> JsResult<JsArrayBuffer> {
    // Arguments from JavaScript
    let path = cx.argument::<JsString>(0)?.value();
    let width = cx.argument::<JsNumber>(1)?.value() as u32;
    let height = cx.argument::<JsNumber>(2)?.value() as u32;

    // TODO: change to class so that we can set camera separately
    let cam_position = cx.argument::<JsArrayBuffer>(3)?;
    let cam_rotation = cx.argument::<JsArrayBuffer>(4)?;
    let cam_scale = cx.argument::<JsArrayBuffer>(5)?;
    let cam_aspect_ratio = cx.argument::<JsNumber>(6)?.value() as f32;
    let cam_near_z = cx.argument::<JsNumber>(7)?.value() as f32;
    let cam_far_z = cx.argument::<JsNumber>(8)?.value() as f32;
    let cam_fov_deg = cx.argument::<JsNumber>(9)?.value() as f32;

    // TODO: Change to return a class so we can reuse resources
    let mut ctx = ThreadRenderContext::new().expect("Unable to create ThreadRenderContext");
    let file = Arc::new(Mutex::new(
        File::open(Path::new(&path)).expect("Unable to open file"),
    ));

    let camera = RenderCamera::Camera(Arc::new(create_camera(
        Vec3::from(take_3(&cx, &cam_position)),
        Quaternion::from(Vec4::from(take_4(&cx, &cam_rotation))),
        Vec3::from(take_3(&cx, &cam_scale)),
        cam_aspect_ratio,
        cam_near_z,
        cam_far_z,
        cam_fov_deg,
    )));

    let job = RenderJob {
        scale: unsafe { NonZeroU32::new_unchecked(1) },
        root: RenderNode::RenderedImage(RenderedImage {
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
            camera,
            lights: RenderLights::Lights(Arc::new(vec![Arc::new(Light {
                data: Arc::new(LightType::Directional {
                    color: Rgb::white(),
                    intensity: 1.0,
                }),
                world_transform: Mat4::rotation_x((-60.0f32).to_radians()),
            })])),
            ambient_light: Rgb::white() * 0.5,
            geometry: FileQuery {
                query: GeometryQuery {
                    models: GeometryFilter::all_in_default_scene(),
                    animation: None,
                },
                file,
            },
            outline: Outline {
                thickness: 0.0,
                color: Rgba::black(),
            },
        }),
    };
    let image = job.execute(&mut ctx).expect("Sprite creation failed");

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
    Ok(array_buffer)
}

/// Takes the first three f32 numbers from a JavaScript array buffer
fn take_3(cx: &FunctionContext, array_buffer: &Handle<JsArrayBuffer>) -> [f32; 3] {
    cx.borrow(array_buffer, |data| {
        let slice = data.as_slice::<f32>();
        [slice[0], slice[1], slice[2]]
    })
}

/// Takes the first four f32 numbers from a JavaScript array buffer
fn take_4(cx: &FunctionContext, array_buffer: &Handle<JsArrayBuffer>) -> [f32; 4] {
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
    cx.export_function("render_sprite", render_sprite)?;
    Ok(())
});
