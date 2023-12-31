// This example demonstrates a program that also acts as a rebootor.
// When being programmed with the `-r` flag, it will reboot itself into
// the bootloader.

#![no_std]
#![no_main]

use teensy4_bsp as bsp;

use bsp::pins::common::{P0, P1};
imxrt_uart_panic::register!(LPUART6, P1, P0, 115200, teensy4_panic::sos);

#[rtic::app(device = teensy4_bsp)]
mod app {
    use super::bsp;

    use bsp::board;
    use bsp::hal;
    use bsp::logging;

    use embedded_hal::serial::Write;

    use hal::usbd::{BusAdapter, EndpointMemory, EndpointState, Speed};
    use usb_device::bus::UsbBusAllocator;

    use teensy4_selfrebootor::Rebootor;

    /// This allocation is shared across all USB endpoints. It needs to be large
    /// enough to hold the maximum packet size for *all* endpoints. If you start
    /// noticing panics, check to make sure that this is large enough for all endpoints.
    static EP_MEMORY: EndpointMemory<1024> = EndpointMemory::new();
    /// This manages the endpoints. It's large enough to hold the maximum number
    /// of endpoints; we're not using all the endpoints in this example.
    static EP_STATE: EndpointState = EndpointState::max_endpoints();

    const LOG_POLL_INTERVAL: u32 = board::PERCLK_FREQUENCY / 100;
    const LOG_DMA_CHANNEL: usize = 0;

    #[local]
    struct Local {
        poll_log: hal::pit::Pit<3>,
        log_poller: logging::Poller,
        rebootor: Rebootor<'static>,
    }

    #[shared]
    struct Shared {}

    #[init(local = [bus: Option<UsbBusAllocator<BusAdapter>> = None])]
    fn init(cx: init::Context) -> (Shared, Local) {
        let board::Resources {
            mut dma,
            pit: (_, _, _, mut poll_log),
            pins,
            usb,
            lpuart6,
            ..
        } = board::t40(cx.device);

        // Logging
        let log_dma = dma[LOG_DMA_CHANNEL].take().unwrap();
        let mut log_uart = board::lpuart(lpuart6, pins.p1, pins.p0, 115200);
        for &ch in "\r\n===== Rebootor example =====\r\n\r\n".as_bytes() {
            nb::block!(log_uart.write(ch)).unwrap();
        }
        nb::block!(log_uart.flush()).unwrap();
        let log_poller =
            logging::log::lpuart(log_uart, log_dma, logging::Interrupts::Enabled).unwrap();
        poll_log.set_interrupt_enable(true);
        poll_log.set_load_timer_value(LOG_POLL_INTERVAL);
        poll_log.enable();

        // USB
        let bus = BusAdapter::with_speed(usb, &EP_MEMORY, &EP_STATE, Speed::LowFull);
        let bus = cx.local.bus.insert(UsbBusAllocator::new(bus));
        let rebootor = teensy4_selfrebootor::Rebootor::new(bus);

        (
            Shared {},
            Local {
                log_poller,
                poll_log,
                rebootor,
            },
        )
    }

    #[task(binds = USB_OTG1, priority = 5, local = [rebootor])]
    fn usb1(ctx: usb1::Context) {
        ctx.local.rebootor.poll();
    }

    #[task(binds = PIT, priority = 1, local = [poll_log, log_poller])]
    fn logger(cx: logger::Context) {
        let logger::LocalResources {
            poll_log,
            log_poller,
            ..
        } = cx.local;

        if poll_log.is_elapsed() {
            poll_log.clear_elapsed();

            log_poller.poll();
        }
    }
}
