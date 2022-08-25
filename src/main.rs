#![no_std]
#![no_main]

use esp32_hal::{clock::ClockControl, pac::Peripherals, prelude::*, timer::TimerGroup, RtcCntl};
use esp_backtrace as _;
use esp_println::println;
use xtensa_lx_rt::entry;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let system = peripherals.DPORT.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Disable the RTC and TIMG watchdog timers
    let mut rtc_cntl = RtcCntl::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    let mut wdt1 = timer_group1.wdt;

    rtc_cntl.set_wdt_global_enable(false);
    wdt0.disable();
    wdt1.disable();

    println!("Howdy!");

    let mut bmp180 = Bmp180::new();
    bmp180.measure();
    println!("Current temperature {}", bmp180.get_temperature());

    loop {}
}

pub struct Bmp180 {
    ac1: i32,
    ac2: i32,
    ac3: i32,
    ac4: i32,
    ac5: i32,
    ac6: i32,
    b1: i32,
    b2: i32,
    mb: i32,
    mc: i32,
    md: i32,

    temp: f32,
}

impl Bmp180 {
    pub fn new() -> Bmp180 {
        Bmp180 {
            ac1: 7408,
            ac2: -1157,
            ac3: -14690,
            ac4: -31198,
            ac5: 25186,
            ac6: 18982,
            b1: 6515,
            b2: 45,
            mb: -32768,
            mc: -11786,
            md: 2733,
            temp: 0.0,
        }
    }

    pub fn measure(&mut self) {
        // Read 2 bytes of data from address 0xF6(246)
        // temp msb, temp lsb
        let mut data = [0u8; 2];
        data[0] = 108;
        data[1] = 162;

        // Convert the data
        let temp = (data[0] as u32) << 8 | data[1] as u32;

        // Callibration for Temperature
        let x1: f64 = (temp as f64 - self.ac6 as f64) * self.ac5 as f64 / 32768.0;
        let x2: f64 = (self.mc as f64 * 2048.0) / (x1 + self.md as f64);
        let b5: f64 = x1 + x2;
        let c_temp: f64 = ((b5 + 8.0) / 16.0) / 10.0;

        self.temp = c_temp as f32;
    }

    pub fn get_temperature(&self) -> f32 {
        self.temp
    }
}
