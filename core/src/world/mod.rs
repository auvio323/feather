use crate::world::block::*;
use crate::world::chunk::Chunk;
use glm::{DVec3, Vec3};
use hashbrown::HashMap;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use rayon::iter::ParallelIterator;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Sub};
use std::sync::Arc;

pub mod block;
#[allow(clippy::cast_lossless)]
pub mod chunk;

#[macro_export]
macro_rules! position {
    ($x:expr, $y:expr, $z:expr, $pitch:expr, $yaw:expr, $on_ground:expr) => {
        $crate::Position {
            x: $x,
            y: $y,
            z: $z,
            pitch: $pitch,
            yaw: $yaw,
            on_ground: $on_ground,
        }
    };
    ($x:expr, $y:expr, $z:expr, $pitch: expr, $yaw: expr) => {
        position!($x, $y, $z, $pitch, $yaw, true)
    };
    ($x:expr, $y:expr, $z:expr) => {
        position!($x, $y, $z, 0.0, 0.0)
    };
    ($x:expr, $y:expr, $z:expr, $on_ground: expr) => {
        position!($x, $y, $z, 0.0, 0.0, $on_ground)
    };
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub pitch: f32,
    pub yaw: f32,
    pub on_ground: bool,
}

impl Position {
    pub fn distance(&self, other: Position) -> f64 {
        self.distance_squared(other).sqrt()
    }

    pub fn distance_squared(&self, other: Position) -> f64 {
        square(self.x - other.x) + square(self.y - other.y) + square(self.z - other.z)
    }

    /// Returns the position of the chunk
    /// this position is in.
    pub fn chunk_pos(&self) -> ChunkPosition {
        ChunkPosition::new(self.x.floor() as i32 / 16, self.z.floor() as i32 / 16)
    }

    /// Retrieves the position of the block
    /// this position is in.
    pub fn block_pos(&self) -> BlockPosition {
        BlockPosition::new(
            self.x.floor() as i32,
            self.y.floor() as i32,
            self.z.floor() as i32,
        )
    }

    /// Returns a unit vector representing
    /// the direction of this position's pitch
    /// and yaw.
    pub fn direction(&self) -> DVec3 {
        let rotation_x = f64::from(self.yaw.to_radians());
        let rotation_y = f64::from(self.pitch.to_radians());

        let y = -rotation_y.sin();

        let xz = rotation_y.cos();

        let x = -xz * rotation_x.sin();
        let z = xz * rotation_x.cos();

        glm::vec3(x, y, z)
    }

    pub fn as_vec(&self) -> DVec3 {
        (*self).into()
    }
}

impl Add<Vec3> for Position {
    type Output = Position;

    fn add(mut self, vec: Vec3) -> Self::Output {
        self.x += f64::from(vec.x);
        self.y += f64::from(vec.y);
        self.z += f64::from(vec.z);
        self
    }
}

impl Add<DVec3> for Position {
    type Output = Position;

    fn add(mut self, rhs: DVec3) -> Self::Output {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
        self
    }
}

impl Add<Position> for Position {
    type Output = Position;

    fn add(mut self, rhs: Position) -> Self::Output {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
        self.pitch += rhs.pitch;
        self.yaw += rhs.yaw;
        self
    }
}

impl Sub<Vec3> for Position {
    type Output = Position;

    fn sub(mut self, vec: Vec3) -> Self::Output {
        self.x -= f64::from(vec.x);
        self.y -= f64::from(vec.y);
        self.z -= f64::from(vec.z);
        self
    }
}

impl Sub<DVec3> for Position {
    type Output = Position;

    fn sub(mut self, vec: DVec3) -> Self::Output {
        self.x -= vec.x;
        self.y -= vec.y;
        self.z -= vec.z;
        self
    }
}

impl Sub<Position> for Position {
    type Output = Position;

    fn sub(mut self, rhs: Position) -> Self::Output {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
        self
    }
}

impl Into<Vec3> for Position {
    fn into(self) -> Vec3 {
        glm::vec3(self.x as f32, self.y as f32, self.z as f32)
    }
}

impl Into<DVec3> for Position {
    fn into(self) -> DVec3 {
        glm::vec3(self.x, self.y, self.z)
    }
}

impl From<Vec3> for Position {
    fn from(vec: Vec3) -> Self {
        position!(f64::from(vec.x), f64::from(vec.y), f64::from(vec.z))
    }
}

impl From<DVec3> for Position {
    fn from(vec: DVec3) -> Self {
        position!(vec.x, vec.y, vec.z)
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "({:.2}, {:.2}, {:.2}), ({:.2}, {:.2}), on_ground: {}",
            self.x, self.y, self.z, self.pitch, self.yaw, self.on_ground
        )
    }
}

fn square(x: f64) -> f64 {
    x * x
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Hash32, Default)]
pub struct ChunkPosition {
    pub x: i32,
    pub z: i32,
}

impl ChunkPosition {
    pub const fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    /// Computes the Manhattan distance from this chunk to another.
    pub fn manhattan_distance(self, other: ChunkPosition) -> i32 {
        (self.x - other.z).abs() + (self.z - other.z).abs()
    }
}

impl Display for ChunkPosition {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "({}, {})", self.x, self.z)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Hash32, Default, new)]
pub struct BlockPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPosition {
    pub fn chunk_pos(&self) -> ChunkPosition {
        ChunkPosition::new(self.x >> 4, self.z >> 4)
    }

    pub fn world_pos(&self) -> Position {
        position!(f64::from(self.x), f64::from(self.y), f64::from(self.z))
    }
}

impl Add<BlockPosition> for BlockPosition {
    type Output = BlockPosition;

    fn add(mut self, rhs: BlockPosition) -> Self::Output {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
        self
    }
}

pub type ChunkMapInner = HashMap<ChunkPosition, Arc<RwLock<Chunk>>>;

/// The chunk map.
///
/// This struct stores all the chunks on the server,
/// so it allows access to blocks and lighting data.
///
/// Chunks are internally wrapped in `Arc<RwLock>`,
/// allowing multiple systems to access different parts
/// of the world in parallel. Mutable access to this
/// type is only required for inserting and removing
/// chunks.
pub struct ChunkMap(ChunkMapInner);

impl ChunkMap {
    /// Retrieves a handle to the chunk at the given
    /// position, or `None` if it is not loaded.
    pub fn chunk_at(&self, pos: ChunkPosition) -> Option<RwLockReadGuard<Chunk>> {
        self.0.get(&pos).map(|lock| lock.read())
    }

    /// Retrieves a handle to the chunk at the given
    /// position, or `None` if it is not loaded.
    pub fn chunk_at_mut(&self, pos: ChunkPosition) -> Option<RwLockWriteGuard<Chunk>> {
        self.0.get(&pos).map(|lock| lock.write())
    }

    /// Retrieves the block at the given position,
    /// or `None` if its chunk is not loaded.
    pub fn block_at(&self, pos: BlockPosition) -> Option<Block> {
        let (x, y, z) = chunk_relative_pos(pos);
        self.chunk_at(pos.chunk_pos())
            .map(|chunk| chunk.block_at(x, y, z))
    }

    /// Sets the block at the given position.
    ///
    /// Returns `true` if the block was set, or `false`
    /// if its chunk was not loaded and thus no operation
    /// was performed.
    pub fn set_block_at(&self, pos: BlockPosition, block: Block) -> bool {
        let (x, y, z) = chunk_relative_pos(pos);

        self.chunk_at_mut(pos.chunk_pos())
            .map(|mut chunk| chunk.set_block_at(x, y, z, block))
            .is_ok()
    }

    /// Returns an iterator over chunks.
    pub fn iter_chunks(&self) -> impl IntoIterator<Item = &Arc<RwLock<Chunk>>> {
        self.0.iter()
    }

    /// Returns a parallel iterator over chunks.
    pub fn par_iter_chunks(&self) -> impl ParallelIterator<Item = &Arc<RwLock<Chunk>>> {
        self.0.par_iter()
    }

    /// Inserts a new chunk into the chunk map.
    pub fn insert(&mut self, chunk: Chunk) {
        self.0
            .insert(chunk.position(), Arc::new(RwLock::new(chunk)));
    }

    /// Removes the chunk at the given position, returning `true` if it existed.
    pub fn remove(&mut self, pos: ChunkPosition) -> bool {
        self.0.remove(&pos).is_some()
    }
}

impl Default for ChunkMap {
    fn default() -> Self {
        Self::new()
    }
}

fn chunk_relative_pos(block_pos: BlockPosition) -> (usize, usize, usize) {
    (
        block_pos.x as usize & 0xf,
        block_pos.y as usize,
        block_pos.z as usize & 0xf,
    )
}

pub trait ChunkGenerator {
    fn generate(&self, chunk: &mut Chunk);
}

pub struct FlatChunkGenerator {}

impl ChunkGenerator for FlatChunkGenerator {
    fn generate(&self, chunk: &mut Chunk) {
        for x in 0..16 {
            for y in 0..64 {
                for z in 0..16 {
                    chunk.set_block_at(x, y, z, Block::Stone);
                }
            }
        }
    }
}

pub struct GridChunkGenerator {}

impl ChunkGenerator for GridChunkGenerator {
    fn generate(&self, chunk: &mut Chunk) {
        for x in 0..15 {
            for y in 0..64 {
                for z in 0..15 {
                    chunk.set_block_at(x, y, z, Block::Stone);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_map() {
        let mut world = ChunkMap::new();

        let chunk = world.chunk_at(ChunkPosition::new(0, 0));
        if chunk.is_some() {
            panic!();
        }

        let mut chunk = Chunk::new(ChunkPosition::new(0, 0));
        FlatChunkGenerator {}.generate(&mut chunk);
        world.chunk_map.insert(ChunkPosition::new(0, 0), chunk);

        let chunk = world.chunk_at(ChunkPosition::new(0, 0)).unwrap();

        for x in 0..15 {
            for y in 0..64 {
                for z in 0..15 {
                    assert_eq!(chunk.block_at(x, y, z), Block::Stone);
                }
            }
        }

        assert_eq!(chunk.block_at(8, 64, 8), Block::Air);
    }

    #[test]
    fn test_set_block_at() {
        let mut world = ChunkMap::new();

        let mut chunk = Chunk::new(ChunkPosition::new(0, 0));
        GridChunkGenerator {}.generate(&mut chunk);
        world.chunk_map.insert(ChunkPosition::new(0, 0), chunk);

        println!("-----");
        world
            .set_block_at(BlockPosition::new(1, 63, 1), Block::Air)
            .unwrap();

        println!("-----");
        assert_eq!(
            world.block_at(BlockPosition::new(1, 63, 1)).unwrap(),
            Block::Air
        );
    }
}
