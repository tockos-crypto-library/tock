/// interface for making a DAC conversion

pub trait DacSingle {
    /// Call enable before any conversion
    /// Return true when enabled successfully
    fn enable(&self) -> bool;

    /// Make a conversion request.
    /// If TXREADY bit is low or not enbaled, it will return false
    /// Otherwise return true
    fn set(&self, data: u16) -> bool;
}
