#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use defmt::*;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy::util::Forever;
use embassy_nrf::gpio::NoPin;
use embassy_nrf::pwm::{
    Config, Prescaler, Sequence, SequenceConfig, SequenceLoad, SequenceMode, SequencePwm, Sequences,
};
use embassy_nrf::{peripherals, Peripherals};

// WS2812B LED light demonstration. Drives just one light.
// The following reference on WS2812B may be of use:
// https://cdn-shop.adafruit.com/datasheets/WS2812B.pdf.
// This demo lights up a single LED in blue. It then proceeds
// to pulsate the LED rapidly.

// In the following declarations, setting the high bit tells the PWM
// to reverse polarity, which is what the WS2812B expects.

const T1H: u16 = 0x8000 | 13; // Duty = 13/20 ticks (0.8us/1.25us) for a 1
const T0H: u16 = 0x8000 | 7; // Duty 7/20 ticks (0.4us/1.25us) for a 0
const RES: u16 = 0x8000;

static PWM: Forever<SequencePwm<peripherals::PWM0>> = Forever::new();

// Provides data to a WS2812b (Neopixel) LED and makes it go blue. The data
// line is assumed to be P1_05.
#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    let mut config = Config::default();
    config.sequence_load = SequenceLoad::Common;
    config.prescaler = Prescaler::Div1;
    config.max_duty = 20; // 1.25us (1s / 16Mhz * 20)
    let pwm = unwrap!(SequencePwm::new(
        p.PWM0, p.P1_05, NoPin, NoPin, NoPin, config,
    ));

    let mut pwm = PWM.put(pwm);

    // Declare the bits of 24 bits in a buffer we'll be
    // mutating later.
    let mut seq_words = [
        T0H, T0H, T0H, T0H, T0H, T0H, T0H, T0H, // G
        T0H, T0H, T0H, T0H, T0H, T0H, T0H, T0H, // R
        T1H, T1H, T1H, T1H, T1H, T1H, T1H, T1H, // B
        RES,
    ];
    let mut seq_config = SequenceConfig::default();
    seq_config.end_delay = 799; // 50us (20 ticks * 40) - 1 tick because we've already got one RES;

    let mut color_bit = 16;
    let mut bit_value = T0H;

    loop {
        let sequence0 = Sequence::new(&seq_words, seq_config.clone());
        let sequences = Sequences::new(&mut pwm, sequence0, None);
        unwrap!(sequences.start(SequenceMode::Times(2)));

        Timer::after(Duration::from_millis(50)).await;

        if bit_value == T0H {
            if color_bit == 20 {
                bit_value = T1H;
            } else {
                color_bit += 1;
            }
        } else {
            if color_bit == 16 {
                bit_value = T0H;
            } else {
                color_bit -= 1;
            }
        }

        drop(sequences);

        seq_words[color_bit] = bit_value;
    }
}
