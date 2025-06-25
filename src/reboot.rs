/// Reboots the device into the HalfKay bootloader,
/// putting it into firmware flashing mode.
pub fn reboot_to_bootloader() -> ! {
    unsafe {
        core::arch::asm!("bkpt #251");
    }
    panic!("Failed to reboot to bootloader!");
}
