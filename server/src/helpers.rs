use std::fmt::Write;
use anyhow::anyhow;
use rocket_dyn_templates::handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext, RenderError, RenderErrorReason};
pub(crate) fn bytes(h: &Helper< '_>, _: &Handlebars<'_>, _: &Context, rc:
&mut RenderContext<'_, '_>, out: &mut dyn Output) -> HelperResult {
    // get parameter from helper or throw an error
    let param = h.param(0).and_then(|v| v.value().as_i64()).unwrap_or(0);
    let output = humanize_bytes::humanize_bytes_decimal!(param);
    out.write(&*output)?;
    Ok(())
}

pub(crate) fn debug(h: &Helper< '_>, _: &Handlebars<'_>, _: &Context, rc:
&mut RenderContext<'_, '_>, out: &mut dyn Output)  -> HelperResult {
    let param = h.param(0)
        .and_then(|v| v.value().as_object())
        .ok_or::<RenderError>(RenderErrorReason::ParamNotFoundForIndex("", 0).into())?;
    let output = serde_json::to_string(param).unwrap();
    out.write(&output)?;
    Ok(())
}