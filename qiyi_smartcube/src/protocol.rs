use crate::crc::crc16;
use crate::messages::{self, C2aBody, CubeHello, StateChange};
use aes::{
    cipher::{BlockDecrypt, BlockEncrypt, KeyInit},
    Aes128, Block,
};
use btleplug::{
    api::{Characteristic, Peripheral as _, ValueNotification, WriteType},
    platform::Peripheral,
};
use futures::stream::{self, Stream};
use smartcube::SmartcubeEvent;
use std::time::{Duration, Instant};

pub struct Cube {
    pub perip: Peripheral,
    pub fff6: Characteristic,
    cipher: Aes128,
    last_battery: Option<u8>,
    epoch: Instant,
}

impl Cube {
    pub fn new(perip: Peripheral, fff6: Characteristic) -> Self {
        Self {
            perip,
            fff6,
            cipher: Aes128::new(
                &[
                    87, 177, 249, 171, 205, 90, 232, 167, 156, 185, 140, 231, 87, 140, 81, 8,
                ]
                .into(),
            ),
            last_battery: None,
            epoch: Instant::now(),
        }
    }

    pub async fn write_cmd_inner_bytes(&self, bytes: &[u8]) {
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

    pub async fn apply_notif(
        &mut self,
        notif: ValueNotification,
    ) -> Option<impl Stream<Item = SmartcubeEvent>> {
        assert!(notif.uuid == self.fff6.uuid);
        let mut bytes = notif.value;
        assert!(bytes.len() % 16 == 0);

        for mut block in bytes.chunks_mut(16).map(Block::from_mut_slice) {
            self.cipher.decrypt_block(&mut block);
        }

        let msg = messages::parse_c2a_message(&bytes).unwrap();

        if let Some(pkt) = msg.make_ack() {
            self.write_cmd_inner_bytes(pkt).await;
        }

        match msg.into_body() {
            C2aBody::CubeHello(CubeHello { state, battery })
            | C2aBody::StateChange(StateChange { state, battery, .. }) => {
                let newbat = Some(battery);

                let batmsg = if newbat != self.last_battery {
                    self.last_battery = newbat;

                    Some(SmartcubeEvent::Battery(battery))
                } else {
                    None
                };

                let ts = self
                    .epoch
                    .checked_add(Duration::from_millis(msg.timestamp().into()))
                    .unwrap();

                stream::iter(std::iter::once(SmartcubeEvent::StateChange(state, ts)).chain(batmsg))
            }
        }
    }
}
