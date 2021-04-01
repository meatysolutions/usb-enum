use crate::utils::{get_optional_u16, ToJsObject};
use napi::*;
use napi_derive::js_function;
use usb_enumeration::{enumerate, UsbDevice};

struct List {
  vendor_id: Option<u16>,
  product_id: Option<u16>,
}

impl Task for List {
  type Output = Vec<UsbDevice>;
  type JsValue = JsObject;

  fn compute(&mut self) -> Result<Self::Output> {
    Ok(enumerate(self.vendor_id, self.product_id))
  }

  fn resolve(self, env: Env, devices: Self::Output) -> Result<Self::JsValue> {
    let mut array = env.create_array()?;
    for (i, device) in devices.iter().enumerate() {
      array.set_element(i as u32, device.to_js_object(env)?)?;
    }
    Ok(array)
  }
}

#[js_function(2)]
pub fn list(ctx: CallContext) -> Result<JsObject> {
  let vendor_id = get_optional_u16(&ctx, 0)?;
  let product_id = get_optional_u16(&ctx, 1)?;

  ctx
    .env
    .spawn(List {
      vendor_id,
      product_id,
    })
    .map(|task| task.promise_object())
}
