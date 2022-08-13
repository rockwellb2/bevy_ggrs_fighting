use std::default;

use bevy::{reflect::FromReflect};
use bevy::prelude::*;
use bytemuck::{Pod, Zeroable};
use ggrs::PlayerHandle;
use packed_struct::prelude::{PrimitiveEnum_u8, PackedStruct};
use serde::{Deserialize, Serialize, de};

use crate::util::Buffer;

pub const BUFFER_SIZE: usize = 10;


const LP: u16 = 1 << 0;
const MP: u16 = 1 << 1;
const HP: u16 = 1 << 2;
const LK: u16 = 1 << 3;
const MK: u16 = 1 << 4;
const HK: u16 = 1 << 5;

pub const LEFT: u16 = 1 << 6;
pub const RIGHT: u16 = 1 << 7;
pub const UP: u16 = 1 << 8;
pub const DOWN: u16 = 1 << 9;
pub const EMPTY: u16 = 0;

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Pod, Zeroable, Reflect, FromReflect)]
pub struct Input(pub u16);



pub fn input(
    _handle: In<PlayerHandle>,
    keyboard_input: Res<bevy::input::Input<KeyCode>>
) -> Input 
{
    let mut inp: u16 = 0;

    if keyboard_input.pressed(KeyCode::U) {
        inp |= LP;
    }
    if keyboard_input.pressed(KeyCode::I) {
        inp |= MP;
    }
    if keyboard_input.pressed(KeyCode::O) {
        inp |= HP;
    }
    if keyboard_input.pressed(KeyCode::J) {
        inp |= LK;
    }
    if keyboard_input.pressed(KeyCode::K) {
        inp |= MK;
    }
    if keyboard_input.pressed(KeyCode::L) {
        inp |= HK;
    }


    if keyboard_input.pressed(KeyCode::A) {
        inp |= LEFT;
    }
    if keyboard_input.pressed(KeyCode::D) {
        inp |= RIGHT;
    }
    if keyboard_input.pressed(KeyCode::W) {
        inp |= UP;
    }
    if keyboard_input.pressed(KeyCode::S) {
        inp |= DOWN;
    }

    Input(inp)
}


#[derive(Debug, Serialize, Deserialize, Default, FromReflect, Reflect, Clone)]
pub struct MatchExpression {
    #[serde(deserialize_with = "deserialize_bits")]
    with: u16,
    #[serde(deserialize_with = "deserialize_bits")]
    #[serde(default)]
    without: u16,
}

fn deserialize_bits<'de, D>(deserializer: D) -> Result<u16, D::Error>
where 
    D: de::Deserializer<'de>,
{
    struct BitVisitor;

    impl <'de> de::Visitor<'de> for BitVisitor {
        type Value = u16;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string containing a 16-bit value")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error, 
        
        {
            let trimmed = v.trim().trim_start_matches("0b");


            return if let Ok(bits) = u16::from_str_radix(trimmed, 2) {
                Ok(bits)
            }

            else {
                Err(E::custom("error deserializing bits from string"))
            }  
        }
    }

    deserializer.deserialize_any(BitVisitor)

}

#[derive(Debug, Serialize, Deserialize, Default, FromReflect, Reflect, Clone)]
pub struct CommandInput {
    list: Vec<MatchExpression>,
    window: u16
}

impl CommandInput {
    pub fn compare(&self, input: &Buffer<Input>) -> bool {
        // let mut input = input.clone();
        // input.reverse();
        // let mut input_iter = input.iter().rev();
        let mut input_iter = input.into_iter();

        let mut index = 0;


        for command in &self.list {
            loop {
                index += 1;
                if index > self.window {
                    return false;
                }

                if let Some(next) = input_iter.next() {
                    let has = next.0 & command.with == command.with;
                    let not = next.0 & command.without == 0;

                    if has && not {
                        break;
                    }
                }
                else {
                    return false;
                }
            }
        }

        return true;
    }
}

#[derive(PrimitiveEnum_u8, Copy, Clone, Debug, Default)]
pub enum ButtonPress {
    #[default]
    None = 0,
    Press = 1,
    Hold = 2,
    Release = 3,
}

#[derive(PrimitiveEnum_u8, Copy, Clone, Debug, Default)]
pub enum DirectionalInput {
    #[default]
    None = 0,
    Positive = 1,
    Negative = 2,
}

#[derive(PackedStruct, Default)]
#[packed_struct(bit_numbering="msb0")]
pub struct NewInput {
    #[packed_field(bits="0..=1", ty= "enum")]
    a: ButtonPress,
    #[packed_field(bits="2..=3", ty= "enum")]
    b: ButtonPress,
    #[packed_field(bits="4..=5", ty= "enum")]
    c: ButtonPress,
    #[packed_field(bits="6..=7", ty= "enum")]
    d: ButtonPress,
    #[packed_field(bits="8..=9", ty= "enum")]
    e: ButtonPress,
    #[packed_field(bits="10..=11", ty= "enum")]
    f: ButtonPress,
    #[packed_field(bits="12..=13", ty= "enum")]
    g: DirectionalInput,
    #[packed_field(bits="14..=15", ty= "enum")]
    h: DirectionalInput
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use packed_struct::PackedStruct;

    use super::{NewInput, ButtonPress};

    #[test]
    fn something() {
        let input = NewInput::default();
        let packed = input.pack().unwrap();
        
    }

}