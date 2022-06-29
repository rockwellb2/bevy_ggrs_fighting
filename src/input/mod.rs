use bevy::render::render_resource::internal::bytemuck;
use bevy::prelude::*;
use bytemuck::{Pod, Zeroable};
use ggrs::PlayerHandle;
use ruwren::Class;
//use bitflags:: bitflags;

// bitflags! {
//     pub struct Buttons: u8 {
//         const LP = 1 << 0;
//         const MP = 1 << 1;
//         const HP = 1 << 2;
//         const LK = 1 << 3;
//         const MK = 1 << 4;
//         const HK = 1 << 5;
//     }

//     pub struct Directionals: u8 {
//         const LEFT = 1 << 0;
//         const RIGHT = 1 << 1;
//         const UP = 1 << 2;
//         const DOWN = 1 << 3;
//     }
// }

// unsafe impl Pod for Buttons {}
// unsafe impl Zeroable for Buttons {}

// unsafe impl Pod for Directionals {}
// unsafe impl Zeroable for Directionals {}

// #[repr(C)]
// #[derive(Debug, Copy, Clone, PartialEq, Pod, Zeroable)]
// pub struct Input {
//     buttons: Buttons,
//     direectionals: Directionals
// }

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

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Pod, Zeroable)]
pub struct Input(pub u16);

impl Class for Input {
    fn initialize(_: &ruwren::VM) -> Self
    where
        Self: Sized {
        todo!()
    }
}



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