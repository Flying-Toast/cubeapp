use crate::crc::crc16;
use anyhow::{anyhow, bail, Result};
use btleplug::api::BDAddr;
use cubestruct::{Color, CubieCube, FaceletCube, Move};
use thiserror::Error;

#[derive(Debug)]
enum Opcode {
    CubeHello,
    StateChange,
    SyncConfirmation,
}

impl Opcode {
    fn from_u8(x: u8) -> Result<Self> {
        Ok(match x {
            0x2 => Self::CubeHello,
            0x3 => Self::StateChange,
            0x4 => Self::SyncConfirmation,
            _ => bail!(ParseError::BadOpcode { bad_opcode: x }),
        })
    }
}

/// A cube->app message.
#[derive(Debug)]
pub struct C2aMessage<'a> {
    /// Reference to bytes 3-7 for use in ACKs
    ack_head: &'a [u8],
    millis_timestamp: u32,
    body: C2aBody,
}

impl<'a> C2aMessage<'a> {
    fn needs_ack(&self) -> bool {
        match &self.body {
            C2aBody::CubeHello(_) => true,
            C2aBody::StateChange(sc) => sc.needs_ack,
        }
    }

    /// Returns `Some(ack)` if this message needs to be ACKed;
    /// returns `None` if it doesn't need an ACK.
    // TODO: make structs for app->cube messages instead of returning &[u8] here
    pub fn make_ack(&self) -> Option<&'a [u8]> {
        if self.needs_ack() {
            Some(self.ack_head)
        } else {
            None
        }
    }

    /// Get the timestamp in milliseconds
    pub fn timestamp(&self) -> u32 {
        self.millis_timestamp
    }

    pub fn into_body(self) -> C2aBody {
        self.body
    }
}

/// The "body" of a cube->app message is the decrypted contents
/// minus the `0xfe` prefix, length, opcode, padding, and checksum.
#[derive(Debug)]
pub enum C2aBody {
    CubeHello(CubeHello),
    StateChange(StateChange),
}

#[derive(Debug)]
pub struct CubeHello {
    pub state: CubieCube,
    pub battery: u8,
}

fn move_from_byte(x: u8) -> Result<Move> {
    Ok(match x {
        1 => Move::Li,
        2 => Move::L,
        3 => Move::Ri,
        4 => Move::R,
        5 => Move::Di,
        6 => Move::D,
        7 => Move::Ui,
        8 => Move::U,
        9 => Move::Fi,
        10 => Move::F,
        11 => Move::Bi,
        12 => Move::B,
        _ => bail!(ParseError::BadTurn { turn: x }),
    })
}

#[derive(Debug)]
pub struct StateChange {
    pub state: CubieCube,
    pub battery: u8,
    pub turn: Move,
    pub needs_ack: bool,
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Missing magic `0xfe` byte at start of message")]
    BadMagic,
    #[error("Expected message to be longer (tried to index outside the message)")]
    TooShort,
    #[error("Invalid checksum")]
    FailedChecksum,
    #[error("Invalid opcode (got {bad_opcode})")]
    BadOpcode { bad_opcode: u8 },
    #[error("Invalid turn ({turn} is not a valid move)")]
    BadTurn { turn: u8 },
}

struct Parser<'a> {
    bytes: &'a [u8],
}

impl<'a> Parser<'a> {
    fn get_bytes(&self, idx: usize, n: usize) -> Result<&'a [u8]> {
        self.bytes
            .get(idx..idx + n)
            .ok_or(anyhow!(ParseError::TooShort))
    }

    fn trim_padding(&mut self, message_length: u8) {
        self.bytes = &self.bytes[..message_length as usize];
    }

    fn get_u8(&self, idx: usize) -> Result<u8> {
        self.bytes
            .get(idx)
            .copied()
            .ok_or(anyhow!(ParseError::TooShort))
    }

    fn get_u16(&self, idx: usize) -> Result<u16> {
        Ok(u16::from_le_bytes(
            self.get_bytes(idx, 2)?.try_into().unwrap(),
        ))
    }

    fn get_u32_be(&self, idx: usize) -> Result<u32> {
        Ok(u32::from_be_bytes(
            self.get_bytes(idx, 4)?.try_into().unwrap(),
        ))
    }
}

pub fn make_app_hello(mac: BDAddr) -> Vec<u8> {
    // fill the 11-byte unknown field with zeros
    let mut v = vec![0; 11];

    let mut mac = mac.into_inner();
    mac.reverse();

    v.extend_from_slice(&mac);

    v
}

/// Given the bytes of an **decrypted** message, parse them into a cube->app message.
pub fn parse_c2a_message(bytes: &[u8]) -> Result<C2aMessage> {
    let mut p = Parser { bytes };

    if p.get_u8(0)? != 0xfe {
        bail!(ParseError::BadMagic);
    }

    let length = p.get_u8(1)?;
    if p.bytes.len() < length as usize {
        bail!(ParseError::TooShort);
    }
    p.trim_padding(length);
    let checksum = p.get_u16(length as usize - 2)?;
    if crc16(p.get_bytes(0, length as usize - 2)?) != checksum {
        bail!(ParseError::FailedChecksum);
    }

    let opcode = Opcode::from_u8(p.get_u8(2)?)?;
    let millis_timestamp = (p.get_u32_be(3)? as f32 / 1.6) as u32;
    let body = match opcode {
        Opcode::CubeHello => {
            let rawstate = p.get_bytes(7, 27)?;
            let battery = p.get_u8(35)?;

            C2aBody::CubeHello(CubeHello {
                state: cubie_cube_from_bytes(rawstate),
                battery,
            })
        }
        Opcode::StateChange => {
            let rawstate = p.get_bytes(7, 27)?;
            let turnbyte = p.get_u8(34)?;
            let battery = p.get_u8(35)?;
            let needs_ack = p.get_u8(91)? == 1;

            // workaround for slice move glitch
            let state = if needs_ack {
                CubieCube::SOLVED
            } else {
                cubie_cube_from_bytes(rawstate)
            };

            C2aBody::StateChange(StateChange {
                turn: move_from_byte(turnbyte)?,
                state,
                needs_ack,
                battery,
            })
        }
        Opcode::SyncConfirmation => {
            todo!()
        }
    };

    assert!(p.bytes.len() >= 7);

    Ok(C2aMessage {
        ack_head: p.get_bytes(2, 5)?,
        millis_timestamp,
        body,
    })
}

fn cubie_cube_from_bytes(raw: &[u8]) -> CubieCube {
    let color_order = [
        Color::White,
        Color::Red,
        Color::Green,
        Color::Yellow,
        Color::Orange,
        Color::Blue,
    ];
    let mut builder = FaceletCube::builder();

    let mut facelet_colors = raw
        .iter()
        .flat_map(|&x| [x & 0xf, (x & 0xF0) >> 4])
        .map(|x| color_from_u8(x).unwrap());

    for face_color in color_order {
        for i in 0..9 {
            builder.set(face_color, i, facelet_colors.next().unwrap());
        }
    }

    builder.build().unwrap().to_cubie_cube().unwrap()
}

fn color_from_u8(x: u8) -> Option<Color> {
    Some(match x {
        0 => Color::Orange,
        1 => Color::Red,
        2 => Color::Yellow,
        3 => Color::White,
        4 => Color::Green,
        5 => Color::Blue,
        _ => return None,
    })
}
