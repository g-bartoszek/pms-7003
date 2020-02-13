use linux_embedded_hal;

use pms_7003::*;
use std::time::Duration;

fn main() {
    let path = std::env::args()
        .skip(1)
        .next()
        .expect("Missing path to device");

    println!("Connecting to: {}", path);

    let device = linux_embedded_hal::Serial::open(std::path::Path::new(&path)).unwrap();
    let mut sensor = Pms7003Sensor::new(device);

    sensor.sleep().unwrap();
    std::thread::sleep(Duration::from_secs(3));
    sensor.wake().unwrap();

    loop {
        match sensor.read() {
            Ok(frame) => println!("{:?}", frame),
            Err(e) => println!("{:?}", e),
        }
    }
}
