mod crc;
mod cube;
mod messages;

use aes::{cipher::BlockDecrypt, Block};
use async_stream::stream;
use btleplug::{
    api::{bleuuid::uuid_from_u16, Peripheral as _},
    platform::Peripheral,
};
use futures::stream::Stream;
use messages::{C2aBody, CubeHello, StateChange};
use smartcube::SmartcubeEvent;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

#[derive(Debug)]
pub struct Driver;

impl smartcube::Driver for Driver {
    fn name(&self) -> &'static str {
        "QiYi Smartcube"
    }

    fn check_compat<'a>(
        &self,
        perip: &'a Peripheral,
    ) -> Pin<Box<dyn Future<Output = bool> + Send + 'a>> {
        Box::pin(async move {
            let props = perip.properties().await.unwrap();

            if let Some(props) = props {
                props
                    .local_name
                    .map(|name| name.starts_with("QY-QYSC"))
                    .unwrap_or(false)
            } else {
                false
            }
        })
    }

    fn events(&self, perip: Peripheral) -> Pin<Box<dyn Stream<Item = SmartcubeEvent> + Send>> {
        Box::pin(run_protocol(perip))
    }
}

fn run_protocol(perip: Peripheral) -> impl Stream<Item = SmartcubeEvent> + Send {
    stream! {
        perip.discover_services().await.unwrap();

        let fff6 = perip
            .characteristics()
            .into_iter()
            .find(|c| c.uuid == uuid_from_u16(0xfff6))
            .unwrap();

        perip.subscribe(&fff6).await.unwrap();

        let mut cube = cube::Cube::new(perip, fff6);
        let notifs = cube.perip.notifications().await.unwrap();

        // send App Hello
        cube.write_cmd_inner_bytes(&messages::make_app_hello(cube.perip.address()))
            .await;

        for await n in notifs {
            assert!(n.uuid == cube.fff6.uuid);
            let mut bytes = n.value;
            assert!(bytes.len() % 16 == 0);

            for mut block in bytes.chunks_mut(16).map(Block::from_mut_slice) {
                cube.cipher.decrypt_block(&mut block);
            }

            let msg = messages::parse_c2a_message(&bytes).unwrap();

            if let Some(pkt) = msg.make_ack() {
                cube.write_cmd_inner_bytes(pkt).await;
            }

            let timestamp = msg.timestamp();
            let instant = cube
                .epoch
                .checked_add(Duration::from_millis(timestamp.into()))
                .unwrap();

            match msg.into_body() {
                C2aBody::CubeHello(CubeHello { state, battery })
                | C2aBody::StateChange(StateChange { state, battery, .. }) => {
                    if cube.last_bat != Some(battery) {
                        cube.last_bat = Some(battery);
                        yield SmartcubeEvent::Battery(battery);
                    }

                    yield SmartcubeEvent::StateChange(state, instant);
                }
            }
        }
    }
}
