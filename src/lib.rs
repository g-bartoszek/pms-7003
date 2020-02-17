#![no_std]

use embedded_hal::serial::{Read, Write};
use nb::block;
use scroll::{Pread, Pwrite, BE};

mod read_fsm;

const CMD_FRAME_SIZE: usize = 7;
const OUTPUT_FRAME_SIZE: usize = 32;
const RESPONSE_FRAME_SIZE: usize = 8;
const CHECKSUM_SIZE: usize = 2;

type Response = [u8; RESPONSE_FRAME_SIZE];

const PASSIVE_MODE_RESPONSE: Response = [0x42, 0x4D, 0x00, 0x04, 0xE1, 0x00, 0x01, 0x74];
const ACTIVE_MODE_RESPONSE: Response = [0x42, 0x4D, 0x00, 0x04, 0xE1, 0x01, 0x01, 0x75];
const SLEEP_RESPONSE: Response = [0x42, 0x4D, 0x00, 0x04, 0xE4, 0x00, 0x01, 0x77];

#[derive(Debug)]
pub enum Error {
    SendFailed,
    ReadFailed,
    ChecksumError,
    IncorrectResponse,
    NoResponse,
}

/// Sensor interface
pub struct Pms7003Sensor<Serial>
where
    Serial: Read<u8> + Write<u8>,
{
    serial: Serial,
}

impl<Serial> Pms7003Sensor<Serial>
where
    Serial: Read<u8> + Write<u8>,
{
    /// Creates a new sensor instance
    /// * `serial` - single object implementing embedded hal serial traits
    pub fn new(mut serial: Serial) -> Self {
        loop {
            if serial.read().is_err() {
                break;
            }
        }

        Self { serial }
    }

    fn read_from_device(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        use read_fsm::*;

        let mut read = ReadStateMachine::new(buffer);
        loop {
            match read.update(self.serial.read()) {
                ReadStatus::Failed => return Err(Error::ReadFailed),
                ReadStatus::Finished => return Ok(()),
                ReadStatus::InProgress => {}
            }
        }

    }

    /// Reads sensor status. Blocks until status is available.
    pub fn read(&mut self) -> Result<OutputFrame, Error> {
        let mut buffer = [0_u8; OUTPUT_FRAME_SIZE];
        self.read_from_device(&mut buffer)?;
        OutputFrame::from_buffer(&buffer)
    }

    pub fn sleep(&mut self) -> Result<(), Error> {
        self.send_cmd(&create_command(0xe4, 0))?;
        self.receive_response(SLEEP_RESPONSE)
    }

    pub fn wake(&mut self) -> Result<(), Error> {
        self.send_cmd(&create_command(0xe4, 1))
    }

    /// Passive mode - sensor reports air quality on request
    pub fn passive(&mut self) -> Result<(), Error> {
        self.send_cmd(&create_command(0xe1, 0))?;
        self.receive_response(PASSIVE_MODE_RESPONSE)
    }

    /// Active mode - sensor reports air quality continuously
    pub fn active(&mut self) -> Result<(), Error> {
        self.send_cmd(&create_command(0xe1, 1))?;
        self.receive_response(ACTIVE_MODE_RESPONSE)
    }

    /// Requests status in passive mode
    pub fn request(&mut self) -> Result<(), Error> {
        self.send_cmd(&create_command(0xe2, 0))
    }

    fn send_cmd(&mut self, cmd: &[u8]) -> Result<(), Error> {
        for byte in cmd {
            block!(self.serial.write(*byte)).map_err(|_| Error::SendFailed)?;
        }
        Ok(())
    }

    fn receive_response(&mut self, expected_response: Response) -> Result<(), Error> {
        let mut resp = [0u8; RESPONSE_FRAME_SIZE];
        self.read_from_device(&mut resp)?;

        if resp != expected_response {
            Err(Error::IncorrectResponse)
        } else {
            Ok(())
        }
    }
}

fn create_command(cmd: u8, data: u16) -> [u8; CMD_FRAME_SIZE] {
    let mut buffer = [0_u8; CMD_FRAME_SIZE];
    let mut offset = 0usize;

    buffer.gwrite::<u8>(0x42, &mut offset).unwrap();
    buffer.gwrite::<u8>(0x4d, &mut offset).unwrap();
    buffer.gwrite::<u8>(cmd, &mut offset).unwrap();
    buffer.gwrite_with::<u16>(data, &mut offset, BE).unwrap();

    let checksum = buffer
        .iter()
        .take(CMD_FRAME_SIZE - CHECKSUM_SIZE)
        .map(|b| *b as u16)
        .sum::<u16>();
    buffer
        .gwrite_with::<u16>(checksum, &mut offset, BE)
        .unwrap();

    buffer
}

/// Contains data reported by the sensor
#[derive(Default, Debug)]
pub struct OutputFrame {
    pub start1: u8,
    pub start2: u8,
    pub frame_length: u16,
    pub pm1_0: u16,
    pub pm2_5: u16,
    pub pm10: u16,
    pub pm1_0_atm: u16,
    pub pm2_5_atm: u16,
    pub pm10_atm: u16,
    pub beyond_0_3: u16,
    pub beyond_0_5: u16,
    pub beyond_1_0: u16,
    pub beyond_2_5: u16,
    pub beyond_5_0: u16,
    pub beyond_10_0: u16,
    pub reserved: u16,
    pub check: u16,
}

impl OutputFrame {
    pub fn from_buffer(buffer: &[u8; OUTPUT_FRAME_SIZE]) -> Result<Self, Error> {
        let sum: usize = buffer
            .iter()
            .take(OUTPUT_FRAME_SIZE - CHECKSUM_SIZE)
            .map(|e| *e as usize)
            .sum();

        let mut frame = OutputFrame::default();
        let mut offset = 0usize;

        frame.start1 = buffer.gread::<u8>(&mut offset).unwrap();
        frame.start2 = buffer.gread::<u8>(&mut offset).unwrap();
        frame.frame_length = buffer.gread_with::<u16>(&mut offset, BE).unwrap();
        frame.pm1_0 = buffer.gread_with::<u16>(&mut offset, BE).unwrap();
        frame.pm2_5 = buffer.gread_with::<u16>(&mut offset, BE).unwrap();
        frame.pm10 = buffer.gread_with::<u16>(&mut offset, BE).unwrap();
        frame.pm1_0_atm = buffer.gread_with::<u16>(&mut offset, BE).unwrap();
        frame.pm2_5_atm = buffer.gread_with::<u16>(&mut offset, BE).unwrap();
        frame.pm10_atm = buffer.gread_with::<u16>(&mut offset, BE).unwrap();
        frame.beyond_0_3 = buffer.gread_with::<u16>(&mut offset, BE).unwrap();
        frame.beyond_0_5 = buffer.gread_with::<u16>(&mut offset, BE).unwrap();
        frame.beyond_1_0 = buffer.gread_with::<u16>(&mut offset, BE).unwrap();
        frame.beyond_2_5 = buffer.gread_with::<u16>(&mut offset, BE).unwrap();
        frame.beyond_5_0 = buffer.gread_with::<u16>(&mut offset, BE).unwrap();
        frame.beyond_10_0 = buffer.gread_with::<u16>(&mut offset, BE).unwrap();
        frame.reserved = buffer.gread_with::<u16>(&mut offset, BE).unwrap();
        frame.check = buffer.gread_with::<u16>(&mut offset, BE).unwrap();

        if sum != frame.check as usize {
            return Err(Error::ChecksumError);
        }

        Ok(frame)
    }
}

impl<TX, RX> Pms7003Sensor<Wrapper<TX, RX>>
where
    TX: Write<u8>,
    RX: Read<u8>,
{
    /// Creates a new sensor instance
    /// * `tx` - embedded hal serial Write
    /// * `rx` - embedded hal serial Read
    pub fn new_tx_rx(tx: TX, rx: RX) -> Self {
        Self::new(Wrapper(tx, rx))
    }
}

/// Combines two serial traits objects into one
pub struct Wrapper<TX, RX>(TX, RX)
where
    TX: Write<u8>,
    RX: Read<u8>;

impl<TX, RX> Read<u8> for Wrapper<TX, RX>
where
    TX: Write<u8>,
    RX: Read<u8>,
{
    type Error = RX::Error;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        self.1.read()
    }
}

impl<TX, RX> Write<u8> for Wrapper<TX, RX>
where
    TX: Write<u8>,
    RX: Read<u8>,
{
    type Error = TX::Error;

    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        self.0.write(word)
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        self.0.flush()
    }
}
