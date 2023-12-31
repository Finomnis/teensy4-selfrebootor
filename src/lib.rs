#![no_std]
//#![deny(missing_docs)]
#![doc = include_str!("../README.md")]
#![doc(issue_tracker_base_url = "https://github.com/Finomnis/teensy4-selfrebootor/issues")]
#![cfg_attr(docsrs, feature(doc_cfg))]

use teensy4_bsp::hal::usbd::BusAdapter;
use usb_device::{class_prelude::*, prelude::*};
use usbd_hid::{descriptor::SerializedDescriptor, hid_class::HIDClass};

mod hid_descriptor;
mod reboot;

pub struct Rebootor<'a> {
    class: HIDClass<'a, BusAdapter>,
    device: UsbDevice<'a, BusAdapter>,
    configured: bool,
}

impl<'a> Rebootor<'a> {
    pub fn new(bus_alloc: &'a UsbBusAllocator<BusAdapter>) -> Self {
        let class = HIDClass::new(bus_alloc, crate::hid_descriptor::Rebootor::desc(), 10);
        let device = UsbDeviceBuilder::new(bus_alloc, UsbVidPid(0x16C0, 0x0477))
            .product("Self-Rebootor")
            .manufacturer("PJRC")
            .self_powered(true)
            .max_packet_size_0(64)
            .build();

        device.bus().set_interrupts(true);

        Self {
            class,
            device,
            configured: false,
        }
    }

    pub fn poll(&mut self) {
        self.device.poll(&mut [&mut self.class]);

        if self.device.state() == UsbDeviceState::Configured {
            if !self.configured {
                self.device.bus().configure();
            }
            self.configured = true;
        } else {
            self.configured = false;
        }

        if self.configured {
            let mut buf = [0u8; 6];

            let result = self.class.pull_raw_output(&mut buf);
            match result {
                Ok(info) => {
                    let buf = &buf[..info];
                    if buf == b"reboot" {
                        log::info!("Rebooting to HalfKay ...");
                        reboot::do_reboot();
                    }
                }
                Err(usb_device::UsbError::WouldBlock) => (),
                Err(_) => {}
            }
        }
    }
}
