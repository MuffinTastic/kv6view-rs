use crate::kv6;
use crate::kv6::KV6Data;

use std::io::Result;

use cgmath::Vector3;

#[derive(Debug, Copy, Clone, Default)]
pub struct KV6Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    face: [f32; 3],
    color: [u8; 3]
}
implement_vertex!(KV6Vertex, position, normal, face, color);

mod legacy {
    use std::f32::consts::PI;
    use cgmath::Vector3;
    use cgmath::Zero;

    const GOLDRAT: f32 = 0.3819660112501052;
    const LUT_POINTS: usize = 255;
    const ZMULK: f32 = 2.0 / LUT_POINTS as f32;
    const ZADDK: f32 = ZMULK * 0.5 - 1.0;

    pub fn create_normal_table() -> Vec<Vector3<f32>> {
        let mut table = Vec::new();

        for i in 0..LUT_POINTS + 1 {
            let mut vec = Vector3::new(0.0, 0.0, i as f32 * ZMULK + ZADDK);
            let g = i as f32 * (GOLDRAT * PI * 2.0);
            let r = (1.0 - vec.z * vec.z).sqrt();
            vec.x = -g.cos()*r; // flip X for compat
            vec.y = g.sin()*r;
            vec.z *= -1.0;
            table.push(vec);
        }
        
        table[LUT_POINTS] = Vector3::zero();
        
        table
    }
}

const LEFT_VISIBLE: u8 = 1;
const RIGHT_VISIBLE: u8 = 2;
const BACK_VISIBLE: u8 = 4;
const FRONT_VISIBLE: u8 = 8;
const TOP_VISIBLE: u8 = 16;
const BOTTOM_VISIBLE: u8 = 32;

fn kv6_gen_vertices(data: &KV6Data) -> Vec<KV6Vertex> {
    let mut vertices = Vec::new();
    let normal_table = legacy::create_normal_table();

    let mut vox_index = 0;
    for x in 0..data.size.x {
        for y in 0..data.size.y {
            for _ in 0..data.xy_entries[(x * data.size.y + y) as usize] {
                let voxel = &data.voxels[vox_index];
                let z = voxel.z;

                let vox_pos = Vector3::new(
                    -(x as f32 - data.pivot.x),   // set center of the model to the pivot
                    y as f32 - data.pivot.y,      // and flip model axes for compatibility with worldspace
                    -(z as f32) - data.pivot.z 
                );

                // TODO: find a way to simplify/automate this process more by generating vertices?

                let mut emit_face = |face: [f32; 3], v1: Vector3<f32>, v2: Vector3<f32>, v3: Vector3<f32>, v4: Vector3<f32>| {
                    let mut vertex = KV6Vertex {
                        normal: normal_table[voxel.normal_index as usize].into(),
                        color: [voxel.color.r, voxel.color.g, voxel.color.b],
                        face,
                        .. Default::default()
                    };
                    vertex.position = (vox_pos + v1).into(); vertices.push(vertex);
                    vertex.position = (vox_pos + v2).into(); vertices.push(vertex);
                    vertex.position = (vox_pos + v3).into(); vertices.push(vertex);
                    vertex.position = (vox_pos + v3).into(); vertices.push(vertex);
                    vertex.position = (vox_pos + v4).into(); vertices.push(vertex);
                    vertex.position = (vox_pos + v1).into(); vertices.push(vertex);
                };

                if voxel.visibility & FRONT_VISIBLE > 0 {
                    emit_face( [0.0, 1.0, 0.0],
                        Vector3::new(-0.5, 0.5, -0.5),
                        Vector3::new(-0.5, 0.5,  0.5),
                        Vector3::new( 0.5, 0.5,  0.5),
                        Vector3::new( 0.5, 0.5, -0.5)
                    );
                }

                if voxel.visibility & BACK_VISIBLE > 0 {
                    emit_face( [0.0, -1.0, 0.0],
                        Vector3::new(-0.5, -0.5, -0.5),
                        Vector3::new( 0.5, -0.5, -0.5),
                        Vector3::new( 0.5, -0.5,  0.5),
                        Vector3::new(-0.5, -0.5,  0.5)
                    );
                }

                if voxel.visibility & TOP_VISIBLE > 0 {
                    emit_face( [0.0, 0.0, 1.0],
                        Vector3::new(-0.5, -0.5,  0.5),
                        Vector3::new( 0.5, -0.5,  0.5),
                        Vector3::new( 0.5,  0.5,  0.5),
                        Vector3::new(-0.5,  0.5,  0.5)
                    );
                }

                if voxel.visibility & BOTTOM_VISIBLE > 0 {
                    emit_face( [0.0, 0.0, -1.0],
                        Vector3::new(-0.5, -0.5, -0.5),
                        Vector3::new(-0.5,  0.5, -0.5),
                        Vector3::new( 0.5,  0.5, -0.5),
                        Vector3::new( 0.5, -0.5, -0.5)
                    );
                }

                if voxel.visibility & RIGHT_VISIBLE > 0 {
                    emit_face( [-1.0, 0.0, 0.0],
                        Vector3::new(-0.5, -0.5, -0.5),
                        Vector3::new(-0.5, -0.5,  0.5),
                        Vector3::new(-0.5,  0.5,  0.5),
                        Vector3::new(-0.5,  0.5, -0.5)
                    );
                }

                if voxel.visibility & LEFT_VISIBLE > 0 {
                    emit_face( [1.0, 0.0, 0.0],
                        Vector3::new( 0.5, -0.5, -0.5),
                        Vector3::new( 0.5,  0.5, -0.5),
                        Vector3::new( 0.5,  0.5,  0.5),
                        Vector3::new( 0.5, -0.5,  0.5)
                    );
                }

                vox_index += 1;
            }
        }
    }

    vertices
}

pub struct KV6Mesh {
    pub vertex_buffer: glium::VertexBuffer<KV6Vertex>,
    pub indices: glium::index::NoIndices,
}

impl KV6Mesh {
    pub fn from_data(data: KV6Data, display: &glium::Display) -> KV6Mesh {
        let vertices = kv6_gen_vertices(&data);

        KV6Mesh {
            vertex_buffer: glium::VertexBuffer::new(&*display, &vertices).unwrap(),
            indices: glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList)
        }
    }

    pub fn from_file(path: &str, display: &glium::Display) -> Result<KV6Mesh> {
        let data = kv6::load_kv6(path)?;
        Ok(KV6Mesh::from_data(data, display))
    }
}