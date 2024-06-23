use crate::crc::crc16;
use aes::{
    cipher::{BlockEncrypt, KeyInit},
    Aes128, Block,
};
use btleplug::api::{Characteristic, Peripheral as _, WriteType};
use btleplug::platform::Peripheral;
use std::time::Instant;

pub struct Cube {
    pub perip: Peripheral,
    pub fff6: Characteristic,
    pub last_bat: Option<u8>,
    pub cipher: Aes128,
    pub epoch: Instant,
}

impl Cube {
    pub fn new(perip: Peripheral, fff6: Characteristic) -> Self {
        Self {
            perip,
            fff6,
            last_bat: None,
            cipher: Aes128::new(
                &[
                    87, 177, 249, 171, 205, 90, 232, 167, 156, 185, 140, 231, 87, 140, 81, 8,
                ]
                .into(),
            ),
            epoch: Instant::now(),
        }
    }

    /// Given the bytes of an app->cube command:
    /// - prefixes with `0xfe` and the length;
    /// - computes the checksum and appends it to the end;
    /// - adds zero-padding;
    /// - encrypts the message;
    /// - writes it to the fff6 characteristic
    pub async fn write_cmd_inner_bytes(&mut self, bytes: &[u8]) {
        // +2 for checksum, +2 for fe/length prefix
        let cmdlen = bytes.len() + 2 + 2;
        let npad = if cmdlen % 16 == 0 {
            0
        } else {
            16 - (cmdlen % 16)
        };
        let total_len = npad + cmdlen;
        assert!(total_len % 16 == 0);

        let mut bytes = {
            let mut v = Vec::<u8>::with_capacity(total_len);
            v.push(0xfe);
            v.push(cmdlen.try_into().expect("Packet len > 255"));
            v.extend_from_slice(bytes);
            v.extend_from_slice(&crc16(&v).to_le_bytes());
            v.resize(total_len, 0);
            v
        };

        // encrypt bytes
        for mut block in bytes.chunks_mut(16).map(Block::from_mut_slice) {
            self.cipher.encrypt_block(&mut block);
        }

        self.perip
            .write(&self.fff6, &bytes, WriteType::WithoutResponse)
            .await
            .unwrap();
    }
}
