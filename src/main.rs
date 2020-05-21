
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

use typenum::Unsigned;

#[global_allocator]
static mut ALLOCATOR: LockedHeap = LockedHeap::empty();

// We need a function to handle our allocation errors.
#[alloc_error_handler]
fn allocation_error_handler(_: core::alloc::Layout) -> ! {
    // We just pass our error as a panic.
    panic!("Allocation failure.");
}

use lazy_static::lazy_static;

lazy_static! {
    pub static ref PERIPHERALS: Mutex<msp432p401r::Peripherals> = Mutex::new(msp432p401r::Peripherals::take().unwrap());
}

/// A task to be ran.
struct Task {
    function: fn()
}

impl Task {
    fn new(function: fn()) -> Task {
        Task {
            function
        }
    }

    fn run(&self) {
        (self.function)();
    }
}

fn blink<N: Unsigned, M: Unsigned, D: Unsigned>() {
    for _ in 0..N::to_u16() {
        cortex_m::interrupt::free(|cs| {
            let p = PERIPHERALS.borrow(cs);
    
            // Get the Digital I/O module
            let dio = &p.DIO;
    
            // Set red LED output to on.
            dio.paout.modify(|r, w| unsafe { w.p2out().bits(r.p2out().bits() | M::to_u8()) });
        });
        asm::delay(D::to_u32() * 100000);
    
        cortex_m::interrupt::free(|cs| {
            let p = PERIPHERALS.borrow(cs);
    
            // Get the Digital I/O module
            let dio = &p.DIO;
    
            // Set red LED output to off.
            dio.paout.modify(|r, w| unsafe { w.p2out().bits(r.p2out().bits() & !M::to_u8()) });
        });
        asm::delay(D::to_u32() * 100000);
    }
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

    let mut task_list = vec![
        Task::new(blink::<typenum::U5 , typenum::U1, typenum::U6>),
        Task::new(blink::<typenum::U10, typenum::U2, typenum::U3>),
        Task::new(blink::<typenum::U15, typenum::U4, typenum::U2>)
        ];

    cortex_m::interrupt::free(|cs| {
        let p = PERIPHERALS.borrow(cs);

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

        // The red LED is on port 2 pin 0.
        // The green LED is on port 2 pin 1.
        // The blue LED is on port 2 pin 2.
        // Set them all up as outputs.
        dio.padir.modify(|r, w| unsafe { w.p2dir().bits(r.p2dir().bits() | 0x07) });

        // Turn all the LEDs off.
        dio.paout.modify(|r, w| unsafe { w.p2out().bits(r.p2out().bits() & !7) });
    });

    loop {
        for task in task_list.iter_mut() {
            task.run();
        }
    }
}
