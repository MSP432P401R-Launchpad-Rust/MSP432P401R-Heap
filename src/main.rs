
#![no_main]
#![no_std]
#![feature(alloc_error_handler)] // The alloc error handler is experemental and we must opt-in to use it.

extern crate cortex_m;
extern crate cortex_m_rt;
extern crate msp432p401r;
extern crate panic_halt;

use cortex_m_rt::entry;
use cortex_m::asm;

use cortex_m::interrupt::Mutex;

extern crate alloc;
extern crate linked_list_allocator;

use alloc::vec;
use linked_list_allocator::LockedHeap;

#[global_allocator]
static mut ALLOCATOR: LockedHeap = LockedHeap::empty();

// We need a function to handle our allocation errors.
#[alloc_error_handler]
fn allocation_error_handler(_: core::alloc::Layout) -> ! {
    // We just pass our error as a panic.
    panic!("Allocation failure.");
}

#[entry]
fn main() -> ! {

    // Allocator must be setup before we can use any heap operations.
    let start = cortex_m_rt::heap_start() as usize;
    let size = 1024; // Size of heap in bytes.
    unsafe {
        // We're manipulating a global and we're giving it memory locations.
        // Everything about this is unsafe, so it must be done here.
        ALLOCATOR = LockedHeap::new(start, size);
    }

    let p = Mutex::new(msp432p401r::Peripherals::take().unwrap());

    let task_list = vec![0, 1, 2];

    cortex_m::interrupt::free(|cs| {
        let p = p.borrow(cs);

        // Get the Watchdog Timer
        let wdt = &p.WDT_A;

        // Get the Digital I/O module
        let dio = &p.DIO;

        // We shall disable the timer.
        wdt.wdtctl.write(|w| {
            unsafe {
                w.wdtpw().bits(0x5A);
            }
            w.wdthold().bit(true)
        });

        // The red LED is on port 2 pin 0. Set it to be an output.
        dio.padir.modify(|r, w| unsafe { w.p2dir().bits(r.p2dir().bits() | 0x01) });
    });

    loop {
        // Will put the processor to sleep until the next interrupt happens.
        asm::wfi();
    }
}

// To use this macro, we had to enable the rt feature in the msp432p401r crate. See the Cargo.toml file for details.
// #[interrupt]
// fn PORT1_IRQ() {
//     static mut STATE: bool = false;

//     *STATE = !*STATE;

//     cortex_m::interrupt::free(|_| {
//         let p = PERIPHERALS.lock();

//         // Get the Digital I/O module
//         let dio = &p.DIO;

//         if *STATE {
//             // Set LED output to on.
//             dio.paout.modify(|r, w| unsafe { w.p2out().bits(r.p2out().bits() | 1) });
//         } else {
//             // Set LED output to off.
//             dio.paout.modify(|r, w| unsafe { w.p2out().bits(r.p2out().bits() & 0) });
//         }

//         dio.paifg.write(|w| unsafe { w.p1ifg().bits(0x00) }); // Clear all P1 interrupt flags.
//     });
// }