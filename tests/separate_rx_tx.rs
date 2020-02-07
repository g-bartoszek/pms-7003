use pms_7003::Pms7003Sensor;

struct RxMock {}
struct TxMock {}

impl embedded_hal::serial::Read<u8> for RxMock {
    type Error = ();

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        Ok(0)
    }
}

impl embedded_hal::serial::Write<u8> for TxMock {
    type Error = ();

    fn write(&mut self, _: u8) -> nb::Result<(), Self::Error> {
        Ok(())
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        Ok(())
    }
}

#[test]
fn crate_instance_using_separate_rx_tx() {
    let tx = TxMock {};
    let rx = RxMock {};

    let mut pms = Pms7003Sensor::new_tx_rx(tx, rx);
    pms.sleep().unwrap();
}
