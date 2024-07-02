use crate::crc::crc16;
use anyhow::{anyhow, bail, Result};
use btleplug::api::BDAddr;
use cubestruct::{Color, CubeState};
use std::fmt;
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
    pub state: CubeState,
    pub battery: u8,
}

#[derive(Debug)]
pub enum Turn {
    Li,
    L,
    Ri,
    R,
    Di,
    D,
    Ui,
    U,
    Fi,
    F,
    Bi,
    B,
}

impl Turn {
    fn from_byte(x: u8) -> Result<Turn> {
        Ok(match x {
            1 => Self::Li,
            2 => Self::L,
            3 => Self::Ri,
            4 => Self::R,
            5 => Self::Di,
            6 => Self::D,
            7 => Self::Ui,
            8 => Self::U,
            9 => Self::Fi,
            10 => Self::F,
            11 => Self::Bi,
            12 => Self::B,
            _ => bail!(ParseError::BadTurn { turn: x }),
        })
    }
}

impl fmt::Display for Turn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Li => write!(f, "L'"),
            Self::L => write!(f, "L"),
            Self::Ri => write!(f, "R'"),
            Self::R => write!(f, "R"),
            Self::Di => write!(f, "D'"),
            Self::D => write!(f, "D"),
            Self::Ui => write!(f, "U'"),
            Self::U => write!(f, "U"),
            Self::Fi => write!(f, "F'"),
            Self::F => write!(f, "F"),
            Self::Bi => write!(f, "B'"),
            Self::B => write!(f, "B"),
        }
    }
}

#[derive(Debug)]
pub struct StateChange {
    pub state: CubeState,
    pub battery: u8,
    pub turn: Turn,
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
                state: cubestate_from_bytes(rawstate),
                battery,
            })
        }
        Opcode::StateChange => {
            let rawstate = p.get_bytes(7, 27)?;
            let turnbyte = p.get_u8(34)?;
            let battery = p.get_u8(35)?;
            let needs_ack = p.get_u8(91)? == 1;

            C2aBody::StateChange(StateChange {
                turn: Turn::from_byte(turnbyte)?,
                state: cubestate_from_bytes(rawstate),
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

fn cubestate_from_bytes(raw: &[u8]) -> CubeState {
    CubeState {
        facelets: raw
            .iter()
            .flat_map(|&x| [x & 0xf, (x & 0xF0) >> 4])
            .map(|x| color_from_u8(x).unwrap())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap(),
    }
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
