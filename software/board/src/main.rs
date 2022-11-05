#![deny(unsafe_code)]
#![no_main]
#![no_std]
#![allow(unused_imports)]

use panic_rtt_target as _panic_handler;
use rtic::app;

#[app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [TIM2 ])]
mod app {
    use bbqueue::{BBBuffer, Consumer, Producer};
    use core::fmt::Write;
    use core::sync::atomic::{AtomicUsize, Ordering};
    use rtt_target::{rprintln, rtt_init_print};
    use stm32f4xx_hal::block;
    use stm32f4xx_hal::dma::traits::SafePeripheralRead;
    use stm32f4xx_hal::serial::Event;
    use stm32f4xx_hal::{
        gpio::{gpioa::PA0, gpioc::PC6, Alternate, Edge, Input, Output, Pin, PushPull},
        hal, pac,
        prelude::*,
        serial::*,
    };
    use systick_monotonic::{fugit::Duration, Systick};

    static UART_BUFFER: BBBuffer<1024> = BBBuffer::new();

    #[shared]
    struct Shared {
        prod: Producer<'static, 1024>,
        cons: Consumer<'static, 1024>,
    }

    #[local]
    struct Local<'_> {
        led: Pin<'A', 5, Output<PushPull>>,
        rx: Rx<pac::USART2>,
        tx: Tx<pac::USART2>,
    }

    #[monotonic(binds = SysTick, default = true)]
    type Tonic = Systick<1000>;

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        rtt_init_print!();

        let rcc = ctx.device.RCC.constrain();
        let _clocks = rcc.cfgr.sysclk(48.MHz()).freeze();

        let gpioa = ctx.device.GPIOA.split();
        let led = gpioa.pa5.into_push_pull_output();
        // let mut sys_cfg = ctx.device.SYSCFG.constrain();

        let mono = Systick::new(ctx.core.SYST, 48_000_000);

        let mut s = Serial::new(
            ctx.device.USART2,
            (gpioa.pa2, gpioa.pa3),
            Config::default()
                .baudrate(115200.bps())
                .wordlength_8()
                .parity_none(),
            &_clocks,
        )
        .unwrap();

        s.listen(Event::Rxne);

        let (mut tx, rx) = s.split();
        let (prod, cons) = UART_BUFFER.try_split().unwrap();

        write!(tx, "INIT!\r\n").unwrap();

        blink::spawn().ok();

        (
            Shared { prod, cons },
            Local { led, rx, tx },
            init::Monotonics(mono),
        )
    }

    #[task(local = [led, tx], shared =[cons], priority = 4)]
    fn blink(mut ctx: blink::Context) {
        ctx.local.led.toggle();

        write!(ctx.local.tx, "Blink\r\n").unwrap();

        ctx.shared.cons.lock(|cons| {
            let rgr = match cons.read() {
                Ok(it) => it,
                _ => return,
            };
            rgr.buf()
                .iter()
                .for_each(|&byte| block!(ctx.local.tx.write(byte)).unwrap());

            let len = rgr.len();
            rgr.release(len);
        });
        blink::spawn_after(Duration::<u64, 1, 1000>::from_ticks(1500)).ok();
    }

    #[task(binds = USART2, local = [rx], shared =[prod])]
    fn serial(mut ctx: serial::Context) {
        match block!(ctx.local.rx.read()) {
            Ok(byte) => {
                ctx.shared.prod.lock(|prod| {
                    if let Ok(mut wgr) = prod.grant_exact(1) {
                        wgr[0] = byte;
                        wgr.commit(1);
                    }
                });
            }
            Err(_) => {}
        }
    }
}
