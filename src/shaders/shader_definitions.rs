//!#pragma once

pub const WORKGROUP_DIM: u32 = 25;

pub const F90: f32 = 0.04;

pub const GAMMA: f32 = 1.0;

pub const MAX_TEXTURES: u32 = 1024;


pub const USE_PATHTRACER: bool = true;
// PT settings
pub const MAX_BOUNCES: u32 = 10;
pub const SAMPLES_PER_PIXEL: u32 = 2;

pub const SEND_TO_LIGHT_PROBABILITY: f32 = 0.0;

/* -------------------------------------------- */

pub const USE_BIDIRECTIONAL_PATHTRACER: bool = false;
// BDPT settings
pub const BDPT_SAMPLES_PER_PIXEL: u32 = 2;
pub const BDPT_MAX_DEPTH: u32 = 3;