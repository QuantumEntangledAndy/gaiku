/// Implementation of a naive cubical voxel terrain generation.
mod common;
mod density;
mod discret;
mod tables;

pub use self::{density::DensityVoxelBaker, discret::VoxelBaker};
