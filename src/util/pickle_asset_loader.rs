use bevy::app::{App, Plugin};
use bevy::asset::{AddAsset, Asset, AssetLoader, BoxedFuture, LoadContext, LoadedAsset};
//use serde_json::from_slice;
use serde_pickle::{from_slice, DeOptions};
use std::marker::PhantomData;

pub struct PickleAssetPlugin<A> {
    extensions: Vec<&'static str>,
    _marker: PhantomData<A>
}

impl<A> Plugin for  PickleAssetPlugin<A>
where 
    for <'de> A: serde::Deserialize<'de> + Asset,
{
    fn build(&self, app: &mut App) {
        app.add_asset::<A>().add_asset_loader(PickleAssetLoader::<A> {
            extensions: self.extensions.clone(),
            _marker: PhantomData
        });
    }
}

impl<A> PickleAssetPlugin<A>
where
    for<'de> A: serde::Deserialize<'de> + Asset,
{
    /// Create a new plugin that will load assets from files with the given extensions.
    pub fn new(extensions: &[&'static str]) -> Self {
        Self {
            extensions: extensions.to_owned(),
            _marker: PhantomData,
        }
    }
}


struct PickleAssetLoader<A> {
    extensions: Vec<&'static str>,
    _marker: PhantomData<A>
}

impl<A> AssetLoader for PickleAssetLoader<A>
where 
    for <'de> A: serde::Deserialize<'de> + Asset,
{
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let asset = from_slice::<A>(bytes, DeOptions::default())?;
            load_context.set_default_asset(LoadedAsset::new(asset));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &self.extensions
    }
}
