use trellis_m4 as bsp;
use ws2812_timer_delay as ws2812;

use bsp::{
    gpio::{v2::PA27, Output, Pin, PushPull},
    hal::{
        clock::GenericClockController,
        delay::Delay,
        ehal::digital::v1_compat::OldOutputPin,
        pac::{CorePeripherals, Peripherals},
        timer::SpinTimer,
    },
    Keypad,
};

pub type Neopixel = ws2812::Ws2812<SpinTimer, OldOutputPin<Pin<PA27, Output<PushPull>>>>;

pub struct Board {
    // pub peripherals: Peripherals,
    // pub core: CorePeripherals,
    // pub pins: Sets,
    pub clocks: GenericClockController,
    pub delay: Delay,
    pub neopixel: Neopixel,
    pub keypad: Keypad,
}

impl Board {
    pub fn new() -> Self {
        let mut peripherals = Peripherals::take().unwrap();
        let mut core = CorePeripherals::take().unwrap();

        let mut clocks = GenericClockController::with_internal_32kosc(
            peripherals.GCLK,
            &mut peripherals.MCLK,
            &mut peripherals.OSC32KCTRL,
            &mut peripherals.OSCCTRL,
            &mut peripherals.NVMCTRL,
        );

        let delay = Delay::new(core.SYST, &mut clocks);

        let mut pins = bsp::Pins::new(peripherals.PORT).split();

        // neo pixel
        let timer = SpinTimer::new(4);
        let neopixel_pin: OldOutputPin<_> =
            pins.neopixel.into_push_pull_output(&mut pins.port).into();
        let neopixel: Neopixel = ws2812::Ws2812::new(timer, neopixel_pin);

        let keypad = bsp::Keypad::new(pins.keypad, &mut pins.port);

        crate::usb::setup_usb(
            &mut peripherals.MCLK,
            peripherals.USB,
            &mut core.NVIC,
            pins.usb,
            &mut clocks,
        );

        Self {
            clocks,
            delay,
            neopixel,
            keypad,
        }
    }
}
