use crate::board::{Board, Clock};

use cortex_m::peripheral::{DCB, DWT};

use hal::{
    rcc::{Config, PLLSrc, PllConfig, Rcc, RccExt, SysClockSrc},
    stm32::Peripherals,
};
use stm32g4xx_hal as hal;

pub struct StmClock;

impl Clock for StmClock {
    fn cycles(&self) -> u64 {
        DWT::get_cycle_count() as u64
    }
}

pub struct Stm32G4 {
    rcc: Rcc,
}

impl Board for Stm32G4 {
    type Clock = StmClock;

    fn setup() -> Self {
        // define peripherals of the board
        let dp = Peripherals::take().unwrap();
        let cp = cortex_m::Peripherals::take().expect("cannot take core peripherals");
        let rcc = dp.RCC.constrain();
        let rcc = config_rcc(rcc);

        init_timer(cp.DWT, cp.DCB);
        Self { rcc }
    }

    fn get_clock(&self) -> Self::Clock {
        StmClock {}
    }

    fn get_clock_frequency(&self) -> embedded_time::rate::Hertz {
        embedded_time::rate::Hertz::new(self.rcc.clocks.ahb_clk.0)
    }
}

fn config_rcc(rcc: Rcc) -> Rcc {
    rcc.freeze(
        Config::new(SysClockSrc::PLL)
            .pll_cfg(PllConfig {
                mux: PLLSrc::HSI,
                m: 4,
                n: 85,
                r: 2,
                q: Some(2),
                p: Some(2),
            })
            .ahb_psc(hal::rcc::Prescaler::NotDivided)
            .apb_psc(hal::rcc::Prescaler::NotDivided),
    )
}

fn init_timer(mut dwt: DWT, mut dcb: DCB) {
    dcb.enable_trace();
    dwt.enable_cycle_counter();

    // now the CYCCNT counter can't be stopped or reset
    drop(dwt);
}
