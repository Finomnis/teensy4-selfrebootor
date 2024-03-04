/// Reboots the device into the HalfKay bootloader,
/// putting it into firmware flashing mode.
pub fn reboot_to_bootloader() -> ! {
    loop {
        unsafe {
            core::arch::asm!("bkpt #251");
        }
    }
}
