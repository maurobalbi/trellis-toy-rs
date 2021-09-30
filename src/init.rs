use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::i2c_master;
use hal::pac::{interrupt, CorePeripherals, Peripherals};
use hal::time::Hertz;
use hal::usb::UsbBus;

use alloc::string::String;

use usb_device::bus::UsbBusAllocator;

use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

use feather_m4::sercom::I2CMaster2;
use cortex_m::peripheral::NVIC;


pub fn init<'a>() -> (
  I2CMaster2<
    hal::sercom::Sercom2Pad0<hal::gpio::Pa12<hal::gpio::PfC>>,
    hal::sercom::Sercom2Pad1<hal::gpio::Pa13<hal::gpio::PfC>>,
  >,
  Delay,
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
  let mut pins = hal::Pins::new(peripherals.PORT);
  let delay = Delay::new(core.SYST, &mut clocks);

  let bus_allocator = unsafe {
    USB_ALLOCATOR = Some(hal::usb_allocator(
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

  (i2c, delay)
}

static mut USB_ALLOCATOR: Option<UsbBusAllocator<UsbBus>> = None;
static mut USB_BUS: Option<UsbDevice<UsbBus>> = None;
static mut USB_SERIAL: Option<SerialPort<UsbBus>> = None;

pub fn debug(mut text: String) {
  text.push('\r');
  text.push('\n');
  unsafe {
    USB_SERIAL.as_mut().map(|serial| serial.write(text.as_bytes())).unwrap();
  }
}

fn poll_usb() {
  unsafe {
    if let Some(usb_dev) = USB_BUS.as_mut() { USB_SERIAL.as_mut().map(|serial| {
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
      }); }
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
