use bbqueue::BBBuffer;
use core::fmt::Write;
use firmware::msg_types::MsgTypes;
use heapless::String;
use stm32f4xx_hal::block;
use stm32f4xx_hal::{pac, prelude::*, serial::*};
use transmission::send::{send, setup};

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    #[allow(unsafe_code)]
    let dp = unsafe { pac::Peripherals::steal() };

    let gpioa = dp.GPIOA.split();
    let mut led = gpioa.pa5.into_push_pull_output();

    let rcc = dp.RCC.constrain();
    let _clocks = rcc.cfgr.sysclk(48.MHz()).freeze();

    // TODO: Move the config to a variable and use it in the init function
    let mut tx: Tx<pac::USART2, u8> = Serial::tx(
        dp.USART2,
        gpioa.pa2,
        Config::default()
            .baudrate(115200.bps())
            .wordlength_8()
            .parity_none(),
        &_clocks,
    )
    .unwrap();

    let mut msg: String<128> = heapless::String::new();
    write!(msg, "{}", info).unwrap();

    let buf: BBBuffer<128> = BBBuffer::new();
    let (mut prod, mut cons) = buf.try_split().unwrap();

    setup(&mut prod);
    send(&mut prod, MsgTypes::Msg(msg)).unwrap();
    send(&mut prod, MsgTypes::Ping(128)).unwrap();

    cons.read().unwrap().iter().for_each(|&byte| {
        block!(tx.write(byte)).unwrap();
    });

    loop {
        led.set_low();
        cortex_m::asm::delay(4_000_000);
        led.set_high();
        cortex_m::asm::delay(12_000_000);
    }
}
