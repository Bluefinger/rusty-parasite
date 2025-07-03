#![no_std]

use heapless::Vec;
use para_fmt::assert;

const BR_EDR_NOT_SUPPORTED: u8 = 4;
const LE_GENERAL_DISCOVERABLE: u8 = 2;

const BTHOME_AD_HEADER: [u8; 8] = [
    0x02,
    0x01,
    LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED,
    0x04,
    0x16,
    0xD2,
    0xFC,
    0x40,
];

pub const BTHOME_UUID16: u16 = 0xFCD2;

macro_rules! impl_fields {
    { $(($name:ident, $id:literal, $internal_repr:ty, $external_repr:ty),)+ } => {
        $(
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

            impl From<$name> for BtHomeEnum {
                fn from(value: $name) -> Self {
                    Self::$name(value)
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
        )*

        #[derive(Debug, Clone)]
        #[cfg_attr(feature = "defmt", derive(::defmt::Format))]
        pub enum BtHomeEnum {
            $(
                $name($name),
            )*
        }

        impl PartialEq for BtHomeEnum {
            fn eq(&self, other: &Self) -> bool {
                self.id() == other.id()
            }
        }

        impl Eq for BtHomeEnum {}

        impl PartialOrd for BtHomeEnum {
            fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for BtHomeEnum {
            fn cmp(&self, other: &Self) -> core::cmp::Ordering {
                self.id().cmp(&other.id())
            }
        }

        impl BtHomeEnum {
            pub const fn id(&self) -> u8 {
                match self {
                    $(
                        Self::$name(_) => $id,
                    )*
                }
            }

            pub fn encode(&self) -> &[u8] {
                match self {
                    $(
                        Self::$name(repr) => &repr.0,
                    )*
                }
            }
        }
    }
}

impl_fields! {
    (Battery1Per, 0x01, [u8; 2], u8),
    (Temperature10mK, 0x02, [u8; 3], i16),
    (Humidity10mPer, 0x03, [u8; 3], u16),
    (Illuminance10mLux, 0x05, [u8; 4], u32),
    (Moisture10mPer, 0x14, [u8; 3], u16),
}

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
        assert!(N >= BTHOME_AD_HEADER.len(), "Ad buffer is too small");

        let buffer = Vec::from_iter(BTHOME_AD_HEADER);

        Self { buffer }
    }

    pub fn add_data(&mut self, payload: impl Into<BtHomeEnum>) -> &mut Self {
        let payload = payload.into();
        let encoded = payload.encode();

        assert!(
            self.buffer.len() + encoded.len() < N,
            "Can't fit data into buffer! {}+{}",
            self.buffer.len(),
            encoded.len()
        );

        self.buffer[3] += encoded.len() as u8;
        self.buffer.extend_from_slice(encoded).ok();

        self
    }

    fn add_local_name(&mut self, name: &str) -> &mut Self {
        let len = name.len() + 1;

        assert!(
            self.buffer.len() + len < N,
            "Can't fit local name into buffer!"
        );

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

        home.add_data(Battery1Per::from(34))
            .add_data(Temperature10mK::from(2255));

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
                0x01,
                34,
                0x02,
                207,
                8,
            ]
        );
    }

    #[test]
    fn full_payload() {
        let mut home = BtHomeAd::default();

        home.add_data(Battery1Per::from(34))
            .add_data(Temperature10mK::from(2255))
            .add_data(Humidity10mPer::from(3400))
            .add_data(Illuminance10mLux::from(45000))
            .add_data(Moisture10mPer::from(3632));

        let encoded = home.encode_with_local_name("r-para");

        // Final payload is within the max size for the advertising payload
        assert_eq!(encoded.len(), 31);
        assert_eq!(home.buffer[3], 19);
    }
}
