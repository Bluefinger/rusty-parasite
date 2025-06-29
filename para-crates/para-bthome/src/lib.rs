#![no_std]

use heapless::Vec;
use para_fmt::panic;

const BR_EDR_NOT_SUPPORTED: u8 = 4;
const LE_GENERAL_DISCOVERABLE: u8 = 2;

pub const BTHOME_UUID16: u16 = 0xFCD2;

macro_rules! impl_field {
    ($name:ident, $id:literal, $internal_repr:ty, $external_repr:ty) => {
        #[derive(Debug, Clone)]
        #[cfg_attr(feature = "defmt", derive(::defmt::Format))]
        pub struct $name($internal_repr);

        impl $name {
            const ID: u8 = $id;
            const SIZE: usize = core::mem::size_of::<$internal_repr>() - 1;

            #[inline]
            pub fn get(&self) -> $external_repr {
                let mut bytes = [0u8; core::mem::size_of::<$external_repr>()];
                bytes[0..Self::SIZE].copy_from_slice(&self.0[1..]);
                <$external_repr>::from_le_bytes(bytes)
            }
        }

        impl BtHomeData for $name {
            fn encode(&self) -> &[u8] {
                &self.0
            }
        }

        impl From<$external_repr> for $name {
            #[inline]
            fn from(value: $external_repr) -> Self {
                let mut bytes = [0u8; core::mem::size_of::<$internal_repr>()];
                bytes[0] = Self::ID;
                bytes[1..].copy_from_slice(&value.to_le_bytes()[0..Self::SIZE]);
                $name(bytes)
            }
        }
    };
}

impl_field!(Battery1Per, 0x01, [u8; 2], u8);
impl_field!(Moisture10mPer, 0x14, [u8; 3], u16);
impl_field!(Humidity10mPer, 0x03, [u8; 3], u16);
impl_field!(Illuminance10mLux, 0x05, [u8; 4], u32);
impl_field!(Temperature10mK, 0x02, [u8; 3], i16);

pub trait BtHomeData {
    fn encode(&self) -> &[u8];
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BtHomeAd<const N: usize> {
    buffer: Vec<u8, N>,
}

impl<const N: usize> BtHomeAd<N> {
    pub fn new() -> Self {
        let mut buffer = Vec::from_iter([
            0x02,
            0x01,
            LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED,
            0x04,
            0x16,
        ]);

        buffer.extend(BTHOME_UUID16.to_le_bytes());
        buffer.extend([0x40]);

        Self { buffer }
    }

    pub fn add_data(&mut self, payload: &dyn BtHomeData) -> &mut Self {
        let encoded = payload.encode();

        if (self.buffer.len() + encoded.len()) >= N {
            panic!("Can't fit data into buffer!");
        }

        self.buffer[3] += encoded.len() as u8;
        self.buffer.extend_from_slice(encoded).ok();

        self
    }

    fn add_local_name(&mut self, name: &str) -> &mut Self {
        let len = name.len() + 1;

        if (self.buffer.len() + len) >= N {
            panic!("Can't fit local name into buffer!");
        }

        self.buffer.extend_from_slice(&[len as u8, 0x09]).ok();
        self.buffer.extend_from_slice(name.as_bytes()).ok();
        self
    }

    fn encode(&self) -> &[u8] {
        &self.buffer
    }

    pub fn encode_with_local_name(&mut self, name: &str) -> &[u8] {
        self.add_local_name(name).encode()
    }
}

impl Default for BtHomeAd<31> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_add_name() {
        let mut home = BtHomeAd::default();

        let name = "hello";

        home.add_local_name(name);

        assert_eq!(home.buffer.len(), 15);

        assert_eq!(
            home.encode(),
            &[
                0x02,
                0x01,
                LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED,
                0x04,
                0x16,
                0xD2,
                0xFC,
                0x40,
                (name.len() + 1) as u8,
                0x09,
                b"h"[0],
                b"e"[0],
                b"l"[0],
                b"l"[0],
                b"o"[0]
            ]
        );
    }

    #[test]
    fn add_data() {
        let mut home = BtHomeAd::default();

        home.add_data(&Temperature10mK::from(2255))
            .add_data(&Battery1Per::from(34));

        assert_eq!(
            home.encode(),
            &[
                0x02,
                0x01,
                LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED,
                0x09,
                0x16,
                0xD2,
                0xFC,
                0x40,
                0x02,
                207,
                8,
                0x01,
                34
            ]
        );
    }

    #[test]
    fn full_payload() {
        let mut home = BtHomeAd::default();

        home.add_data(&Temperature10mK::from(2255))
            .add_data(&Humidity10mPer::from(3400))
            .add_data(&Illuminance10mLux::from(45000))
            .add_data(&Moisture10mPer::from(3632))
            .add_data(&Battery1Per::from(34));

        let encoded = home.encode_with_local_name("r-para");

        // Final payload is within the max size for the advertising payload
        assert_eq!(encoded.len(), 31);
        assert_eq!(home.buffer[3], 19);
    }
}
