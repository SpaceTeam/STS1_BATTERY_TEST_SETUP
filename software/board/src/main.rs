#![deny(unsafe_code)]
#![no_main]
#![no_std]

#[allow(unused_imports)]
use rtic::app;
use stm32f4xx_hal::gpio::Alternate;
use stm32f4xx_hal::timer::PwmChannel;

use crate::interfaces::*;
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
    gpio::{Analog, Output, Pin, PushPull},
    pac,
    prelude::*,
    rtc::{Lse, Lsi, Rtc},
    serial::*,
    timer,
};
use systick_monotonic::{fugit::Duration, Systick};
use time::PrimitiveDateTime;
use transmission::{
    receive::receive,
    send::{send, setup},
};
// use firmware::

mod interfaces;
mod panic_handler;

type Firmware = firmware::Firmware<
    interfaces::SerialReceiver,
    interfaces::SerialTransmitter,
    interfaces::GpioOutput<'A', 5, Output<PushPull>>,
    interfaces::AdcInput<'A', 0, Analog>,
    interfaces::PwmOutput<PwmChannel<pac::TIM1, 1>>,
>;
type BatteryTestUnit = firmware::BatteryTestUnit<
    interfaces::AdcInput<'A', 0, Analog>,
    interfaces::PwmOutput<PwmChannel<pac::TIM1, 1>>,
>;
#[app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [TIM2 ])]
mod app {
    use firmware::{
        traits::{PwmOutput, SerialReceiver},
        BatteryTestUnitMode,
    };
    use heapless::pool::Box;
    use stm32f4xx_hal::{
        gpio::Analog,
        pac::TIM1,
        timer::{Channel, Pwm, PwmChannel},
    };

    use crate::interfaces::SerialTransmitter;

    use super::*;

    static UART_RX_BUFFER: BBBuffer<1024> = BBBuffer::new();
    static UART_TX_BUFFER: BBBuffer<1024> = BBBuffer::new();

    #[shared]
    struct Shared {
        prod_tx: Producer<'static, 1024>,
        cons_rx: Consumer<'static, 1024>,
        // adc: Adc<pac::ADC1>,
        rtc: Rtc<Lsi>,
        fm: Firmware,
    }

    #[local]
    struct Local<'_> {
        rx: Rx<pac::USART2>,
        tx: Tx<pac::USART2>,

        prod_rx: Producer<'static, 1024>,
        cons_tx: Consumer<'static, 1024>,
    }

    #[monotonic(binds = SysTick, default = true)]
    type Tonic = Systick<1000>;

    #[init]
    fn init(mut ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let rcc = ctx.device.RCC.constrain();
        let _clocks = rcc.cfgr.sysclk(48.MHz()).freeze();

        let gpioa = ctx.device.GPIOA.split();
        let led = gpioa.pa5.into_push_pull_output();

        // let test = GpioOutput {
        //     pin: gpioa.pa5.into_push_pull_output(),
        // };

        // let led = Pin::<'A', 5, Output<PushPull>>::new();

        let config = AdcConfig::default();

        let analog = gpioa.pa0.into_analog();
        let mut adc = Adc::adc1(ctx.device.ADC1, true, config);

        adc.configure_channel(&analog, Sequence::One, SampleTime::Cycles_112);
        adc.enable();
        adc.start_conversion();

        let a = AdcInput::new(analog, adc);

        let mut pwm_pin = gpioa.pa9.into_alternate();
        let mut pwm = ctx.device.TIM1.pwm_hz(pwm_pin, 50.kHz(), &_clocks).split();

        let max_duty = pwm.get_max_duty();
        pwm.enable();
        // pwm.set_duty(390);

        let mut p = interfaces::PwmOutput::<PwmChannel<TIM1, 1>> { pwm };

        p.set_duty_cycle(550);

        // let val = adc.current_sample();
        // let val = adc.convert(&analog, SampleTime::Cycles_112);

        let mut mono = Systick::new(ctx.core.SYST, 48_000_000);

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

        let mut rtc = Rtc::lsi_with_config(ctx.device.RTC, &mut ctx.device.PWR, 249, 127);
        rtc.set_datetime(&time::PrimitiveDateTime::new(
            time::Date::from_calendar_date(2000, time::Month::January, 1).unwrap(),
            time::Time::from_hms(0, 0, 0).unwrap(),
        ));

        // btu.set_mode(BatteryTestUnitMode::Discharging(1.3));

        let mut fm = Firmware {
            serial_receiver: interfaces::SerialReceiver {},
            serial_transmitter: SerialTransmitter {},
            on_board_led: GpioOutput::new(led),
            btu1: BatteryTestUnit::new(a, p),
        };

        fm.btu1.set_mode(BatteryTestUnitMode::Discharging(1.3));

        blink::spawn().ok();
        update_btu::spawn().ok();

        setup(&mut prod_tx);
        send(&mut prod_tx, MsgTypes::Msg(String::from("Init done"))).unwrap();
        send(&mut prod_tx, MsgTypes::SampleAdcResult(max_duty)).unwrap();
        // send(&mut prod_tx, MsgTypes::SampleAdcResult(1234)).unwrap();
        // send(&mut prod_tx, MsgTypes::SampleAdcResult(t.millisecond())).unwrap();

        // cortex_m::asm::delay(100_000_000);
        // send(&mut prod_tx, MsgTypes::SampleAdcResult(t.second() as u16)).unwrap();

        // cortex_m::peripheral::syst.enable_counter();
        // ctx.device.

        // let t = mono.now().checked_duration_since(Systick::zero()).unwrap();
        // send(
        //     &mut prod_tx,
        //     MsgTypes::SampleAdcResult(t.to_millis() as u32),
        // )
        // .unwrap();

        (
            Shared {
                prod_tx,
                cons_rx,
                // adc,
                rtc,
                fm,
            },
            Local {
                rx,
                tx,
                prod_rx,
                cons_tx,
            },
            init::Monotonics(mono),
        )
    }

    #[task(shared = [ rtc, prod_tx, fm ], priority = 4)]
    fn update_btu(mut ctx: update_btu::Context) {
        let t = ctx.shared.rtc.lock(|rtc| rtc.get_datetime());
        let time = t.second() as f32 / 2.0;

        ctx.shared.fm.lock(|fm| {
            fm.update_battery_units(time, 0.0);
        });

        // ctx.shared.btu.lock(|btu| {
        //     btu.update(time, 0.0);
        // });

        ctx.shared.prod_tx.lock(|prod_tx| {
            send(prod_tx, MsgTypes::SampleAdcResult(time as u16)).unwrap();
        });

        update_btu::spawn_after(Duration::<u64, 1, 1000>::from_ticks(100)).ok();
    }

    #[task(local = [tx, cons_tx], shared =[fm, prod_tx, cons_rx,  rtc], priority = 4)]
    fn blink(mut ctx: blink::Context) {
        macro_rules! handle_msg {
            ($ctx:expr, $msg:expr) => {
                match $msg {
                    MsgTypes::Ping(number) => {
                        $ctx.shared.prod_tx.lock(|prod_tx| {
                            send(prod_tx, MsgTypes::Ping(number + 1)).unwrap();
                        });
                    }
                    // MsgTypes::SampleAdc(channel) => {
                    // $ctx.shared.prod_tx.lock(|prod_tx| {
                    // $ctx.shared.adc.lock(|adc| {
                    //     let val = adc.convert(ctx.local.adc1, SampleTime::Cycles_112);
                    //     send(prod_tx, MsgTypes::SampleAdcResult(val)).unwrap();
                    // });
                    // });
                    // }
                    _ => {}
                }
            };
        }

        // let t = monotonics::now()
        //     .checked_duration_since(Systick::zero())
        //     .unwrap();
        // ctx.shared.prod_tx.lock(|prod_tx| {
        //     send(prod_tx, MsgTypes::SampleAdcResult(t.to_millis() as u32)).unwrap();
        // });

        ctx.shared.fm.lock(|fm| {
            fm.toggle_on_board_led();
        });

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
