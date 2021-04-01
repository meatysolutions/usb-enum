use napi::*;
use std::{fmt::Display, usize};
use usb_enumeration::UsbDevice;

pub fn to_js_error<T: Display>(error: T) -> napi::Error {
  napi::Error::new(Status::Unknown, error.to_string())
}

pub trait ToJsObject {
  fn to_js_object(&self, env: Env) -> Result<JsObject>;
}

impl ToJsObject for UsbDevice {
  fn to_js_object(&self, env: Env) -> Result<JsObject> {
    let mut obj = env.create_object()?;

    obj.set_named_property("id", env.create_string(&self.id)?)?;
    obj.set_named_property("vendor_id", env.create_int32(self.vendor_id as i32)?)?;
    obj.set_named_property("product_id", env.create_int32(self.product_id as i32)?)?;

    if let Some(description) = &self.description {
      obj.set_named_property("description", env.create_string(description)?)?;
    }

    Ok(obj)
  }
}

pub fn get_optional_u16(ctx: &CallContext, index: usize) -> Result<Option<u16>> {
  let arg2 = ctx.get::<JsUnknown>(index)?;
  Ok(match arg2.get_type()? {
    ValueType::Undefined | ValueType::Null => None,
    ValueType::Number => arg2.coerce_to_number()?.get_uint32().ok().map(|v| v as u16),
    _ => {
      return Err(to_js_error(
        "First argument (vendor_id), expected number or undefined",
      ))
    }
  })
}
