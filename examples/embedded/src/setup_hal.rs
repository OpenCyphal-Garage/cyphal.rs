use cortex_m_rt::exception;
use stm32g4xx_hal::stm32::{GPIOA, RCC};

#[no_mangle]
#[allow(non_snake_case)]
unsafe fn HAL_FDCAN_MspInit(hfdcan: *mut stm32_hal::FDCAN_HandleTypeDef) {
    let mut GPIO_InitStruct = core::mem::zeroed::<stm32_hal::GPIO_InitTypeDef>();

    if (*hfdcan).Instance == stm32_hal::FDCAN1_BASE as *mut stm32_hal::FDCAN_GlobalTypeDef {
        GPIO_InitStruct.Pin = 0x0800 | 0x1000;
        GPIO_InitStruct.Mode = 0x00000002;
        GPIO_InitStruct.Pull = 0x0;
        GPIO_InitStruct.Speed = 0x0;
        GPIO_InitStruct.Alternate = 0x9;

        stm32_hal::HAL_GPIO_Init(
            stm32_hal::GPIOA_BASE as *mut stm32_hal::GPIO_TypeDef,
            &mut GPIO_InitStruct as *mut stm32_hal::GPIO_InitTypeDef,
        );
    }
}

#[exception]
fn SysTick() {
    unsafe { stm32_hal::HAL_IncTick() };
}

pub struct InitHandles {
    pub fdcan: stm32_hal::FDCAN_HandleTypeDef,
}

pub unsafe fn init_hal_and_config_board(gpioa: &GPIOA, rcc: &RCC) -> InitHandles {
    stm32_hal::HAL_Init();

    rcc.apb2enr.write(|w| w.syscfgen().set_bit());
    rcc.apb1enr1.write(|w| w.pwren().set_bit());

    let mut main_clock = core::mem::zeroed::<stm32_hal::RCC_OscInitTypeDef>();
    main_clock.OscillatorType = stm32_hal::RCC_OSCILLATORTYPE_HSI;
    main_clock.HSIState = stm32_hal::RCC_HSI_ON;
    main_clock.HSICalibrationValue = stm32_hal::RCC_HSICALIBRATION_DEFAULT;
    main_clock.PLL.PLLState = stm32_hal::RCC_PLL_ON;
    main_clock.PLL.PLLSource = stm32_hal::RCC_PLLSOURCE_HSI;
    main_clock.PLL.PLLM = stm32_hal::RCC_PLLM_DIV4;
    main_clock.PLL.PLLN = 85;
    main_clock.PLL.PLLP = stm32_hal::RCC_PLLP_DIV2;
    main_clock.PLL.PLLQ = stm32_hal::RCC_PLLQ_DIV2;
    main_clock.PLL.PLLR = stm32_hal::RCC_PLLR_DIV2;
    stm32_hal::HAL_RCC_OscConfig(&mut main_clock as *mut stm32_hal::RCC_OscInitTypeDef);

    let mut clk_clock = core::mem::zeroed::<stm32_hal::RCC_ClkInitTypeDef>();
    clk_clock.ClockType = stm32_hal::RCC_CLOCKTYPE_HCLK
        | stm32_hal::RCC_CLOCKTYPE_SYSCLK
        | stm32_hal::RCC_CLOCKTYPE_PCLK1
        | stm32_hal::RCC_CLOCKTYPE_PCLK2;
    clk_clock.SYSCLKSource = stm32_hal::RCC_SYSCLKSOURCE_PLLCLK;
    clk_clock.AHBCLKDivider = stm32_hal::RCC_SYSCLK_DIV1;
    clk_clock.APB1CLKDivider = stm32_hal::RCC_HCLK_DIV1;
    clk_clock.APB2CLKDivider = stm32_hal::RCC_HCLK_DIV1;

    stm32_hal::HAL_RCC_ClockConfig(
        &mut clk_clock as *mut stm32_hal::RCC_ClkInitTypeDef,
        0x00000004,
    );

    let mut prefis_clk = core::mem::zeroed::<stm32_hal::RCC_PeriphCLKInitTypeDef>();
    prefis_clk.PeriphClockSelection =
        stm32_hal::RCC_PERIPHCLK_LPUART1 | stm32_hal::RCC_PERIPHCLK_FDCAN;
    prefis_clk.Lpuart1ClockSelection = stm32_hal::RCC_LPUART1CLKSOURCE_PCLK1;
    prefis_clk.FdcanClockSelection = stm32_hal::RCC_FDCANCLKSOURCE_PCLK1;

    stm32_hal::HAL_RCCEx_PeriphCLKConfig(
        &mut prefis_clk as *mut stm32_hal::RCC_PeriphCLKInitTypeDef,
    );

    /* GPIO Ports Clock Enable */
    rcc.ahb2enr.write(|w| w.gpioaen().set_bit());

    /*Configure GPIO pin Output Level (LED) */
    gpioa.moder.write(|w| w.moder5().bits(0b01));
    gpioa.otyper.write(|w| w.ot5().clear_bit());
    gpioa.ospeedr.write(|w| w.ospeedr5().bits(0b11));
    gpioa.pupdr.write(|w| w.pupdr5().bits(0b00));

    let mut fdcan_handle = core::mem::zeroed::<stm32_hal::FDCAN_HandleTypeDef>();
    fdcan_handle.Instance = stm32_hal::FDCAN1_BASE as *mut stm32_hal::FDCAN_GlobalTypeDef;
    fdcan_handle.Init.ClockDivider = 0x00000001;
    fdcan_handle.Init.FrameFormat = 0x0;
    fdcan_handle.Init.Mode = 0x0;
    fdcan_handle.Init.AutoRetransmission = stm32_hal::FunctionalState::DISABLE;
    fdcan_handle.Init.TransmitPause = stm32_hal::FunctionalState::DISABLE;
    fdcan_handle.Init.ProtocolException = stm32_hal::FunctionalState::DISABLE;
    fdcan_handle.Init.NominalPrescaler = 5;
    fdcan_handle.Init.NominalSyncJumpWidth = 1;
    fdcan_handle.Init.NominalTimeSeg1 = 14;
    fdcan_handle.Init.NominalTimeSeg2 = 2;
    fdcan_handle.Init.DataPrescaler = 5;
    fdcan_handle.Init.DataSyncJumpWidth = 1;
    fdcan_handle.Init.DataTimeSeg1 = 14;
    fdcan_handle.Init.DataTimeSeg2 = 2;
    fdcan_handle.Init.StdFiltersNbr = 0;
    fdcan_handle.Init.ExtFiltersNbr = 0;
    fdcan_handle.Init.TxFifoQueueMode = 0x0;

    // FDCAN clock enable
    rcc.apb1enr1.write(|w| w.fdcanen().set_bit());

    stm32_hal::HAL_FDCAN_Init(&mut fdcan_handle as *mut stm32_hal::FDCAN_HandleTypeDef);
    stm32_hal::HAL_FDCAN_Start(&mut fdcan_handle as *mut stm32_hal::FDCAN_HandleTypeDef);

    InitHandles {
        fdcan: fdcan_handle,
    }
}
