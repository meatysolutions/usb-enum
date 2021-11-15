use napi::{bindgen_prelude::*, threadsafe_function::*, JsFunction};
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, RwLock},
    thread::{self, JoinHandle},
};
use usb_enumeration::{enumerate, Observer, Subscription, UsbDevice as InternalUsbDevice};

#[napi(object)]
#[derive(Serialize, Deserialize, Debug)]
pub struct UsbDevice {
    pub id: String,
    pub vendor_id: u32,
    pub product_id: u32,
    pub description: Option<String>,
}

impl From<InternalUsbDevice> for UsbDevice {
    fn from(device: InternalUsbDevice) -> Self {
        UsbDevice {
            id: device.id,
            vendor_id: device.vendor_id as u32,
            product_id: device.product_id as u32,
            description: device.description,
        }
    }
}

pub struct List {
    vendor_id: Option<u16>,
    product_id: Option<u16>,
}

#[napi(task)]
impl Task for List {
    type Output = Vec<InternalUsbDevice>;
    type JsValue = Vec<UsbDevice>;

    fn compute(&mut self) -> Result<Self::Output> {
        Ok(enumerate(self.vendor_id, self.product_id))
    }

    fn resolve(&mut self, _env: napi::Env, devices: Self::Output) -> Result<Self::JsValue> {
        Ok(devices.into_iter().map(|d| d.into()).collect())
    }
}

#[napi]
pub fn list(vendor_id: Option<u32>, product_id: Option<u32>) -> AsyncTask<List> {
    AsyncTask::new(List {
        vendor_id: vendor_id.map(|v| v as u16),
        product_id: product_id.map(|v| v as u16),
    })
}

#[derive(PartialEq)]
enum Event {
    Connect,
    Disconnect,
}

struct WatchSubscription {
    event: Event,
    vendor_id: Option<u16>,
    product_id: Option<u16>,
    func: ThreadsafeFunction<UsbDevice>,
}

impl WatchSubscription {
    pub fn parse(method: String, callback: JsFunction) -> Result<Self> {
        let func =
            callback.create_threadsafe_function(0, |ctx: ThreadSafeCallContext<UsbDevice>| {
                ctx.env.to_js_value(&ctx.value).map(|v| vec![v])
            })?;

        todo!()
    }

    pub fn matches(&self, event: &Event, device: &InternalUsbDevice) -> bool {
        if &self.event != event {
            return false;
        }

        if let Some(vendor_id) = self.vendor_id {
            if vendor_id != device.vendor_id {
                return false;
            }
        }

        if let Some(product_id) = self.product_id {
            if product_id != device.product_id {
                return false;
            }
        }

        true
    }
}

#[napi]
#[derive(Clone)]
pub struct Watch {
    thread_handle: Option<Arc<JoinHandle<()>>>,
    subscription: Subscription,
    listeners: Arc<RwLock<Vec<WatchSubscription>>>,
}

#[napi]
impl Watch {
    #[napi(constructor)]
    pub fn new() -> Result<Self> {
        let observer = Observer::new().with_poll_interval(5);
        let subscription = observer.subscribe();

        let mut watcher = Watch {
            thread_handle: None,
            subscription,
            listeners: Default::default(),
        };

        let thread_handle = thread::spawn({
            let this = watcher.clone();

            move || loop {
                for event in this.subscription.rx_event.iter() {
                    match event {
                        usb_enumeration::Event::Connect(device) => {
                            this.dispatch(Event::Connect, device);
                        }
                        usb_enumeration::Event::Disconnect(device) => {
                            this.dispatch(Event::Disconnect, device);
                        }
                        _ => {}
                    }
                }
            }
        });

        watcher.thread_handle = Some(Arc::new(thread_handle));

        Ok(watcher)
    }

    #[napi]
    pub fn on(&self, method: String, callback: JsFunction) -> Result<()> {
        self.listeners
            .write()
            .unwrap()
            .push(WatchSubscription::parse(method, callback)?);

        Ok(())
    }

    fn dispatch(&self, event: Event, device: InternalUsbDevice) {
        let listeners = self.listeners.read().unwrap();

        for listener in listeners.iter() {
            if listener.matches(&event, &device) {
                listener.func.call(
                    Ok(device.clone().into()),
                    ThreadsafeFunctionCallMode::NonBlocking,
                );
            }
        }
    }
}
