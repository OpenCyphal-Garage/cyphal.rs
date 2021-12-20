use core::sync::atomic::{AtomicUsize, Ordering};

use defmt_rtt as _; // global logger

use panic_probe as _;
use stm32g4xx_hal as _;
// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

static COUNT: AtomicUsize = AtomicUsize::new(0);
defmt::timestamp!("{=usize}", {
    // NOTE(no-CAS) `timestamps` runs with interrupts disabled
    let n = COUNT.load(Ordering::Relaxed);
    COUNT.store(n + 1, Ordering::Relaxed);
    n
});

/// Terminates the application and makes `probe-run` exit with exit-code = 0
#[allow(unused)]
pub fn exit() -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}
