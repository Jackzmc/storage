use std::fmt::Write;
use anyhow::anyhow;
use log::debug;
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
    if let Some(param) = h.param(0) {
        if let Some(obj) = param.value().as_object() {
            let output = serde_json::to_string(obj).unwrap();
            out.write(&output)?;
        } else if let Some(str) = param.value().as_str() {
            out.write(str)?;
        } else {
            out.write("[unknown]")?;
        }
    } else {
        out.write("undefined")?;
    }
    Ok(())
}

pub(crate) fn is_active(h: &Helper< '_>, hbs: &Handlebars<'_>, ctx: &Context, rc: &mut RenderContext<'_, '_>, out: &mut dyn Output)
    -> HelperResult
{
    let current_path = h.param(0)
        .and_then(|v| v.value().as_str())
        .ok_or::<RenderError>(RenderErrorReason::ParamNotFoundForIndex("", 0).into())?;
    let href = h.param(1)
        .and_then(|v| v.value().as_str())
        .ok_or::<RenderError>(RenderErrorReason::ParamNotFoundForIndex("", 1).into())?;
    // debug!("name={} curr={} href={}", h.name(), current_path, href);
    if h.name() == "is-active-exact" {
        if current_path == href {
            out.write("is-active")?;
        }
    } else {
        if current_path.starts_with(href) {
            out.write("is-active")?;
        }
    }
    Ok(())
}