use std::{
    io::{self, ErrorKind},
    marker::PhantomData,
    string,
    sync::{Mutex, RwLock, RwLockReadGuard},
};

use bevy::{
    asset::AssetLoader,
    prelude::{info, App, AppTypeRegistry, Entity, FromWorld, Query, ReflectComponent, Vec3, With},
    reflect::{
        serde::{ReflectSerializer, UntypedReflectDeserializer},
        DynamicStruct, Reflect, ReflectDeserialize, ReflectOwned, TypeRegistration, TypeRegistry,
        TypeRegistryArc, TypeRegistryInternal,
    },
    DefaultPlugins, ecs::query::WorldQuery,
};
use bevy_mod_scripting::{
    api::{common::bevy::ScriptWorld, wrappers::ScriptReference, ScriptRef},
    lua::api::{
        bevy::{LuaEntity, LuaVec3},
        ValueLuaType,
    },
    prelude::{
        mlua::{UserDataMethods, Variadic},
        tealr::mlu::{mlua::UserData, MaybeSend, UserDataProxy},
        *,
    },
};
//use crate::util::scripting::mlua::UserDataMethods;
use bevy::reflect::erased_serde::Serialize;
use serde::{
    de::{DeserializeSeed, Error, Expected, Unexpected},
    Deserialize, Deserializer,
};
//use serde::Deserialize;

use crate::fighter::{
    state::{CurrentState, SerializedState, Velocity, Conditions, Variables},
    Fighter,
};

pub fn testing_this(
    query: Query<Entity, With<Fighter>>,
    mut events: PriorityEventWriter<LuaEvent<PlayerEntityArg>>,
) {
    for entity in query.iter() {
        events.send(
            LuaEvent {
                hook_name: "help".to_owned(),
                args: PlayerEntityArg::new(entity),
                recipients: Recipients::All,
            },
            0,
        )
    }
}

pub fn send_process_event_system(
    query: Query<(Entity, &CurrentState), With<Fighter>>,
    mut events: PriorityEventWriter<LuaEvent<PlayerEntityArg>>,
) {
    for (player, state) in query.iter() {
        events.send(
            LuaEvent {
                hook_name: "process".to_owned(),
                args: PlayerEntityArg::new(player),
                recipients: Recipients::Entity(state.0),
            },
            2,
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
        //ctx.create_proxy()

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

        // ctx.globals()
        //     .set(
        //         "set",
        //         ctx.create_function_mut(|_ctx, values: LuaMultiValue| {
        //             values.pop_front().
        //             Ok(())

        //         })
        //         .map_err(ScriptError::new_other)?,
        // ).map_err(ScriptError::new_other)?;

        //ctx.globals().set(key, value)

        // ctx.globals()
        //     .set(
        //         "set_velocity",
        //         ctx.create_function(|_ctx, player: PlayerEntityArg| {

        //             Ok(())
        //         })
        //         .map_err(ScriptError::new_other)?,
        //     )
        //     .map_err(ScriptError::new_other)?;

        Ok(())
    }
}

// pub trait ScriptWorldExtension {
//     fn attempting_something(&self);
// }

// impl ScriptWorldExtension for ScriptWorld {
//     fn attempting_something(&self) {
//         println!("I am attempting something here, don't know if it will work")
//     }
// }

#[derive(Clone)]
pub struct PlayerEntityArg {
    pub entity: Entity,
}

impl PlayerEntityArg {
    pub fn new(entity: Entity) -> Self {
        PlayerEntityArg { entity }
    }
}

impl From<Entity> for PlayerEntityArg {
    fn from(value: Entity) -> Self {
        PlayerEntityArg::new(value)
    }
}

impl<'lua> ToLua<'lua> for PlayerEntityArg {
    fn to_lua(self, lua: &'lua Lua) -> mlua::Result<Value<'lua>> {
        LuaEntity::new(self.entity).to_lua(lua)
    }
}

impl<'lua> FromLuaMulti<'lua> for PlayerEntityArg {
    fn from_lua_multi(mut values: LuaMultiValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        let value = values.pop_front().ok_or(mlua::Error::BindError)?;
        let entity: LuaEntity = FromLua::from_lua(value, lua)?;
        Ok(entity.inner()?.into())
    }
}

// impl<'lua> FromLuaMulti<'lua> for Velocity {
//     fn from_lua_multi(mut values: LuaMultiValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
//         let value = values.pop_front().ok_or(mlua::Error::BindError)?;
//         // let entity: LuaEntity = FromLua::from_lua(value, lua)?;
//         // Ok(entity.inner()?.into())

//         let x: Velocity = FromLua::from_lua(value, lua)?;
//     }
// }

// impl UserData for Velocity {
//     fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
//         methods.add_method_mut("set", |_, s, (x, y, z): (f32, f32, f32)| {
//             //s.0 = Vec3::new(x, y, z);
//             s.0.x = x;
//             s.0.y = y;
//             s.0.z = z;
//             Ok(())
//         });
//     }

//     fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
//         fields.add_field_method_get("x", |_, s| {
//             Ok(s.0.x)
//         });

//         fields.add_field_method_set("x", |_, this, val| {
//             this.0.x = val;
//             Ok(())
//         });
//     }
// }

// impl ValueLuaType for Velocity {}

// impl Velocity {
//     pub fn crazy(&mut self, x: f32, y: f32, z: f32) {
//         self.0 = Vec3::new(x, y, z);
//     }
// }

// impl_script_newtype! (
//     #[languages(on_feature(lua))]
//     Velocity:
//         Clone + Debug
//         + Fields(

//         )
//         + Methods(
//             crazy(&mut self:Raw(f32),Raw(f32),Raw(f32))
//         )

//     lua impl {

//     }
// );

impl bevy::reflect::Struct for Velocity {
    fn field(&self, name: &str) -> Option<&dyn Reflect> {
        match name {
            "x" => Some(&self.0.x),
            "y" => Some(&self.0.y),
            "z" => Some(&self.0.z),
            _ => None,
        }
    }

    fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect> {
        match name {
            "x" => Some(&mut self.0.x),
            "y" => Some(&mut self.0.y),
            "z" => Some(&mut self.0.z),
            _ => None,
        }
    }

    fn field_at(&self, index: usize) -> Option<&dyn Reflect> {
        match index {
            0 => Some(&self.0.x),
            1 => Some(&self.0.y),
            2 => Some(&self.0.z),
            _ => None,
        }
    }

    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        match index {
            0 => Some(&mut self.0.x),
            1 => Some(&mut self.0.y),
            2 => Some(&mut self.0.z),
            _ => None,
        }
    }

    fn name_at(&self, index: usize) -> Option<&str> {
        match index {
            0 => Some("x"),
            1 => Some("y"),
            2 => Some("z"),
            _ => None,
        }
    }

    fn field_len(&self) -> usize {
        3
    }

    fn iter_fields(&self) -> bevy::reflect::FieldIter {
        todo!()
    }

    fn clone_dynamic(&self) -> DynamicStruct {
        todo!()
    }
}

pub enum Trigger {
    SetComponent {
        component: ReflectComponent,
        value: DynamicStruct,
    },
    SetVariable {
        variable: String,
        value: Box<dyn Reflect>,
    },
    GetVariable {
        variable: String
    },
    Add(f32),
    Sub(f32),
    Mul(f32),
    Div(f32)
}

impl Trigger {
    pub fn evaluate(&self, context: Option<Box<dyn Reflect>>, components: &VariableQuery) -> Option<Box<dyn Reflect>> {
        match self {
            Self::GetVariable { variable } => components.variables.get(variable),
            Self::Add(number) => {
                let context: f32 = *context
                    .expect("No context value")
                    .downcast()
                    .expect("Couldn't downcast to f32");

                Some(Box::new(number + context))
            },
            Self::Sub(number) => {
                let context: f32 = *context
                    .expect("No context value")
                    .downcast()
                    .expect("Couldn't downcast to f32");

                Some(Box::new(context - number))
            },
            Self::Mul(number) => {
                let context: f32 = *context
                    .expect("No context value")
                    .downcast()
                    .expect("Couldn't downcast to f32");

                Some(Box::new(number * context))
            },
            Self::Div(number) => {
                let context: f32 = *context
                    .expect("No context value")
                    .downcast()
                    .expect("Couldn't downcast to f32");

                Some(Box::new(context / number))
            },

            _ => None
            
        }
    }
}

#[derive(WorldQuery)]
pub struct VariableQuery {
    velocity: &'static  Velocity,
    variables: &'static Variables
}

pub struct IfThenTrigger {
    condition: Conditions,
    then: Trigger,
    or_else: Trigger
}

// impl<T: Reflect> Perform<T> {
//     fn thing(&self, registry: &TypeRegistry) -> TypeRegistration {
//         match self {
//             Perform::SetComponent { component, value } => todo!(),
//             Perform::SetDifferent { value, _type } => {
//                 //T::

//             },
//         }

//     }

// }

pub struct CustomAssetLoader {
    registry: AppTypeRegistry,
}

impl FromWorld for CustomAssetLoader {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let registry = world.resource::<AppTypeRegistry>().clone();

        CustomAssetLoader { registry }
    }
}

impl AssetLoader for CustomAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let registry = self.registry.read();

            let list: Vec<ron::Value> = ron::de::from_bytes(bytes).expect("Not a Vec<Value>");

            for value in list {
                if let ron::Value::Map(mut map) = value {
                    let id: u16 = map
                        .remove(&ron::Value::String("id".to_string()))
                        .and_then(|v| v.into_rust().ok())
                        .unwrap_or_default();

                    let name: String = map
                        .remove(&ron::Value::String("name".to_string()))
                        .or_else(|| map.remove(&ron::Value::String("debug_name".to_string())))
                        .and_then(|v| v.into_rust().ok())
                        .unwrap_or("State".to_owned());

                    let duration: Option<u16> = map
                        .remove(&ron::Value::String("duration".to_string()))
                        .and_then(|v| v.into_rust().ok());

                    let transitions: Vec<u16> = map
                        .remove(&ron::Value::String("transitions".to_string()))
                        .and_then(|v| v.into_rust().ok())
                        .unwrap_or(vec![0]);

                    let switch: (Option<Vec<Conditions>>, Vec<Vec<Conditions>>) = map
                        .remove(&ron::Value::String("switch".to_string()))
                        .and_then(|v| v.into_rust().ok())
                        .unwrap_or((None, Vec::new()));


                }
            }

            let value: serde_json::Value = serde_json::de::from_slice(bytes)
                .map_err(|error| bevy::asset::Error::new(error))?;
            let value = value.as_object().ok_or_else(|| {
                bevy::asset::Error::from(serde_json::Error::invalid_type(
                    Unexpected::Other("not an object"),
                    &"object",
                ))
            })?;
            let component = value
                .get("component")
                .expect("Doesn't contain 'component' key")
                .as_str()
                .expect("Not a string");

            let registration = registry
                .get_with_short_name(component)
                .expect("Type not registered");
            let reflect_component = registration
                .data::<ReflectComponent>()
                .expect("Couldn't get ReflectComponent for type")
                .clone();

            let mut dynamic = DynamicStruct::default();
            let map = value
                .get("value")
                .expect("Doesn't contain 'value' key")
                .as_object()
                .expect("Not an object");

            for (k, v) in map {
                convert_value_to_struct_field(&mut dynamic, k, v)?;
            }

            let trigger = Trigger::SetComponent {
                component: reflect_component,
                value: dynamic,
            };

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        todo!()
    }
}

#[derive(Reflect)]
pub struct Thingy {
    component: String,
    value: DynamicStruct,
}

#[test]
fn reflect_thing() -> Result<(), serde_json::Error> {
    let mut app = App::new();
    app.register_type::<Velocity>()
        .register_type::<Thingy>()
        .register_type::<Vec3>();

    let dynamic = Vec3::new(11., 2., -1.);

    let registry = app.world.resource::<AppTypeRegistry>().read();

    let serializer = ReflectSerializer::new(&dynamic, &registry);
    let string = serde_json::to_string(&serializer).expect("Couldn't serialize");

    println!("{}", string);

    let json = r#"{
        "component": "Velocity",
        "value": {
            "x": 10
        }
    }"#;

    //app.update();

    let value: serde_json::Value = serde_json::de::from_str(json)?;
    let object = value.as_object().unwrap();
    let component = object.get("component").unwrap().as_str().unwrap();
    let x = object.get("value").unwrap().as_object().unwrap();

    //registry.get_type_data(type_id)

    println!("{x:?}");

    //let registry = app.world.resource::<AppTypeRegistry>().read();

    let reflect_deserializer = UntypedReflectDeserializer::new(&registry);

    let mut deserializer = serde_json::Deserializer::from_str(&string);

    let reflect_value = reflect_deserializer.deserialize(&mut deserializer)?;

    let x = reflect_value.downcast_ref::<DynamicStruct>().unwrap();

    let ron = r#"
        (thing: "greg", magic: 7)
    "#;

    let magic: ron::Value = ron::from_str(ron).expect("Couldn't convert from ron");
    if let ron::Value::Map(mut map) = magic {
        let w = map
            .remove(&ron::Value::String(String::from("thing")))
            .unwrap();
        println!("The value of thing is {w:?}");
    }

    Ok(())
}

pub fn convert_value_to_struct_field(
    dynamic: &mut DynamicStruct,
    key: &String,
    value: &serde_json::Value,
) -> Result<(), serde_json::Error> {
    match value {
        serde_json::Value::Null => Err(serde_json::Error::invalid_type(
            Unexpected::Other("null"),
            &"valid value",
        )),
        serde_json::Value::Bool(b) => Ok(dynamic.insert(key, *b)),
        serde_json::Value::Number(number) => {
            if let Some(unsigned) = number.as_u64() {
                Ok(dynamic.insert(key, unsigned))
            } else if let Some(signed) = number.as_i64() {
                Ok(dynamic.insert(key, signed))
            } else {
                Ok(dynamic.insert(key, number.as_f64().unwrap()))
            }
        }
        serde_json::Value::String(s) => Ok(dynamic.insert(key, s.clone())),
        serde_json::Value::Array(_) => todo!(),
        serde_json::Value::Object(_) => todo!(),
    }
}
