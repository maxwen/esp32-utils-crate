#![no_std]


use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_hal_async::i2c::I2c;

pub const FT6236_DEFAULT_ADDR: u8 = 0x38;

pub mod regs {
    pub const CHIPID: u8 = 0xA3;
    pub const VENDID: u8 = 0xA8;
    pub const FIRMVERS: u8 = 0xA6;
    pub const THRESHHOLD: u8 = 0x80;
    pub const NUMTOUCHES: u8 = 0x02;
    pub const GEST_ID: u8 = 0x01;
}

const CHIPID_FT6206: u8 = 0x06;
const CHIPID_FT6236: u8 = 0x36;
const CHIPID_FT6236U: u8 = 0x64;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum Gesture {
    MoveUp = 0x10,
    MoveRight = 0x14,
    MoveDown = 0x18,
    MoveLeft = 0x1C,
    ZoomIn = 0x48,
    ZoomOut = 0x49,
}

impl Gesture {
    pub fn gesture_id(&self) -> u8 {
        match self {
            Gesture::MoveUp => 0x10,
            Gesture::MoveRight => 0x14,
            Gesture::MoveDown => 0x18,
            Gesture::MoveLeft => 0x1C,
            Gesture::ZoomIn => 0x48,
            Gesture::ZoomOut => 0x49,
        }
    }

    fn from_u8(gesture: u8) -> Option<Self> {
        match gesture {
            0x10 => Some(Gesture::MoveUp),
            0x14 => Some(Gesture::MoveRight),
            0x18 => Some(Gesture::MoveDown),
            0x1C => Some(Gesture::MoveLeft),
            0x48 => Some(Gesture::ZoomIn),
            0x49 => Some(Gesture::ZoomOut),
            // 0x00 => None,
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum EventType {
    PressDown = 0b00,
    LiftUp = 0b01,
    Contact = 0b10,
}

impl EventType {
    fn from_u8(event: u8) -> Option<Self> {
        match event {
            0b00 => Some(EventType::PressDown),
            0b01 => Some(EventType::LiftUp),
            0b10 => Some(EventType::Contact),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PointEvent {
    /// Touch X position
    pub x: u16,
    /// Touch Y position
    pub y: u16,
    /// Touch event flag
    pub event: EventType,
    /// Touch weight, in Px_WEIGHT register
    pub weight: u8,
    /// Touch area, in Px_MISC register
    pub area: u8,
    /// Touch ID, 0 or 1
    pub touch_id: u8,
}

pub struct FT6236<I2C> {
    i2c: I2C,
    addr: u8,
}

impl<I2C> FT6236<I2C>
where
    I2C: I2c,
{
    pub fn new(i2c: I2C) -> Self {
        FT6236 {
            i2c,
            addr: FT6236_DEFAULT_ADDR,
        }
    }

    pub fn new_with_addr(i2c: I2C, addr: u8) -> Self {
        FT6236 { i2c, addr }
    }

    pub async fn init(&mut self, config: Config) -> Result<(), I2C::Error> {
        let chipid = self.read_reg(regs::CHIPID).await?;
        if chipid != CHIPID_FT6206 && chipid != CHIPID_FT6236 && chipid != CHIPID_FT6236U {
            #[cfg(feature = "defmt")]
            defmt::error!("invalid chipid 0x{:02x}", chipid);
        }

        #[cfg(feature = "defmt")]
        {
            let vendid = self.read_reg(regs::VENDID)?;
            let active_report_rate = self.read_reg(0x88)?;
            let monitor_report_rate = self.read_reg(0x89)?;
            let g_mode = self.read_reg(0xa4)?;

            defmt::info!("chipid 0x{:02x}, vendid 0x{:02x}", chipid, vendid);
            defmt::info!("active report rate: 0x{:02x}", active_report_rate);
            defmt::info!("monitor report rate: 0x{:02x}", monitor_report_rate);
            defmt::info!("interrupt mode: 0x{:02x}", g_mode);
        }

        self.write_reg(regs::THRESHHOLD, config.threshhold).await?;

        Ok(())
    }

    pub fn reset<P: OutputPin, D: DelayNs>(
        &mut self,
        rst: &mut P,
        delay: &mut D,
    ) -> Result<(), P::Error> {
        rst.set_high()?;
        delay.delay_ms(5);
        rst.set_low()?;
        delay.delay_ms(10); // min = 5ms
        rst.set_high()?;
        delay.delay_ms(350); // min = 300ms

        Ok(())
    }

    /// Number of touches, 0, 1 or 2
    pub async fn get_number_of_touches(&mut self) -> Result<u8, I2C::Error> {
        let n = self.read_reg(regs::NUMTOUCHES).await?;
        if n & 0b11 <= 2 {
            Ok(n)
        } else {
            Ok(0) // invalid
        }
    }

    /// get first touch point
    pub async fn get_point0(&mut self) -> Result<Option<PointEvent>, I2C::Error> {
        self.get_point(0).await
    }

    /// get second touch point
    pub async fn get_point1(&mut self) -> Result<Option<PointEvent>, I2C::Error> {
        self.get_point(1).await
    }

    #[inline]
    pub async fn get_point(&mut self, nth: u8) -> Result<Option<PointEvent>, I2C::Error> {
        if self.get_number_of_touches().await? <= nth {
            return Ok(None);
        }
        let mut buf = [0u8; 6];
        self.i2c
            .write_read(self.addr, &[0x03 + 6 * nth], &mut buf).await?;

        let event = if let Some(event) = EventType::from_u8(buf[0] >> 6) {
            event
        } else {
            return Ok(None);
        };

        let x = (((buf[0] as u16) & 0b111) << 8) | (buf[1] as u16);

        let touch_id = buf[2] >> 4;
        if touch_id == 0x0f {
            return Ok(None); // invalid touch id
        }
        let y = (((buf[2] as u16) & 0b111) << 8) | (buf[3] as u16);
        let weight = buf[4];
        let area = buf[5] & 0b1111;

        Ok(Some(PointEvent {
            x,
            y,
            event,
            weight,
            area,
            touch_id,
        }))
    }

    /// Get the gesture, this is not available for some touch panels
    pub async fn get_gesture(&mut self) -> Result<Option<Gesture>, I2C::Error> {
        let gesture = self.read_reg(regs::GEST_ID).await?;
        Ok(Gesture::from_u8(gesture))
    }

    async fn read_reg(&mut self, reg_addr: u8) -> Result<u8, I2C::Error> {
        let mut buf = [0u8];
        self.i2c.write_read(self.addr, &[reg_addr], &mut buf).await?;

        Ok(buf[0])
    }

    async fn write_reg(&mut self, reg_addr: u8, value: u8) -> Result<(), I2C::Error> {
        self.i2c.write(self.addr, &[reg_addr, value]).await?;

        Ok(())
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Config {
    /// threshold for touch detection
    pub threshhold: u8,
}

impl Default for Config {
    fn default() -> Self {
        Config { threshhold: 0x40 }
    }
}
