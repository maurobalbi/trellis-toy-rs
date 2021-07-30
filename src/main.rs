#![no_std]
#![no_main]

extern crate feather_m4 as hal;
extern crate panic_halt;
extern crate shared_bus;

use hal::entry;
use embedded_hal::blocking::delay::{DelayMs};

mod init;

use init::init;
use neotrellis;
use neotrellis::{Color, Event, MultiEvent};

use core::convert::TryInto;

fn wheel(pos: u8) -> Color {
  if pos < 85 {
    Color::rgb(pos * 3, 255 - pos * 3, 0)
  } else if pos < 170 {
    Color::rgb(255 - pos * 3, 0, pos * 3)
  } else {
    Color::rgb(0, pos * 3, 255 - pos * 3)
  }
}

#[entry]
fn main() -> ! {
  let (i2c, mut delay) = init();

  let bus = shared_bus::BusManagerSimple::new(i2c);

  // let mut neo = neotrellis::NeoTrellis::new(bus.acquire_i2c(), 0x2f, &mut delay).unwrap();

  let mut paint = PaintComponent { steps: [[0; 8]; 8] };

  let mut multi = neotrellis::MultiTrellis {
    trellis: &mut [
      &mut [
        neotrellis::NeoTrellis::new(bus.acquire_i2c(), 0x31, &mut delay).unwrap(),
        neotrellis::NeoTrellis::new(bus.acquire_i2c(), 0x30, &mut delay).unwrap(),
      ],
      &mut [
        neotrellis::NeoTrellis::new(bus.acquire_i2c(), 0x2E, &mut delay).unwrap(),
        neotrellis::NeoTrellis::new(bus.acquire_i2c(), 0x2F, &mut delay).unwrap(),
      ],
    ],
  };

  for y in 0..8 {
    for x in 0..8 {
      multi.set_led_color((x, y), wheel((x + 8 * y) * 4), &mut delay).unwrap();
      multi.show(&mut delay).unwrap();
      delay.delay_ms(45 as u32);
    }
  }

  for y in 0..8 {
    for x in 0..8 {
      multi.set_led_color((x, y), Color::rgb(0, 0, 0), &mut delay).unwrap();
      multi.show(&mut delay).unwrap();
      delay.delay_ms(45 as u32);
    }
  }
  multi.show(&mut delay).unwrap();

  loop {
    let events: &mut [Option<MultiEvent>] = &mut [None; 64];
    multi.read_events(events, &mut delay).unwrap();
    for event in &mut *events {
      match event {
        Some(event) => {
          paint.update(*event);
        }
        _ => (),
      }
    }
    let colors = paint.render();
    for (x, row) in colors.iter().enumerate() {
      for (y, color) in row.iter().enumerate() {
        match color {
          Some(c) => multi
            .set_led_color((x as u8, y as u8), *c, &mut delay)
            .unwrap(),
          None => multi
            .set_led_color(
              (x as u8, y as u8),
              neotrellis::Color::rgb(0, 0, 0),
              &mut delay,
            )
            .unwrap(),
        };
      }
    }
    multi.show(&mut delay).unwrap()
  }
}

struct PaintComponent {
  steps: [[u8; 8]; 8],
}

pub trait Component {
  fn update(&mut self, event: MultiEvent) -> ();
  fn render(&self) -> [[Option<neotrellis::Color>; 8]; 8];
}

impl Component for PaintComponent {
  fn update(&mut self, event: MultiEvent) -> () {
    if let MultiEvent {
      coordinate: (x, y),
      event: Event::Rising,
    } = event
    {
      let xi: usize = x.try_into().unwrap();
      let yi: usize = y.try_into().unwrap();
      let step = &mut self.steps[xi][yi];
      if *step == 3 {
        *step = 0;
      } else {
        *step += 1;
      }
    }
    ()
  }

  fn render(&self) -> [[Option<neotrellis::Color>; 8]; 8] {
    let mut colors: [[Option<neotrellis::Color>; 8]; 8] = [[None; 8]; 8];
    for (x, item) in self.steps.iter().enumerate() {
      for (y, step) in item.iter().enumerate() {
        match step {
          1 => colors[x][y] = Some(neotrellis::Color::rgb(125, 0, 0)),
          2 => colors[x][y] = Some(neotrellis::Color::rgb(0, 125, 0)),
          3 => colors[x][y] = Some(neotrellis::Color::rgb(0, 0, 125)),
          _ => (),
        }
      }
    }

    colors
  }
}
