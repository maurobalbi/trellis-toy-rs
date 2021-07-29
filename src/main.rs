#![no_std]
#![no_main]

extern crate feather_m4 as hal;
extern crate panic_halt;
extern crate shared_bus;

use hal::entry;

mod init;

use init::init;
use neotrellis;
use neotrellis::{Event, KeypadEvent};

use core::convert::TryInto;

#[entry]
fn main() -> ! {
    let (i2c, mut delay) = init();

    let bus = shared_bus::BusManagerSimple::new(i2c);

    let mut neo = neotrellis::NeoTrellis::new(bus.acquire_i2c(), 0x2f, &mut delay).unwrap();

    let mut paint = PaintComponent { steps: [0; 16] };

    let mut multi = neotrellis::MultiTrellis {
      trellis: &mut [& mut [neotrellis::NeoTrellis::new(bus.acquire_i2c(), 0x2e, &mut delay).unwrap()]]
    };

    multi.set_led_color(&mut delay).unwrap();
    multi.show(&mut delay).unwrap();

    let mut events: &mut [Option<KeypadEvent>] = &mut [None; 16];
    loop {
        neo.read_key_events(&mut events, &mut delay).unwrap();
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
                Some(c) => neo.set_led_color(i.try_into().unwrap(), *c, &mut delay).unwrap(),
                None => neo
                    .set_led_color(i.try_into().unwrap(), neotrellis::Color::rgb(0, 0, 0), &mut delay)
                    .unwrap(),
            };
        }
        neo.show_led(&mut delay).unwrap()
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
      if let KeypadEvent {key: _, event: Event::Rising} = event {
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
