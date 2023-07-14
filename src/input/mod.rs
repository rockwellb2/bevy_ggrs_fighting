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

use crate::fighter::state::{Facing, Direction, Frame};
use crate::fighter::systems::InputBuffer;
use crate::util::Buffer;

pub const BUFFER_SIZE: usize = 60;
pub const BUFFER_TIME: Frame = 0;


pub const LEFT_HELD: u32 = 2048;
pub const LEFT: u32 = 2560;

pub const RIGHT_HELD: u32 = 1024;
pub const RIGHT: u32 = 1536;



#[repr(C)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Pod, Zeroable, Reflect, FromReflect)]
pub struct Input(pub u32);

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum Action {
    A,
    B,
    C,
    J,
    K,
    L,
    Left,
    Right,
    Up,
    Down,
}

pub fn input(
    handle: In<PlayerHandle>,
    mut input_query: Query<(&ActionState<Action>, &Facing)>,
    mut local: Local<(StateInput, StateInput)>
) -> Input 
{
    if handle.0 == 1 {
        return Input(StateInput::default().into());
    }

    

    if let Ok((action_state, facing)) = input_query.get_single_mut() {
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

        let lp = button_check(Action::A);
        let mp = button_check(Action::B);
        let hp = button_check(Action::C);
        let lk = button_check(Action::J);
        let mk = button_check(Action::K);
        let hk = button_check(Action::L);

        let (x, just_pressed_x) = directional_check(Action::Right, Action::Left);
        let (y, just_pressed_y) = directional_check(Action::Up, Action::Down);

        // if facing.0 == Direction::Left {
        //     x = match x {
        //         DirectionalInput::None => DirectionalInput::None,
        //         DirectionalInput::Positive => DirectionalInput::Negative,
        //         DirectionalInput::Negative => DirectionalInput::Positive,
        //     }
        // }

        let inp = StateInput::new(lp, mp, hp, lk, mk, hk, x, just_pressed_x, y, just_pressed_y);

        local.0 = inp.clone();

        let inp: u32 = inp.into();
        //buffer.0.insert(inp);
        Input(inp)
    }
    else {
        Input(StateInput::default().into())
    }
}


#[derive(Debug, Serialize, Deserialize, FromReflect, Reflect, Clone)]
#[serde(untagged)]
pub enum MatchExpression {
    Button(String, ButtonPress),
    Directional(String, DirectionalInput, bool),
}

impl Default for MatchExpression {
    fn default() -> Self {
        Self::Button("a".to_string(), ButtonPress::None)
    }
}

#[derive(Debug, Serialize, Deserialize, Default, FromReflect, Reflect, Clone)]
pub struct CommandInput {
    list: Vec<Vec<MatchExpression>>,
    #[serde(default = "CommandInput::window_default")]
    window: Frame,
    #[serde(default = "CommandInput::buffer_time_default", alias = "bufferTime")]
    buffer_time: Frame
}

impl CommandInput {
    fn buffer_time_default() -> Frame {
        BUFFER_TIME
    }

    fn window_default() -> Frame {
        1
    }

    pub fn compare(&self, input: &Buffer, facing: Direction) -> bool {
        let mut input_iter = input.iter();
        let mut index = 0;

        let mut buffer_time = 0;

        // iterate over each match expression
        for command in &self.list {
            
            loop {

                index += 1;
                if index > self.window {
                    return false;
                }

                if let Some(next) = input_iter.next() {
                    let mut next: StateInput = next.into();

                    if facing == Direction::Left {
                        match next.x {
                            DirectionalInput::None => {},
                            DirectionalInput::Positive => {
                                next.x = DirectionalInput::Negative;
                            },
                            DirectionalInput::Negative =>{
                                next.x = DirectionalInput::Positive;
                            },
                        }
                    }


                    //let mut command_iter = command.iter();
                    let mut same = true;
                    for expression in command {
                        if !next.compare_command(expression.clone()) {
                            if index == 1 {
                                if self.buffer_time > buffer_time {
                                    index = 0;
                                    buffer_time += 1;
                                }
                                else {
                                    return false;
                                }
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
                
                // if self.buffer_time > buffer_time {
                //     index = 0;
                //     buffer_time += 1;
                // }
            }
        }

        true
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

#[derive(PrimitiveEnum_u8, Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize, Reflect, FromReflect)]
pub enum ButtonPress {
    #[default]
    None = 0,
    Press = 1,
    Hold = 2,
    Release = 3,

    // These are only used as part of command expressions
    PressOrRelease
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

#[derive(PrimitiveEnum_u8, Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize, Reflect, FromReflect)]
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
    pub a: ButtonPress,
    #[serde(default)]
    #[packed_field(bits = "2..=3", ty = "enum")]
    pub b: ButtonPress,
    #[serde(default)]
    #[packed_field(bits = "4..=5", ty = "enum")]
    pub c: ButtonPress,
    #[serde(default)]
    #[packed_field(bits = "6..=7", ty = "enum")]
    pub j: ButtonPress,
    #[serde(default)]
    #[packed_field(bits = "8..=9", ty = "enum")]
    pub k: ButtonPress,
    #[serde(default)]
    #[packed_field(bits = "10..=11", ty = "enum")]
    pub l: ButtonPress,
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
    #![allow(clippy::too_many_arguments)]
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
            a: lp,
            b: mp,
            c: hp,
            j: lk,
            k: mk,
            l: hk,
            x,
            just_pressed_x,
            y,
            just_pressed_y,
            _reserved: ReservedZero::default(),
        }
    }

    pub fn get_button_from_action(&self, action: Action) -> ButtonPress {
        match action {
            Action::A => self.a,
            Action::B => self.b,
            Action::C => self.c,
            Action::J => self.j,
            Action::K => self.k,
            Action::L => self.l,
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

    pub fn compare_command(&self, command: MatchExpression) -> bool {
        match command {
            MatchExpression::Button(name, button) => {
                let i = match name.as_str() {
                    "a" => self.a,
                    "b" => self.b,
                    "c" => self.c,
                    "lk" => self.j,
                    "mk" => self.k,
                    "hk" => self.l,
                    _ => panic!(),
                };

                i == button
            }
            MatchExpression::Directional(name, direction, just_pressed) => {
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
            a: lp,
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
            b: mp,
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
            c: hp,
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
            j: lk,
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
            k: mk,
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
            l: hk,
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


