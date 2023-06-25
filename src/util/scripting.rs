use std::sync::Mutex;

use bevy::prelude::{info, Entity, Query, With};
use bevy_mod_scripting::{lua::api::bevy::LuaEntity, prelude::*};

use crate::fighter::Fighter;

pub fn testing_this(
    query: Query<Entity, With<Fighter>>,
    mut events: PriorityEventWriter<LuaEvent<PlayerEntityArg>>,
) {
    for entity in query.iter() {
        events.send(
            LuaEvent {
                hook_name: "help".to_owned(),
                args: PlayerEntityArg(entity),
                recipients: Recipients::All,
            },
            0,
        )
    }
}

#[derive(Default)]
pub struct LuaAPIProvider;

impl APIProvider for LuaAPIProvider {
    type APITarget = Mutex<Lua>;
    type DocTarget = LuaDocFragment;
    type ScriptContext = Mutex<Lua>;

    fn attach_api(&mut self, ctx: &mut Self::APITarget) -> Result<(), ScriptError> {
        // callbacks can receive any `ToLuaMulti` arguments, here '()' and
        // return any `FromLuaMulti` arguments, here a `usize`
        // check the Rlua documentation for more details

        let ctx = ctx.get_mut().unwrap();

        ctx.globals()
            .set(
                "print",
                ctx.create_function(|_ctx, msg: String| {
                    println!("{}", msg);
                    Ok(())
                })
                .map_err(ScriptError::new_other)?,
            )
            .map_err(ScriptError::new_other)?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct PlayerEntityArg(Entity);

impl<'lua> ToLua<'lua> for PlayerEntityArg {
    fn to_lua(self, lua: &'lua Lua) -> mlua::Result<Value<'lua>> {
        LuaEntity::new(self.0).to_lua(lua)
    }
}
