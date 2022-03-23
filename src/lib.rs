#![deny(
    warnings,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications,
    missing_docs
)]
#![allow(dead_code)] // TEMP

//! Hanabi -- a particle system plugin for the Bevy game engine.
//!
//! This library provides a particle system for the Bevy game engine.
//!
//! # Example
//!
//! Add the Hanabi plugin to your app:
//!
//! ```no_run
//! # use bevy::prelude::*;
//! # use bevy_hanabi::*;
//! App::default()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugin(HanabiPlugin)
//!     .run();
//! ```
//!
//! Create an EffectAsset describing a visual effect, then add an
//! instance of that effect to an entity:
//!
//! ```
//! # use bevy::prelude::*;
//! # use bevy_hanabi::*;
//! fn setup(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
//!     // Define a color gradient from red to transparent black
//!     let mut gradient = Gradient::new();
//!     gradient.add_key(0.0, Vec4::new(1., 0., 0., 1.));
//!     gradient.add_key(1.0, Vec4::splat(0.));
//!
//!     // Create the effect asset
//!     let effect = effects.add(EffectAsset {
//!         name: "MyEffect".to_string(),
//!         // Maximum number of particles alive at a time
//!         capacity: 32768,
//!         // Spawn at a rate of 5 particles per second
//!         spawner: Spawner::new(SpawnMode::rate(5.)),
//!         ..Default::default()
//!     }
//!     // On spawn, randomly initialize the position and velocity
//!     // of the particle over a sphere of radius 2 units, with a
//!     // radial initial velocity of 6 units/sec away from the
//!     // sphere center.
//!     .init(PositionSphereModifier {
//!         center: Vec3::ZERO,
//!         radius: 2.,
//!         dimension: ShapeDimension::Surface,
//!         speed: 6.,
//!     })
//!     // Every frame, add a gravity-like acceleration downward
//!     .update(AccelModifier {
//!         accel: Vec3::new(0., -3., 0.),
//!     })
//!     // Render the particles with a color gradient over their
//!     // lifetime.
//!     .render(ColorOverLifetimeModifier { gradient })
//!     );
//!
//!     commands
//!         .spawn()
//!         .insert(Name::new("MyEffectInstance"))
//!         .insert_bundle(ParticleEffectBundle {
//!             effect: ParticleEffect::new(effect),
//!             transform: Transform::from_translation(Vec3::new(0., 1., 0.)),
//!             ..Default::default()
//!         });
//! }
//! ```

use bevy::{prelude::*, reflect::TypeUuid};

mod asset;
mod bundle;
mod gradient;
mod modifiers;
mod plugin;
mod render;
mod spawn;

pub use asset::EffectAsset;
pub use bundle::ParticleEffectBundle;
pub use gradient::{Gradient, GradientKey};
pub use modifiers::{
    AccelModifier, ColorOverLifetimeModifier, InitModifier, ParticleTextureModifier, PositionCircleModifier,
    PositionSphereModifier, RenderModifier, ShapeDimension, SizeOverLifetimeModifier,
    UpdateModifier,
};
pub use plugin::HanabiPlugin;
pub use render::EffectCacheId;
pub use spawn::{SpawnCount, SpawnMode, SpawnRate, Spawner, Value};

/// Helper trait to write a floating point number in a format which compiles in WGSL.
///
/// This is required because WGSL doesn't support a floating point constant without
/// a decimal separator (e.g. "0." instead of "0"), which would be what a regular float
/// to string formatting produces.
trait ToWgslFloat {
    /// Convert a floating point value to a string representing a WGSL constant.
    fn to_float_string(&self) -> String;
}

impl ToWgslFloat for f32 {
    fn to_float_string(&self) -> String {
        let s = format!("{:.6}", self);
        s.trim_end_matches("0").to_string()
    }
}

impl ToWgslFloat for f64 {
    fn to_float_string(&self) -> String {
        let s = format!("{:.15}", self);
        s.trim_end_matches("0").to_string()
    }
}

/// Visual effect made of particles.
///
/// The particle effect component represent a single instance of a visual effect. The
/// visual effect itself is described by a handle to an [`EffectAsset`]. This instance
/// is associated to an [`Entity`], inheriting its [`Transform`] as the origin frame
/// for its particle spawning.
#[derive(Debug, Clone, Component, TypeUuid)]
#[uuid = "c48df8b5-7eca-4d25-831e-513c2575cf6c"]
pub struct ParticleEffect {
    /// Handle of the effect to instantiate.
    handle: Handle<EffectAsset>,
    /// Internal effect cache ID of the effect once allocated.
    effect: EffectCacheId,
    /// Particle spawning descriptor.
    spawner: Option<Spawner>,
}

impl ParticleEffect {
    /// Create a new particle effect without a spawner or any modifier.
    pub fn new(handle: Handle<EffectAsset>) -> Self {
        ParticleEffect {
            handle,
            effect: EffectCacheId::INVALID,
            spawner: None,
        }
    }

    /// Configure the spawner of a new particle effect.
    ///
    /// The call returns a reference to the added spawner, allowing to chain
    /// adding modifiers to the effect.
    pub fn spawner(&mut self, spawner: &Spawner) -> &mut Spawner {
        if self.spawner.is_none() {
            self.spawner = Some(spawner.clone());
        }
        self.spawner.as_mut().unwrap()
    }
}
