use napi::{JsObject, Result};
use napi_derive::module_exports;

mod list;
mod utils;
mod watch;

#[module_exports]
fn init(mut exports: JsObject) -> Result<()> {
  exports.create_named_method("list", list::list)?;
  exports.create_named_method("watch", watch::watch)?;
  Ok(())
}
