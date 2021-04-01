use crate::utils::{get_optional_u16, ToJsObject};
use napi::{threadsafe_function::*, *};
use napi_derive::js_function;
use std::{
  sync::Arc,
  thread::{self, JoinHandle},
};
use usb_enumeration::{Observer, Subscription, UsbDevice};

struct Events {
  arrive: Option<ThreadsafeFunction<UsbDevice>>,
  left: Option<ThreadsafeFunction<UsbDevice>>,
}

#[derive(Clone)]
struct WatchContext {
  thread_handle: Arc<JoinHandle<()>>,
  subscription: Subscription,
}

impl WatchContext {
  pub fn new(events: Events, vendor_id: Option<u16>, product_id: Option<u16>) -> Result<Self> {
    let mut observer = Observer::new();
    if let Some(vendor_id) = vendor_id {
      observer = observer.with_vendor_id(vendor_id);
    }
    if let Some(product_id) = product_id {
      observer = observer.with_product_id(product_id);
    }
    let subscription = observer.subscribe();

    let thread_handle = thread::spawn({
      let subscription = subscription.clone();

      move || loop {
        for event in subscription.rx_event.iter() {
          match event {
            usb_enumeration::Event::Initial(_) => {}
            usb_enumeration::Event::Connect(device) => {
              if let Some(arrive) = &events.arrive {
                arrive.call(Ok(device), ThreadsafeFunctionCallMode::NonBlocking);
              }
            }
            usb_enumeration::Event::Disconnect(device) => {
              if let Some(left) = &events.left {
                left.call(Ok(device), ThreadsafeFunctionCallMode::NonBlocking);
              }
            }
          }
        }
      }
    });

    Ok(WatchContext {
      thread_handle: Arc::new(thread_handle),
      subscription,
    })
  }

  pub fn into_js_object(self, env: &mut Env) -> Result<JsObject> {
    let mut obj = env.create_object()?;
    env.wrap(&mut obj, self)?;
    Ok(obj)
  }
}

fn get_threadsafe_fn(ctx: &CallContext, index: usize) -> Option<ThreadsafeFunction<UsbDevice>> {
  ctx.get::<JsFunction>(index).ok().and_then(|f| {
    ctx
      .env
      .create_threadsafe_function(&f, 0, |ctx: ThreadSafeCallContext<UsbDevice>| {
        ctx.value.to_js_object(ctx.env).map(|r| vec![r])
      })
      .ok()
  })
}

#[js_function(4)]
pub fn watch(ctx: CallContext) -> Result<JsObject> {
  let arrive = get_threadsafe_fn(&ctx, 0);
  let left = get_threadsafe_fn(&ctx, 1);

  let vendor_id = get_optional_u16(&ctx, 2)?;
  let product_id = get_optional_u16(&ctx, 3)?;

  let watch = WatchContext::new(Events { arrive, left }, vendor_id, product_id)?;

  watch.into_js_object(ctx.env)
}
