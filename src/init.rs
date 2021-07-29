use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::i2c_master;
use hal::pac::{CorePeripherals, Peripherals};
use hal::time::Hertz;

use feather_m4::sercom::I2CMaster2;

pub fn init<'a>() -> (
    I2CMaster2<
        hal::sercom::Sercom2Pad0<hal::gpio::Pa12<hal::gpio::PfC>>,
        hal::sercom::Sercom2Pad1<hal::gpio::Pa13<hal::gpio::PfC>>,
    >,
    Delay,
) {
    let mut peripherals = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.MCLK,
        &mut peripherals.OSC32KCTRL,
        &mut peripherals.OSCCTRL,
        &mut peripherals.NVMCTRL,
    );
    let mut pins = hal::Pins::new(peripherals.PORT);
    let delay = Delay::new(core.SYST, &mut clocks);

    let sda = pins.sda.into_floating_input(&mut pins.port);
    let scl = pins.scl.into_floating_input(&mut pins.port);

    let i2c = i2c_master(
        &mut clocks,
        Hertz(100_000),
        peripherals.SERCOM2,
        &mut peripherals.MCLK,
        sda,
        scl,
        &mut pins.port,
    );

    (i2c, delay)
}
