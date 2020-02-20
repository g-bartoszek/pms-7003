use pms_7003::Pms7003Sensor;
use std::io::Write;
use std::time::Duration;

#[test]
fn read_messages_from_fake_serial() {
    let mut socat = std::process::Command::new("socat")
        .args(&[
            "-d",
            "-d",
            "pty,raw,echo=0,link=pty1",
            "pty,raw,echo=0,link=pty2",
        ])
        .spawn()
        .expect("Failed to create fake serial. Is socat installed?");

    std::thread::sleep(Duration::from_millis(10));

    let mut pms =
        Pms7003Sensor::new(linux_embedded_hal::Serial::open(std::path::Path::new("pty1")).unwrap());

    let status = [
        0x42, 0x4d, 0x0, 0x1c, 0x0, 0x5, 0x0, 0x7, 0x0, 0x7, 0x0, 0x5, 0x0, 0x7, 0x0, 0x7, 0x0,
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x97, 0x0, 0x1, 0x68,
    ];

    std::fs::write(std::path::Path::new("pty2"), &status).unwrap();
    std::fs::write(std::path::Path::new("pty2"), &status).unwrap();
    std::fs::write(std::path::Path::new("pty2"), &status).unwrap();

    pms.read().unwrap();
    pms.read().unwrap();
    pms.read().unwrap();
    assert!(pms.read().is_err());

    socat.kill().unwrap();
}
