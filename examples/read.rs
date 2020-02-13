use linux_embedded_hal;

use pms_7003::*;

fn main() {
    let path = std::env::args()
        .skip(1)
        .next()
        .expect("Missing path to device");

    println!("Connecting to: {}", path);

    let device = linux_embedded_hal::Serial::open(std::path::Path::new(&path)).unwrap();
    let mut sensor = Pms7003Sensor::new(device);

    loop {
        match sensor.read() {
            Ok(frame) => println!("{:?}", frame),
            Err(e) => println!("{:?}", e),
        }
    }
}
