use napi::{bindgen_prelude::*, threadsafe_function::*};
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use std::{
    sync::Arc,
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

#[napi]
#[derive(Clone)]
pub struct Watch {
    thread_handle: Arc<JoinHandle<()>>,
    subscription: Subscription,
}

#[napi]
impl Watch {
    #[napi(constructor)]
    pub fn new(
        connected: Option<JsFunction>,
        disconnected: Option<JsFunction>,
        vendor_id: Option<u32>,
        product_id: Option<u32>,
    ) -> Result<Self> {
        let connected = connected
            .map(|f| {
                f.create_threadsafe_function(0, |ctx: ThreadSafeCallContext<UsbDevice>| {
                    ctx.env.to_js_value(&ctx.value).map(|v| vec![v])
                })
            })
            .transpose()?;

        let disconnected = disconnected
            .map(|f| {
                f.create_threadsafe_function(0, |ctx: ThreadSafeCallContext<UsbDevice>| {
                    ctx.env.to_js_value(&ctx.value).map(|v| vec![v])
                })
            })
            .transpose()?;

        let mut observer = Observer::new();
        if let Some(vendor_id) = vendor_id {
            observer = observer.with_vendor_id(vendor_id as u16);
        }
        if let Some(product_id) = product_id {
            observer = observer.with_product_id(product_id as u16);
        }
        let subscription = observer.subscribe();

        let thread_handle = thread::spawn({
            let subscription = subscription.clone();

            move || loop {
                for event in subscription.rx_event.iter() {
                    match event {
                        usb_enumeration::Event::Initial(_) => {}
                        usb_enumeration::Event::Connect(device) => {
                            if let Some(connected) = &connected {
                                connected.call(
                                    Ok(device.into()),
                                    ThreadsafeFunctionCallMode::NonBlocking,
                                );
                            }
                        }
                        usb_enumeration::Event::Disconnect(device) => {
                            if let Some(disconnected) = &disconnected {
                                disconnected.call(
                                    Ok(device.into()),
                                    ThreadsafeFunctionCallMode::NonBlocking,
                                );
                            }
                        }
                    }
                }
            }
        });

        Ok(Watch {
            thread_handle: Arc::new(thread_handle),
            subscription,
        })
    }
}
