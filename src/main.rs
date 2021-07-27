#![no_std]
#![no_main]

extern crate feather_m4 as hal;
extern crate panic_halt;

use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::entry;
use hal::i2c_master;
use hal::pac::{CorePeripherals, Peripherals};
use hal::time::Hertz;

use feather_m4::sercom::I2CMaster2;

use neotrellis;
use neotrellis::{Event, KeypadEvent};

use core::convert::TryInto;

#[entry]
fn main() -> ! {
    let (i2c, delay) = init();

    let mut neo = neotrellis::NeoTrellis::new(i2c, 0x2E, delay).unwrap();

    let mut paint = PaintComponent { steps: [0; 16] };

    let mut events: &mut [Option<KeypadEvent>] = &mut [None; 16];
    loop {
        neo.read_key_events(&mut events).unwrap();
        for event in &mut *events {
            match event {
                Some(event) => {
                    paint.update(*event);
                }
                _ => () 
            }
        }
        let colors = paint.render();
        for (i, color) in colors.iter().enumerate() {
            match color {
                Some(c) => neo.set_led_color(i.try_into().unwrap(), *c).unwrap(),
                None => neo
                    .set_led_color(i.try_into().unwrap(), neotrellis::Color::rgb(0, 0, 0))
                    .unwrap(),
            };
        }
        neo.show_led().unwrap()
    }
}

enum Selector {
    Index(u8),
    Coordinate(u8, u8),
}

impl Selector {
    fn new<A>(args: A) -> Self
    where
        A: Into<Selector>,
    {
        args.into()
    }
}

impl From<u8> for Selector {
    fn from(a: u8) -> Selector {
        Selector::Index(a)
    }
}

impl From<(u8, u8)> for Selector {
    fn from((a, b): (u8, u8)) -> Selector {
        Selector::Coordinate(a, b)
    }
}

struct PaintComponent {
    steps: [u8; 16],
}

pub trait Component {
    fn update(&mut self, arr: KeypadEvent) -> ();
    fn render(&self) -> [Option<neotrellis::Color>; 16];
}

impl Component for PaintComponent {
    fn update(&mut self, event: KeypadEvent) -> () {
      if let KeypadEvent {key, event: Event::Rising} = event {
        let step = &mut self.steps[usize::from(event.key.index())];
        if *step == 3 {
            *step = 0;
        } else {
            *step += 1;
        }
      }
     ()
    }

    fn render(&self) -> [Option<neotrellis::Color>; 16] {
        let mut colors: [Option<neotrellis::Color>; 16] = [None; 16];
        for (i, item) in self.steps.iter().enumerate() {
            match item {
                1 => colors[i] = Some(neotrellis::Color::rgb(125, 0, 0)),
                2 => colors[i] = Some(neotrellis::Color::rgb(0, 125, 0)),
                3 => colors[i] = Some(neotrellis::Color::rgb(0, 0, 125)),
                _ => (),
            }
        }

        colors
    }
}

pub fn init() -> (
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
