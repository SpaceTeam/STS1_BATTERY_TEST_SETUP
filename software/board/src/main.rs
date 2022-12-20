#![deny(unsafe_code)]
#![no_main]
#![no_std]

#[allow(unused_imports)]
use rtic::app;

use bbqueue::{BBBuffer, Consumer, Producer};
use firmware::msg_types::MsgTypes;
use heapless::String;
use stm32f4xx_hal::block;
use stm32f4xx_hal::serial::Event;
use stm32f4xx_hal::{
    adc::{
        config::{AdcConfig, SampleTime, Sequence},
        Adc,
    },
    gpio::{Output, Pin, PushPull},
    pac,
    prelude::*,
    serial::*,
};
use systick_monotonic::{fugit::Duration, Systick};
use transmission::{
    receive::receive,
    send::{send, setup},
};

mod panic_handler;

#[app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [TIM2 ])]
mod app {
    use super::*;

    static UART_RX_BUFFER: BBBuffer<1024> = BBBuffer::new();
    static UART_TX_BUFFER: BBBuffer<1024> = BBBuffer::new();

    #[shared]
    struct Shared {
        prod_tx: Producer<'static, 1024>,
        cons_rx: Consumer<'static, 1024>,
    }

    #[local]
    struct Local<'_> {
        led: Pin<'A', 5, Output<PushPull>>,
        rx: Rx<pac::USART2>,
        tx: Tx<pac::USART2>,

        prod_rx: Producer<'static, 1024>,
        cons_tx: Consumer<'static, 1024>,
    }

    #[monotonic(binds = SysTick, default = true)]
    type Tonic = Systick<1000>;

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let rcc = ctx.device.RCC.constrain();
        let _clocks = rcc.cfgr.sysclk(48.MHz()).freeze();

        let gpioa = ctx.device.GPIOA.split();
        let led = gpioa.pa5.into_push_pull_output();

        let config = AdcConfig::default();

        let analog = gpioa.pa0.into_analog();
        let mut adc = Adc::adc1(ctx.device.ADC1, true, config);

        adc.configure_channel(&analog, Sequence::One, SampleTime::Cycles_112);
        adc.enable();
        adc.start_conversion();

        // let val = adc.current_sample();
        let val = adc.convert(&analog, SampleTime::Cycles_112);

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

        let (tx, rx) = s.split();
        let (prod_rx, cons_rx) = UART_RX_BUFFER.try_split().unwrap();
        let (mut prod_tx, cons_tx) = UART_TX_BUFFER.try_split().unwrap();

        blink::spawn().ok();
        setup(&mut prod_tx);
        send(&mut prod_tx, MsgTypes::Msg(String::from("Init done"))).unwrap();

        (
            Shared { prod_tx, cons_rx },
            Local {
                led,
                rx,
                tx,
                prod_rx,
                cons_tx,
            },
            init::Monotonics(mono),
        )
    }

    #[task(local = [led, tx, cons_tx], shared =[prod_tx, cons_rx], priority = 4)]
    fn blink(mut ctx: blink::Context) {
        macro_rules! handle_msg {
            ($ctx:expr, $msg:expr) => {
                match $msg {
                    MsgTypes::Ping(number) => {
                        $ctx.shared.prod_tx.lock(|prod_tx| {
                            send(prod_tx, MsgTypes::Ping(number + 1)).unwrap();
                        });
                    }
                    _ => {}
                }
            };
        }

        ctx.local.led.toggle();

        match ctx.local.cons_tx.read() {
            Ok(rgr) => {
                rgr.buf()
                    .iter()
                    .for_each(|&byte| block!(ctx.local.tx.write(byte)).unwrap());

                let len = rgr.len();
                rgr.release(len);
            }
            _ => (),
        };

        ctx.shared.cons_rx.lock(|cons_rx| {
            receive(cons_rx, |val| match val {
                Ok(msg) => {
                    handle_msg!(ctx, msg);
                }
                Err(_) => {
                    ctx.shared.prod_tx.lock(|prod_tx| {
                        send(
                            prod_tx,
                            MsgTypes::Msg(String::from("Board dropped an invalid packet")),
                        )
                        .unwrap();
                    });
                }
            });
        });

        blink::spawn_after(Duration::<u64, 1, 1000>::from_ticks(50)).ok();
    }

    #[task(binds = USART2, local = [rx, prod_rx])]
    fn serial(ctx: serial::Context) {
        match block!(ctx.local.rx.read()) {
            Ok(byte) => {
                if let Ok(mut wgr) = ctx.local.prod_rx.grant_exact(1) {
                    wgr[0] = byte;
                    wgr.commit(1);
                }
            }
            Err(_) => {}
        }
    }
}
