use bevy::prelude::*;
use bevy::reflect::FromReflect;
use bytemuck::{Pod, Zeroable};
use ggrs::PlayerHandle;
use leafwing_input_manager::prelude::ActionState;
use leafwing_input_manager::Actionlike;
use packed_struct::prelude::{PackedStruct, PrimitiveEnum_u8};
use packed_struct::types::bits::Bits;
use packed_struct::types::ReservedZero;
use serde::{de, Deserialize, Serialize};

use crate::fighter::systems::InputBuffer;
use crate::fighter::Fighter;
use crate::util::Buffer;

pub const BUFFER_SIZE: usize = 10;


pub const LEFT_HELD: u32 = 2048;
pub const LEFT: u32 = 2560;

pub const RIGHT_HELD: u32 = 1024;
pub const RIGHT: u32 = 1536;



#[repr(C)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Pod, Zeroable, Reflect, FromReflect)]
pub struct Input(pub u32);

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum Action {
    Lp,
    Mp,
    Hp,
    Lk,
    Mk,
    Hk,
    Left,
    Right,
    Up,
    Down,
}

pub fn input(
    handle: In<PlayerHandle>,
    mut input_query: Query<(&ActionState<Action>, &mut InputBuffer)>,
    mut local: Local<(StateInput, StateInput)>



    //keyboard_input: Res<bevy::input::Input<KeyCode>>,
    // mut input_params: ParamSet<(
    //     Query<(&ActionState<Action>, &InputBuffer), With<Fighter>>,
    //     Query<&mut InputBuffer, With<ActionState<Action>>>
    // )>,
) -> Input 
{
    if handle.0 == 1 {
        return Input(StateInput::default().into());
    }
    // let output = 
    //  if let Ok((action_state, buffer)) = input_params.p0().get_single() {
    //     if let Some(previous) = buffer.0.get(0) {
    //         let previous: StateInput = previous.into();

    //         let button_check = |action: Action| -> ButtonPress {
    //             if action_state.pressed(action) {
    //                 match previous.get_button_from_action(action) {
    //                     ButtonPress::Press | ButtonPress::Hold => ButtonPress::Hold,
    //                     ButtonPress::None | ButtonPress::Release => ButtonPress::Press,
    //                 }
    //             } else {
    //                 match previous.get_button_from_action(action) {
    //                     ButtonPress::Press | ButtonPress::Hold => ButtonPress::Release,
    //                     ButtonPress::None | ButtonPress::Release => ButtonPress::None,
    //                 }
    //             }
    //         };

    //         let directional_check = |pos: Action, neg: Action| -> (DirectionalInput, bool) {
    //             if action_state.pressed(pos) {
    //                 match (previous.get_directional_from_button(pos)) {
    //                     (DirectionalInput::Positive, _) => (DirectionalInput::Positive, false),
    //                     _ => (DirectionalInput::Positive, true),
    //                 }
    //             } else if action_state.pressed(neg) {
    //                 match (previous.get_directional_from_button(pos)) {
    //                     (DirectionalInput::Negative, _) => (DirectionalInput::Negative, false),
    //                     _ => (DirectionalInput::Negative, true),
    //                 }
    //             } else {
    //                 match previous.get_directional_from_button(pos) {
    //                     (DirectionalInput::None, _) => (DirectionalInput::None, false),
    //                     _ => (DirectionalInput::None, false),
    //                 }
    //             }
    //         };

    //         let lp = button_check(Action::Lp);
    //         let mp = button_check(Action::Mp);
    //         let hp = button_check(Action::Hp);
    //         let lk = button_check(Action::Lk);
    //         let mk = button_check(Action::Mk);
    //         let hk = button_check(Action::Hk);

    //         let (x, just_pressed_x) = directional_check(Action::Right, Action::Left);
    //         let (y, just_pressed_y) = directional_check(Action::Up, Action::Down);

    //         let inp = StateInput::new(lp, mp, hp, lk, mk, hk, x, just_pressed_x, y, just_pressed_y);

    //         if inp.hk.is_held() {
    //             println!("LP | This frame: {:?}, Prev frame: {:?}", inp.lp, previous.lp);
    //         }

            
    //         Input(inp.into())
    //     } else {
    //         Input(StateInput::default().into())
    //     }
    // } else {
    //     Input(StateInput::default().into())
    // };

    // if let Ok(mut buffer) = input_params.p1().get_single_mut() {
    //     println!("Does this??????");
    //     buffer.0.push(output.0);
    // }

    // output

    if let Ok((action_state, mut buffer)) = input_query.get_single_mut() {
        let previous = &local.0;

        let button_check = |action: Action| -> ButtonPress {
            let prev = previous.get_button_from_action(action);
            let check = action_state.pressed(action);

            if check {
                if prev == ButtonPress::Press || prev == ButtonPress::Hold {
                    ButtonPress::Hold
                }
                else {
                    ButtonPress::Press
                }
            }
            else {
                if prev == ButtonPress::None || prev == ButtonPress::Release {
                    ButtonPress::None
                }
                else {
                    ButtonPress::Release
                }
            }
        };

        let directional_check = |pos: Action, neg: Action| -> (DirectionalInput, bool) {
            let prev = previous.get_directional_from_button(pos);
            let check_pos = action_state.pressed(pos);
            let check_neg = action_state.pressed(neg);

            if check_pos {
                match prev {
                    (DirectionalInput::Positive, _) => (DirectionalInput::Positive, false),
                    _ => (DirectionalInput::Positive, true)
                }
            }
            else if check_neg {
                match prev {
                    (DirectionalInput::Negative, _) => (DirectionalInput::Negative, false),
                    _ => (DirectionalInput::Negative, true)
                }
            }
            else {
                (DirectionalInput::None, false)
            }


        };

        let lp = button_check(Action::Lp);
        let mp = button_check(Action::Mp);
        let hp = button_check(Action::Hp);
        let lk = button_check(Action::Lk);
        let mk = button_check(Action::Mk);
        let hk = button_check(Action::Hk);

        let (x, just_pressed_x) = directional_check(Action::Right, Action::Left);
        let (y, just_pressed_y) = directional_check(Action::Up, Action::Down);

        let inp = StateInput::new(lp, mp, hp, lk, mk, hk, x, just_pressed_x, y, just_pressed_y);

        if inp.hk.is_held() {
            println!("Y-axis: {:?}, Just pressed: {}", inp.y, inp.just_pressed_y);
        }

        local.0 = inp.clone();

        let inp: u32 = inp.into();
        buffer.0.insert(inp);
        // buffer.0.pop_back();
        // buffer.0.push_front(inp);


        Input(inp)
    }
    else {
        Input(StateInput::default().into())
    }
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

    impl<'de> de::Visitor<'de> for BitVisitor {
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
            } else {
                Err(E::custom("error deserializing bits from string"))
            };
        }
    }

    deserializer.deserialize_any(BitVisitor)
}

#[derive(Debug, Serialize, Deserialize, Default, FromReflect, Reflect, Clone)]
pub struct CommandInput {
    list: Vec<MatchExpression>,
    window: u16,
}

#[derive(Debug, Serialize, Deserialize, FromReflect, Reflect, Clone)]
#[serde(untagged)]
pub enum NewMatchExpression {
    Button(String, ButtonPress),
    Directional(String, DirectionalInput, bool),
}

impl Default for NewMatchExpression {
    fn default() -> Self {
        Self::Button("lp".to_string(), ButtonPress::None)
    }
}

#[derive(Debug, Serialize, Deserialize, Default, FromReflect, Reflect, Clone)]
pub struct NewCommandInput {
    list: Vec<Vec<NewMatchExpression>>,
    window: u16,
}

impl NewCommandInput {
    pub fn compare(&self, input: &Buffer) -> bool {
        let mut input_iter = input.iter();
        let mut index = 0;

        for command in &self.list {
            
            loop {
                index += 1;
                if index > self.window {
                    return false;
                }

                //println!("Peek at next value: {:?}", input_iter.peek());

                if let Some(next) = input_iter.next() {
                    let next: StateInput = next.into();

                    if next.lp == ButtonPress::Press {
                        print!("");
                    }

                    let mut command_iter = command.iter();
                    let mut same = true;
                    while let Some(expression) = command_iter.next() {
                        if !next.compare_command(expression.clone()) {
                            if index == 1 {
                                return false;
                            }
                            same = false;
                        }
                    }

                    if same {
                        break;
                    }
                } else {
                    return false;
                }
            }
        }

        return true;
    }
}

fn deserialize_command<'de, D>(deserializer: D) -> Result<Vec<u32>, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct CommandVisitor;

    impl<'de> de::Visitor<'de> for CommandVisitor {
        type Value = Vec<u32>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string containing a 16-bit value")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut out_vec: Vec<u32> = Vec::new();

            while let Some(element) = seq.next_element::<StateInput>()? {
                out_vec.push(element.into())
            }

            Ok(out_vec)
        }
    }

    deserializer.deserialize_any(CommandVisitor)
}

#[cfg(test)]
mod input_tests {
    use super::{NewCommandInput, StateInput};

    #[test]
    fn deserialize_command() {
        let string = r#"{
            "list": [
                {
                    "lk": "Press"
                }
            ],
            "window": 1
        }"#;

        let result: NewCommandInput = serde_json::from_str(string).unwrap();
        //let input: StateInput = result.list.get(0).unwrap().to_owned().into();

        // println!("{:?}", input);
    }
}

#[derive(PrimitiveEnum_u8, Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum ButtonPress {
    #[default]
    None = 0,
    Press = 1,
    Hold = 2,
    Release = 3,
}

impl ButtonPress {
    pub fn from_u32(value: u32) -> ButtonPress {
        match value {
            0 => ButtonPress::None,
            1 => ButtonPress::Press,
            2 => ButtonPress::Hold,
            3 => ButtonPress::Release,
            _ => panic!("Unknown value"),
        }
    }

    pub fn is_held(&self) -> bool {
        self == &ButtonPress::Hold
    }
}

impl From<u32> for ButtonPress {
    fn from(value: u32) -> Self {
        Self::from_u32(value)
    }
}

#[derive(PrimitiveEnum_u8, Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum DirectionalInput {
    #[default]
    None = 0,
    Positive = 1,
    Negative = 2,
}

impl DirectionalInput {
    pub fn from_u32(value: u32) -> DirectionalInput {
        match value {
            0 => DirectionalInput::None,
            1 => DirectionalInput::Positive,
            2 => DirectionalInput::Negative,
            _ => panic!("Unknown value"),
        }
    }
}

impl From<u32> for DirectionalInput {
    fn from(value: u32) -> Self {
        Self::from_u32(value)
    }
}

#[derive(PackedStruct, Default, PartialEq, Debug, Serialize, Deserialize, Clone)]
#[packed_struct(bit_numbering = "msb0")]
pub struct StateInput {
    #[serde(default)]
    #[packed_field(bits = "0..=1", ty = "enum")]
    pub lp: ButtonPress,
    #[serde(default)]
    #[packed_field(bits = "2..=3", ty = "enum")]
    pub mp: ButtonPress,
    #[serde(default)]
    #[packed_field(bits = "4..=5", ty = "enum")]
    pub hp: ButtonPress,
    #[serde(default)]
    #[packed_field(bits = "6..=7", ty = "enum")]
    pub lk: ButtonPress,
    #[serde(default)]
    #[packed_field(bits = "8..=9", ty = "enum")]
    pub mk: ButtonPress,
    #[serde(default)]
    #[packed_field(bits = "10..=11", ty = "enum")]
    pub hk: ButtonPress,
    #[serde(default)]
    #[packed_field(bits = "12..=13", ty = "enum")]
    pub x: DirectionalInput,
    #[serde(default)]
    #[packed_field(bits = "14")]
    pub just_pressed_x: bool,
    #[serde(default)]
    #[packed_field(bits = "15..=16", ty = "enum")]
    pub y: DirectionalInput,
    #[serde(default)]
    #[packed_field(bits = "17")]
    pub just_pressed_y: bool,
    #[serde(default)]
    #[packed_field(bits = "18..=31")]
    _reserved: ReservedZero<Bits<14>>,
}

impl StateInput {
    pub fn new(
        lp: ButtonPress,
        mp: ButtonPress,
        hp: ButtonPress,
        lk: ButtonPress,
        mk: ButtonPress,
        hk: ButtonPress,
        x: DirectionalInput,
        just_pressed_x: bool,
        y: DirectionalInput,
        just_pressed_y: bool,
    ) -> StateInput {
        StateInput {
            lp,
            mp,
            hp,
            lk,
            mk,
            hk,
            x,
            just_pressed_x,
            y,
            just_pressed_y,
            _reserved: ReservedZero::default(),
        }
    }

    pub fn get_button_from_action(&self, action: Action) -> ButtonPress {
        match action {
            Action::Lp => self.lp,
            Action::Mp => self.mp,
            Action::Hp => self.hp,
            Action::Lk => self.lk,
            Action::Mk => self.mk,
            Action::Hk => self.hk,
            _ => panic!(),
        }
    }

    pub fn get_directional_from_button(&self, action: Action) -> (DirectionalInput, bool) {
        match action {
            Action::Left => (self.x, self.just_pressed_x),
            Action::Right => (self.x, self.just_pressed_x),
            Action::Up => (self.y, self.just_pressed_y),
            Action::Down => (self.y, self.just_pressed_y),
            _ => panic!(),
        }
    }

    pub fn compare_command(&self, command: NewMatchExpression) -> bool {
        match command {
            NewMatchExpression::Button(name, button) => {
                let i = match name.as_str() {
                    "lp" => self.lp,
                    "mp" => self.mp,
                    "hp" => self.hp,
                    "lk" => self.lk,
                    "mk" => self.mk,
                    "hk" => self.hk,
                    _ => panic!(),
                };

                i == button
            }
            NewMatchExpression::Directional(name, direction, just_pressed) => {
                let i = match name.as_str() {
                    "x" => (self.x, self.just_pressed_x),
                    "y" => (self.y, self.just_pressed_y),
                    _ => panic!(),
                };

                i.0 == direction && (i.1 == just_pressed)
            }
        }
    }

    pub fn lp(just_pressed: bool) -> Self {
        let lp = if just_pressed {
            ButtonPress::Press
        } else {
            ButtonPress::Hold
        };

        StateInput {
            lp,
            ..Default::default()
        }
    }

    pub fn lp_bits(just_pressed: bool) -> u32 {
        Self::lp(just_pressed).into()
    }

    pub fn mp(just_pressed: bool) -> Self {
        let mp = if just_pressed {
            ButtonPress::Press
        } else {
            ButtonPress::Hold
        };

        StateInput {
            mp,
            ..Default::default()
        }
    }

    pub fn mp_bits(just_pressed: bool) -> u32 {
        Self::mp(just_pressed).into()
    }

    pub fn hp(just_pressed: bool) -> Self {
        let hp = if just_pressed {
            ButtonPress::Press
        } else {
            ButtonPress::Hold
        };

        StateInput {
            hp,
            ..Default::default()
        }
    }

    pub fn hp_bits(just_pressed: bool) -> u32 {
        Self::hp(just_pressed).into()
    }

    pub fn lk(just_pressed: bool) -> Self {
        let lk = if just_pressed {
            ButtonPress::Press
        } else {
            ButtonPress::Hold
        };

        StateInput {
            lk,
            ..Default::default()
        }
    }

    pub fn lk_bits(just_pressed: bool) -> u32 {
        Self::lk(just_pressed).into()
    }

    pub fn mk(just_pressed: bool) -> Self {
        let mk = if just_pressed {
            ButtonPress::Press
        } else {
            ButtonPress::Hold
        };

        StateInput {
            mk,
            ..Default::default()
        }
    }

    pub fn mk_bits(just_pressed: bool) -> u32 {
        Self::mk(just_pressed).into()
    }

    pub fn hk(just_pressed: bool) -> Self {
        let hk = if just_pressed {
            ButtonPress::Press
        } else {
            ButtonPress::Hold
        };

        StateInput {
            hk,
            ..Default::default()
        }
    }

    pub fn hk_bits(just_pressed: bool) -> u32 {
        Self::hk(just_pressed).into()
    }

    pub fn left(just_pressed: bool) -> Self {
        StateInput {
            x: DirectionalInput::Negative,
            just_pressed_x: just_pressed,
            ..Default::default()
        }
    }

    pub fn left_bits(just_pressed: bool) -> u32 {
        Self::left(just_pressed).into()
    }

    pub fn right(just_pressed: bool) -> Self {
        StateInput {
            x: DirectionalInput::Positive,
            just_pressed_x: just_pressed,
            ..Default::default()
        }
    }

    pub fn right_bits(just_pressed: bool) -> u32 {
        Self::right(just_pressed).into()
    }

    pub fn up(just_pressed: bool) -> Self {
        StateInput {
            y: DirectionalInput::Positive,
            just_pressed_y: just_pressed,
            ..Default::default()
        }
    }

    pub fn up_bits(just_pressed: bool) -> u32 {
        Self::up(just_pressed).into()
    }

    pub fn down(just_pressed: bool) -> Self {
        StateInput {
            y: DirectionalInput::Negative,
            just_pressed_y: just_pressed,
            ..Default::default()
        }
    }

    pub fn down_bits(just_pressed: bool) -> u32 {
        Self::down(just_pressed).into()
    }
}

impl From<StateInput> for u32 {
    fn from(value: StateInput) -> Self {
        u32::from_le_bytes(value.pack().unwrap())
    }
}

impl From<u32> for StateInput {
    fn from(value: u32) -> Self {
        value.to_le_bytes().into()
    }
}

impl From<&u32> for StateInput {
    fn from(value: &u32) -> Self {
        let value = *value;
        value.into()
    }
}

impl From<[u8; 4]> for StateInput {
    fn from(value: [u8; 4]) -> Self {
        PackedStruct::unpack(&value).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use bevy::reflect::Typed;
    use packed_struct::PackedStruct;
    use serde::Serialize;

    use super::{ButtonPress, StateInput};

    #[test]
    fn something() {
        let input = StateInput {
            x: crate::input::DirectionalInput::None,
            ..Default::default()
        };

        // println!("Input: {}", input);
        // println!("Bits: {}", u32::from(input));

        let packed = input.pack().unwrap();
        let bytes = u32::from_le_bytes(packed);
        let unpacked: StateInput = bytes.into();

        assert_eq!(input, unpacked);

        println!("Size of StateInput is {}", size_of::<StateInput>());
    }

 
}
