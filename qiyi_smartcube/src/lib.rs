mod crc;
mod messages;
mod protocol;

use anyhow::{Context, Result};
use btleplug::{
    api::{bleuuid::uuid_from_u16, Peripheral as _},
    platform::Peripheral,
};
use futures::stream::{self, Stream, StreamExt};
use smartcube::{Smartcube, SmartcubeEvent};
use std::pin::Pin;

pub struct QiyiSmartcube;

impl Smartcube for QiyiSmartcube {
    async fn matches_peripheral(perip: &Peripheral) -> Result<bool> {
        Ok(match perip.properties().await? {
            None => false,
            Some(props) => props
                .local_name
                .iter()
                .any(|name| name.starts_with("QY-QYSC")),
        })
    }

    async fn connect(perip: Peripheral) -> Result<Pin<Box<dyn Stream<Item = SmartcubeEvent>>>> {
        perip.connect().await?;
        perip.discover_services().await?;

        let fff6 = perip
            .characteristics()
            .into_iter()
            .find(|c| c.uuid == uuid_from_u16(0xfff6))
            .context("Expected QiYi cube to have an fff6 characteristic")?;

        perip.subscribe(&fff6).await?;

        let notifications = perip.notifications().await?;
        let cube = protocol::Cube::new(perip, fff6);

        cube.write_cmd_inner_bytes(&messages::make_app_hello(cube.perip.address()))
            .await;

        Ok(Box::pin(
            stream::unfold(
                (cube, notifications),
                |(mut cube, mut notifications)| async {
                    while let Some(notif) = notifications.next().await {
                        if let Some(ev) = cube.apply_notif(notif).await {
                            return Some((ev, (cube, notifications)));
                        }
                    }
                    None
                },
            )
            .flatten(),
        ))
    }
}
