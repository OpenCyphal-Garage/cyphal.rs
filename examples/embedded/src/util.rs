use stm32g4xx_hal::stm32::GPIOA;

pub fn turn_board_led_on(gpioa: &GPIOA) {
    gpioa.bsrr.write(|w| w.bs5().set_bit());
}

pub fn turn_board_led_off(gpioa: &GPIOA) {
    gpioa.brr.write(|w| w.br5().set_bit());
}
