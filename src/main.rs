#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

#[macro_use]
extern crate alloc;
extern crate feather_m4 as hal;
extern crate panic_halt;
extern crate shared_bus;


use alloc_cortex_m::CortexMHeap;
use core::alloc::Layout;

use embedded_hal::blocking::delay::DelayMs;
use hal::entry;

mod init;
mod menu;

use menu::{ Component, MenuComponent };

use init::init;
use neotrellis::{Color, MultiEvent, MultiTrellis, NeoTrellis};


#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

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
  let start = cortex_m_rt::heap_start() as usize;
  let size = 2048; // in bytes
  unsafe { ALLOCATOR.init(start, size) }

  let (i2c, mut delay) = init();

  let bus = shared_bus::BusManagerSimple::new(i2c);


  let mut multi = MultiTrellis {
    trellis: &mut [
      &mut [
        NeoTrellis::new(bus.acquire_i2c(), 0x31, &mut delay).unwrap(),
        NeoTrellis::new(bus.acquire_i2c(), 0x30, &mut delay).unwrap(),
      ],
      &mut [
        NeoTrellis::new(bus.acquire_i2c(), 0x2E, &mut delay).unwrap(),
        NeoTrellis::new(bus.acquire_i2c(), 0x2F, &mut delay).unwrap(),
      ],
    ],
  };

  let mut menu = MenuComponent::new();

  for y in 0..8 {
    for x in 0..8 {
      multi
        .set_led_color((x, y), wheel((x + 8 * y) * 4), &mut delay)
        .unwrap();
      multi.show(&mut delay).unwrap();
      delay.delay_ms(45_u32);
    }
  }

  for y in 0..8 {
    for x in 0..8 {
      multi
        .set_led_color((x, y), Color::rgb(0, 0, 0), &mut delay)
        .unwrap();
      multi.show(&mut delay).unwrap();
      delay.delay_ms(45_u32);
    }
  }
  multi.show(&mut delay).unwrap();

  loop {
    let events: &mut [Option<MultiEvent>] = &mut [None; 64];
    multi.read_events(events, &mut delay).unwrap();
    for event in &mut *events {
      match event {
        Some(event) => {
          menu.update(*event);
        }
        _ => (),
      }
    }
    let colors = menu.render();
    for (x, row) in colors.iter().enumerate() {
      for (y, color) in row.iter().enumerate() {
        match color {
          Some(c) => multi
            .set_led_color((x as u8, y as u8), *c, &mut delay)
            .unwrap(),
          None => multi
            .set_led_color((x as u8, y as u8), Color::rgb(0, 0, 0), &mut delay)
            .unwrap(),
        };
      }
    }
    multi.show(&mut delay).unwrap()
  }
}

#[alloc_error_handler]
fn oom(_: Layout) -> ! {
  loop {}
}
