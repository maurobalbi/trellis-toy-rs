use bsp::clock::GenericClockController;
use bsp::delay::Delay;
use bsp::hal::trng::Trng;
use bsp::i2c_master;
use bsp::pac::{interrupt, CorePeripherals, Peripherals};
use bsp::prelude::*;
use bsp::time::Hertz;
use bsp::timer::TimerCounter;
use bsp::usb::UsbBus;

use alloc::string::String;

use usb_device::bus::UsbBusAllocator;

use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

use bsp::pac::TC2;
use bsp::sercom::I2CMaster2;
use cortex_m::peripheral::NVIC;

pub fn init<'a>() -> (
  I2CMaster2<
    bsp::sercom::Sercom2Pad0<bsp::gpio::Pa12<bsp::gpio::PfC>>,
    bsp::sercom::Sercom2Pad1<bsp::gpio::Pa13<bsp::gpio::PfC>>,
  >,
  Delay,
  TimerCounter<TC2>,
  bsp::hal::trng::Trng
) {
  let mut peripherals = Peripherals::take().unwrap();
  let mut core = CorePeripherals::take().unwrap();
  let mut clocks = GenericClockController::with_external_32kosc(
    peripherals.GCLK,
    &mut peripherals.MCLK,
    &mut peripherals.OSC32KCTRL,
    &mut peripherals.OSCCTRL,
    &mut peripherals.NVMCTRL,
  );

  let gclk0 = clocks.gclk0();
  // Configure a clock for TC2 and TC3 peripherals
  let timer_clock = clocks.tc2_tc3(&gclk0).unwrap();
  //Instantiate a timer object for the TC2 peripheral
  let mut timer = TimerCounter::tc2_(&timer_clock, peripherals.TC2, &mut peripherals.MCLK);
  // Start the timer such that it runs at 50Hz
  timer.start(Hertz(33u32));

  let mut pins = bsp::Pins::new(peripherals.PORT);
  let delay = Delay::new(core.SYST, &mut clocks);

  let rng = Trng::new(&mut peripherals.MCLK, peripherals.TRNG);

  let bus_allocator = unsafe {
    USB_ALLOCATOR = Some(bsp::usb_allocator(
      pins.usb_dm,
      pins.usb_dp,
      peripherals.USB,
      &mut clocks,
      &mut peripherals.MCLK,
    ));
    USB_ALLOCATOR.as_ref().unwrap()
  };

  unsafe {
    USB_SERIAL = Some(SerialPort::new(bus_allocator));
    USB_BUS = Some(
      UsbDeviceBuilder::new(bus_allocator, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST")
        .device_class(USB_CLASS_CDC)
        .build(),
    );
  }

  unsafe {
    core.NVIC.set_priority(interrupt::USB_OTHER, 1);
    core.NVIC.set_priority(interrupt::USB_TRCPT0, 1);
    core.NVIC.set_priority(interrupt::USB_TRCPT1, 1);
    NVIC::unmask(interrupt::USB_OTHER);
    NVIC::unmask(interrupt::USB_TRCPT0);
    NVIC::unmask(interrupt::USB_TRCPT1);
  }

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

  (i2c, delay, timer, rng)
}

static mut USB_ALLOCATOR: Option<UsbBusAllocator<UsbBus>> = None;
static mut USB_BUS: Option<UsbDevice<UsbBus>> = None;
static mut USB_SERIAL: Option<SerialPort<UsbBus>> = None;

pub fn debug(mut text: String) {
  text.push('\r');
  text.push('\n');
  unsafe {
    USB_SERIAL
      .as_mut()
      .map(|serial| serial.write(text.as_bytes()))
      .unwrap();
  }
}

fn poll_usb() {
  unsafe {
    if let Some(usb_dev) = USB_BUS.as_mut() {
      USB_SERIAL.as_mut().map(|serial| {
        usb_dev.poll(&mut [serial]);
        let mut buf = [0u8; 64];

        if let Ok(count) = serial.read(&mut buf) {
          for (i, _c) in buf.iter().enumerate() {
            if i >= count {
              break;
            }
            serial.write(b"mauro").unwrap();
          }
        };
      });
    }
  };
}

#[interrupt]
fn USB_OTHER() {
  poll_usb();
}

#[interrupt]
fn USB_TRCPT0() {
  poll_usb();
}

#[interrupt]
fn USB_TRCPT1() {
  poll_usb();
}
