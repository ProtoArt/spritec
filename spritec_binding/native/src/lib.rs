use neon::prelude::*;

fn version(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(cx.string(spritec::meta::version()))
}

register_module!(mut cx, {
    cx.export_function("version", version)
});
