use embedded_hal::digital::{ErrorType, InputPin, OutputPin};

pub struct DummyPin;

impl ErrorType for DummyPin {
    type Error = core::convert::Infallible;
}

impl OutputPin for DummyPin {
    #[inline(always)]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline(always)]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
