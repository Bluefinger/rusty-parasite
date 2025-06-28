#![no_std]

use heapless::Vec;

const BR_EDR_NOT_SUPPORTED: u8 = 4;
const LE_GENERAL_DISCOVERABLE: u8 = 2;

pub const BTHOME_UUID16: u16 = 0xFCD2;
pub const BTHOME_UUID: u128 = 0x0000FCD2_0000_1000_8000_00805F9B34FB;

pub struct BtHomeAd<const N: usize> {
    buffer: Vec<u8, N>,
}

impl<const N: usize> BtHomeAd<N> {
    #[inline]
    pub fn add_data(&mut self, payload: impl IntoIterator<Item = u8>) -> &mut Self {
        self.buffer.extend(payload);

        self
    }

    #[inline]
    pub fn add_local_name(&mut self, name: &str) -> &mut Self {
        self.buffer.extend([(name.len() + 1) as u8, 0x09]);
        self.buffer.extend_from_slice(name.as_bytes()).ok();
        self
    }

    #[inline]
    pub fn encode(&self) -> &[u8] {
        &self.buffer
    }
}

impl<const N: usize> Default for BtHomeAd<N> {
    #[inline]
    fn default() -> Self {
        let mut buffer =
            Vec::from_iter([0x02, 0x01, LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED]);

        buffer.extend(BTHOME_UUID16.to_le_bytes());
        buffer.extend([0x40]);

        Self { buffer }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_add_name() {
        let mut home = BtHomeAd::<31>::default();

        let name = "hello";

        home.add_local_name(name);

        assert_eq!(home.buffer.len(), 13);

        assert_eq!(
            home.encode(),
            &[
                0x02,
                0x01,
                LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED,
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
}
