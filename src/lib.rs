#![no_std]
#![deny(missing_docs)]
#![doc = include_str!("../README.md")]
#![doc(issue_tracker_base_url = "https://github.com/Finomnis/teensy4-selfrebootor/issues")]
#![cfg_attr(docsrs, feature(doc_cfg))]

use teensy4_bsp::hal::usbd::BusAdapter;
use usb_device::{class_prelude::*, prelude::*};
use usbd_hid::{descriptor::SerializedDescriptor, hid_class::HIDClass};

mod hid_descriptor;
mod reboot;

pub use reboot::reboot_to_bootloader;

/// The rebootor USB driver.
///
/// Once it receives a reboot request (`teensy_loader_cli -r`), it reboots
/// the device into the HalfKay bootloader for flashing.
///
/// This allows reflashing without having to press the `boot` hardware button.
pub struct Rebootor<'a> {
    class: HIDClass<'a, BusAdapter>,
    device: UsbDevice<'a, BusAdapter>,
    configured: bool,
}

impl<'a> Rebootor<'a> {
    /// Creates a rebootor usb device.
    ///
    /// In order for the device to function, its `poll` function has to be called
    /// periodically.
    ///
    /// For more information, see the crate's examples.
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

    /// Needs to be called every couple of milliseconds for the USB device to work
    /// properly.
    ///
    /// See the crate's examples for more information.
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

            match self.class.pull_raw_output(&mut buf) {
                Ok(len) => {
                    let buf = &buf[..len];
                    if buf == b"reboot" {
                        log::info!("Rebooting to HalfKay ...");
                        reboot::reboot_to_bootloader();
                    }
                }
                Err(usb_device::UsbError::WouldBlock) => (),
                Err(_) => {}
            }

            match self.class.pull_raw_report(&mut buf) {
                Ok(info) => {
                    let buf = &buf[..info.len];
                    if buf == b"reboot" {
                        log::info!("Rebooting to HalfKay ...");
                        reboot::reboot_to_bootloader();
                    }
                }
                Err(usb_device::UsbError::WouldBlock) => (),
                Err(_) => {}
            }
        }
    }
}
