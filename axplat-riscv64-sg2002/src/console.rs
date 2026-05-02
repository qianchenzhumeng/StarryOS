use axplat::{
    console::ConsoleIf,
    mem::{pa, phys_to_virt},
};
use kspin::SpinNoIrq;
use lazyinit::LazyInit;
use dw_apb_uart::DW8250;

use crate::config::devices::UART_PADDR;

static UART: LazyInit<SpinNoIrq<DW8250>> = LazyInit::new();

pub(crate) fn init_early(bytes: &[u8]) {
    UART.init_once({
        let mut uart = unsafe { dw_apb_uart::DW8250::new(UART_PADDR) };
        // uart.init();
        uart.set_ier(false);
        bytes.iter().for_each(|x| {
            if *x == b'\n' {
                uart.putchar(b'\r');
                uart.putchar(*x)
            } else {
                uart.putchar(*x)
            }
        });
        SpinNoIrq::new(uart)
    });
}

struct ConsoleIfImpl;

#[impl_plat_interface]
impl ConsoleIf for ConsoleIfImpl {
    /// Writes bytes to the console from input u8 slice.
    fn write_bytes(bytes: &[u8]) {
        for &c in bytes {
            let mut uart = UART.lock();
            match c {
                b'\n' => {
                    uart.putchar(b'\r');
                    uart.putchar(b'\n');
                }
                c => uart.putchar(c),
            }
        }
    }

    /// Reads bytes from the console into the given mutable slice.
    /// Returns the number of bytes read.
    fn read_bytes(bytes: &mut [u8]) -> usize {
        let mut uart = UART.lock();
        for (i, byte) in bytes.iter_mut().enumerate() {
            match uart.getchar() {
                Some(c) => *byte = c,
                None => return i,
            }
        }
        bytes.len()
    }

    /// Returns the IRQ number for the console, if applicable.
    #[cfg(feature = "irq")]
    fn irq_num() -> Option<usize> {
        // Some(crate::config::devices::UART_IRQ)
        None
    }
}
