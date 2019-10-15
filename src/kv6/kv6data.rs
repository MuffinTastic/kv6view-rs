use std::io::Seek;
use std::io::SeekFrom;
use std::fs::File;
use std::io::Result;
use byteorder::{LittleEndian, ReadBytesExt};

use cgmath::Vector3;

pub struct KV6Color {
    pub b: u8,
    pub g: u8,
    pub r: u8,
    pub a: u8
}

pub struct KV6Voxel {
    pub color: KV6Color,
    pub z: u16,
    pub visibility: u8,
    pub normal_index: u8,
}

pub struct KV6Data {
    pub size: Vector3<u32>,
    pub pivot: Vector3<f32>,
    pub voxel_count: u32,
    pub voxels: Vec<KV6Voxel>,
    pub x_entries: Vec<u32>,
    pub xy_entries: Vec<u16>
}

pub fn load_kv6(path: &str) -> Result<KV6Data> {
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(4))?;

    let size = Vector3::new(
        file.read_u32::<LittleEndian>()?,
        file.read_u32::<LittleEndian>()?,
        file.read_u32::<LittleEndian>()?
    );

    let pivot = Vector3::new(
        file.read_f32::<LittleEndian>()?,
        file.read_f32::<LittleEndian>()?,
        file.read_f32::<LittleEndian>()?
    );

    let voxel_count = file.read_u32::<LittleEndian>()?;

    let mut voxels = Vec::new();
    for _ in 0..voxel_count {
        voxels.push(KV6Voxel {
            color: KV6Color {
                b: file.read_u8()?,
                g: file.read_u8()?,
                r: file.read_u8()?,
                a: file.read_u8()?,
            },
            z: file.read_u16::<LittleEndian>()?,
            visibility: file.read_u8()?,
            normal_index: file.read_u8()?
        });
    }

    let mut x_entries = Vec::new();
    for _ in 0..size.x {
        x_entries.push(file.read_u32::<LittleEndian>()?);
    }

    let mut xy_entries = Vec::new();
    for _ in 0..size.x*size.y {
        xy_entries.push(file.read_u16::<LittleEndian>()?);
    }

    Ok(KV6Data {
        size,
        pivot,
        voxel_count,
        voxels,
        x_entries,
        xy_entries
    })
}