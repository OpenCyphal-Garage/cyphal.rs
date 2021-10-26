pub const RAM_START: u32 = 0x20_000_000;
pub const RAM_STOP: u32 = 0x20_008_000;

static mut STACK_COPY: [u32; 1024 / 4] = [0u32; 1024 / 4];
static mut COPY_SIZE: usize = 0;
static mut START_ADDR: u32 = 0;

static mut STACK_PTR: u32 = 0;

#[inline]
pub unsafe fn copy_stack(start: u32, stop: u32, counter: &mut u32) {
    *counter = start;
    while *counter < stop {
        STACK_COPY[((*counter - start) / 4) as usize] = *(*counter as *const u32);
        *counter += 4;
    }
    COPY_SIZE = ((*counter - start) / 4) as usize;
    START_ADDR = start;
}

#[cfg(feature = "logging")]
pub unsafe fn print_stack() {
    defmt::info!("address | value");
    for i in 0..COPY_SIZE {
        defmt::info!(
            "{:#08x}: {:#08x}",
            START_ADDR + (i * 4) as u32,
            STACK_COPY[i]
        );
    }
}

/// Fills the stack until the actual stack pointer and
/// safes the actual stack pointer at the bottom of the stack.
///
/// ```no_run
///                      _______
///     top 0x2..8000 -> |     |
///         stack ptr -> | ### |
///                      | ### |
/// bottom 0x2...0000 -> | ### |
///                      -------
/// ```
#[inline]
pub unsafe fn fill_stack() {
    extern "C" {
        // These symbols come from `link.x`
        static _stack_start: u8;
    }
    asm!(
        "mov r5, #0x20000000",
        "mov r6, #85",
        "2:",
        "cmp r5, {0}", // test if stack ptr reached
        "beq 3f",
        "str r6, [r5]", // set pattern
        "add r5, #1",
        "b 2b",
        "3:",
        in(reg) &_stack_start as *const u8 as usize,
        options(nostack)
    );
}

/// Fills the whole stack
///
/// ```no_run
///                      _______
///     top 0x2..8000 -> | ### |
///                      | ### |
///                      | ### |
/// bottom 0x2...0000 -> | ### |
///                      -------
/// ```
#[inline]
pub unsafe fn fill_stack_whole_stack() {
    asm!(
        "mov r5, #0x20000000",
        "mov r6, #85",
        "mov r4, #32768",
        "movt r4, #8192",
        "2:",
        "cmp r5, r4", // test if stack ptr reached
        "beq 3f",
        "str r6, [r5]", // set pattern
        "add r5, #1",
        "b 2b",
        "3:",
        "mov r5, #0x20000000",
        "str r4, [r5]" // save end of stack to bottom of stack
    );
}

#[inline]
pub unsafe fn count_used_ram() -> (u32, u32) {
    extern "C" {
        // These symbols come from `link.x`
        static _stack_start: u8;
    }
    let mut used;
    asm!(
        "mov r5, #0x20000000",
        "mov r6, #85",
        "mov {0}, #0",
        "2:",
        "cmp r5, {1}", // test if prev stack ptr reached
        "beq 3f",
        "add r5, #1",
        "ldrb r7, [r5]",
        "cmp r6, r7", // if pattern go on loop
        "beq 2b",
        "add {0}, #1", // add counter if not pattern (used)
        "b 2b",
        "3:",
        out(reg) used,
        in(reg) &_stack_start as *const u8 as usize,
        options(nostack)
    );
    (used, &_stack_start as *const u8 as u32)
}
