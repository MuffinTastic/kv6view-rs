use glium;

pub mod legacy {
    use std::io;
    use std::io::Read;
    use std::io::Seek;
    use std::io::SeekFrom;
    use std::mem;
    use std::slice;
    use std::fs::File;

    use cgmath::Vector3;

    #[repr(C, packed)]
    #[derive(Debug)]
    pub struct RawRGBA {
        pub b: u8,
        pub g: u8,
        pub r: u8,
        pub a: u8
    }

    #[repr(C, packed)]
    #[derive(Debug)]
    pub struct RawVOXType {
        pub color: RawRGBA,
        pub z: u16,
        pub visibility: u8,
        pub normal_index: u8,
    }

    #[repr(C, packed)]
    #[derive(Debug)]
    struct RawKV6Data {
        x_size: u32,
        y_size: u32,
        z_size: u32,
        x_piv: f32,
        y_piv: f32,
        z_piv: f32,
        voxel_count: u32
    }

    // Rust-safe type
    #[derive(Debug)]
    pub struct KV6Data {
        pub x_size: u32,
        pub y_size: u32,
        pub z_size: u32,
        pub x_piv: f32,
        pub y_piv: f32,
        pub z_piv: f32,
        pub voxel_count: u32,
        pub vox: Vec<RawVOXType>,
        pub x_entries: Vec<u32>,
        pub xy_entries: Vec<u16>
    }

    // this function is what happens when you become very lazy, if i ever become un-lazy i'll redo this whole source file
    pub fn load_kv6(path: &str) -> Result<KV6Data, io::Error> {
        let mut file = File::open(path)?;
        file.seek(SeekFrom::Start(4))?;

        let mut raw_data: RawKV6Data = unsafe { mem::zeroed() };
        let raw_size = mem::size_of::<RawKV6Data>();

        unsafe {
            let raw_slice = slice::from_raw_parts_mut(
                &mut raw_data as *mut _ as *mut u8,
                raw_size
            );

            file.read_exact(raw_slice)?;
        }

        let mut vox = Vec::with_capacity(raw_data.voxel_count as usize);
        for _ in 0..raw_data.voxel_count {
            let mut raw_data: RawVOXType = unsafe { mem::zeroed() };
            let raw_size = mem::size_of::<RawVOXType>();
            unsafe {
                let raw_slice = slice::from_raw_parts_mut(
                    &mut raw_data as *mut _ as *mut u8,
                    raw_size
                );

                file.read_exact(raw_slice)?;
            }
            vox.push(raw_data);
        }

        let mut x_entries = Vec::with_capacity(raw_data.x_size as usize);
        for _ in 0..raw_data.x_size {
            let mut raw_data: u32 = unsafe { mem::zeroed() };
            let raw_size = mem::size_of::<u32>();

            unsafe {
                let raw_slice = slice::from_raw_parts_mut(
                    &mut raw_data as *mut _ as *mut u8,
                    raw_size
                );

                file.read_exact(raw_slice)?;
            }
            x_entries.push(raw_data);
        }

        let mut xy_entries = Vec::with_capacity(raw_data.x_size as usize * raw_data.y_size as usize);
        for _ in 0..raw_data.x_size * raw_data.y_size {
            let mut raw_data: u16 = unsafe { mem::zeroed() };
            let raw_size = mem::size_of::<u16>();

            unsafe {
                let raw_slice = slice::from_raw_parts_mut(
                    &mut raw_data as *mut _ as *mut u8,
                    raw_size
                );

                file.read_exact(raw_slice)?;
            }
            xy_entries.push(raw_data);
        }

        Ok(KV6Data {
            x_size: raw_data.x_size,
            y_size: raw_data.y_size,
            z_size: raw_data.z_size,
            x_piv: raw_data.x_piv,
            y_piv: raw_data.y_piv,
            z_piv: raw_data.z_piv,
            voxel_count: raw_data.voxel_count,
            vox,
            x_entries,
            xy_entries
        })
    }

    use cgmath::Zero;
    use std::f32::consts::PI;
    const GOLDRAT: f32 = 0.3819660112501052;
    const LUT_POINTS: usize = 255;
    const ZMULK: f32 = 2.0 / LUT_POINTS as f32;
    const ZADDK: f32 = ZMULK * 0.5 - 1.0;

    fn create_normal_table() -> Vec<Vector3<f32>> {
        let ind_to_vec = |i| -> Vector3<f32> {
            let mut vec = Vector3::new(0.0, 0.0, i as f32 * ZMULK + ZADDK);
            let g = i as f32 * (GOLDRAT * PI * 2.0);
            let r = (1.0 - vec.z * vec.z).sqrt();
            vec.x = g.cos()*r;
            vec.y = g.sin()*r;
            vec.z *= -1.0;
            vec
        };

        let mut table: Vec<Vector3<f32>> = (0..LUT_POINTS+1).map(ind_to_vec).collect();
        table[LUT_POINTS] = Vector3::zero();
        table
    }

    #[derive(Debug, Copy, Clone)]
    pub struct KV6Vertex {
        position: [f32; 3],
        normal: [f32; 3],
        face: [f32; 3],
        color: [u8; 3]
    }
    implement_vertex!(KV6Vertex, position, normal, face, color);

    const LEFT_VISIBLE: u8 = 1;
    const RIGHT_VISIBLE: u8 = 2;
    const BACK_VISIBLE: u8 = 4;
    const FRONT_VISIBLE: u8 = 8;
    const TOP_VISIBLE: u8 = 16;
    const BOTTOM_VISIBLE: u8 = 32;

    pub fn kv6_get_vertices(data: &KV6Data) -> Vec<KV6Vertex> {
        let mut vertices = Vec::new();
        let normal_table = create_normal_table();

        let mut i = 0;
        for x in 0..data.x_size {
            for y in 0..data.y_size {
                for _ in 0..data.xy_entries[x as usize * data.y_size as usize + y as usize] {
                    let point = &data.vox[i];
                    let z = point.z;
                    let gl = Vector3::new(
                        x as f32 - data.x_piv,
                        y as f32 - data.y_piv,
                        -(z as f32) - data.z_piv
                    );
                    let normal = normal_table[point.normal_index as usize];
                    let mut vertex = KV6Vertex {
                        position: [0.0, 0.0, 0.0],
                        normal: normal.into(),
                        face: [0.0, 0.0, 0.0],
                        color: [point.color.r, point.color.g, point.color.b]
                    };
                    if point.visibility & FRONT_VISIBLE > 0 {
                        vertex.face = [0.0, 1.0, 0.0];
                        vertex.position = (gl + Vector3::new(-0.5, 0.5, -0.5)).into(); vertices.push(vertex);
                        vertex.position = (gl + Vector3::new(-0.5, 0.5,  0.5)).into(); vertices.push(vertex);
                        vertex.position = (gl + Vector3::new( 0.5, 0.5,  0.5)).into(); vertices.push(vertex);
                        vertex.position = (gl + Vector3::new( 0.5, 0.5,  0.5)).into(); vertices.push(vertex);
                        vertex.position = (gl + Vector3::new( 0.5, 0.5, -0.5)).into(); vertices.push(vertex);
                        vertex.position = (gl + Vector3::new(-0.5, 0.5, -0.5)).into(); vertices.push(vertex);
                    }
                    if point.visibility & BACK_VISIBLE > 0 {
                        vertex.face = [0.0, -1.0, 0.0];
                        vertex.position = (gl + Vector3::new(-0.5, -0.5, -0.5)).into(); vertices.push(vertex);
                        vertex.position = (gl + Vector3::new( 0.5, -0.5, -0.5)).into(); vertices.push(vertex);
                        vertex.position = (gl + Vector3::new( 0.5, -0.5,  0.5)).into(); vertices.push(vertex);
                        vertex.position = (gl + Vector3::new( 0.5, -0.5,  0.5)).into(); vertices.push(vertex);
                        vertex.position = (gl + Vector3::new(-0.5, -0.5,  0.5)).into(); vertices.push(vertex);
                        vertex.position = (gl + Vector3::new(-0.5, -0.5, -0.5)).into(); vertices.push(vertex);
                    }
                    if point.visibility & TOP_VISIBLE > 0 {
                        vertex.face = [0.0, 0.0, 1.0];
                        vertex.position = (gl + Vector3::new(-0.5, -0.5,  0.5)).into(); vertices.push(vertex);
                        vertex.position = (gl + Vector3::new( 0.5, -0.5,  0.5)).into(); vertices.push(vertex);
                        vertex.position = (gl + Vector3::new( 0.5,  0.5,  0.5)).into(); vertices.push(vertex);
                        vertex.position = (gl + Vector3::new( 0.5,  0.5,  0.5)).into(); vertices.push(vertex);
                        vertex.position = (gl + Vector3::new(-0.5,  0.5,  0.5)).into(); vertices.push(vertex);
                        vertex.position = (gl + Vector3::new(-0.5, -0.5,  0.5)).into(); vertices.push(vertex);
                    }
                    if point.visibility & BOTTOM_VISIBLE > 0 {
                        vertex.face = [0.0, 0.0, -1.0];
                        vertex.position = (gl + Vector3::new(-0.5, -0.5, -0.5)).into(); vertices.push(vertex);
                        vertex.position = (gl + Vector3::new(-0.5,  0.5, -0.5)).into(); vertices.push(vertex);
                        vertex.position = (gl + Vector3::new( 0.5,  0.5, -0.5)).into(); vertices.push(vertex);
                        vertex.position = (gl + Vector3::new( 0.5,  0.5, -0.5)).into(); vertices.push(vertex);
                        vertex.position = (gl + Vector3::new( 0.5, -0.5, -0.5)).into(); vertices.push(vertex);
                        vertex.position = (gl + Vector3::new(-0.5, -0.5, -0.5)).into(); vertices.push(vertex);
                    }
                    if point.visibility & RIGHT_VISIBLE > 0 {
                        vertex.face = [1.0, 0.0, 0.0];
                        vertex.position = (gl + Vector3::new( 0.5, -0.5, -0.5)).into(); vertices.push(vertex);
                        vertex.position = (gl + Vector3::new( 0.5,  0.5, -0.5)).into(); vertices.push(vertex);
                        vertex.position = (gl + Vector3::new( 0.5,  0.5,  0.5)).into(); vertices.push(vertex);
                        vertex.position = (gl + Vector3::new( 0.5,  0.5,  0.5)).into(); vertices.push(vertex);
                        vertex.position = (gl + Vector3::new( 0.5, -0.5,  0.5)).into(); vertices.push(vertex);
                        vertex.position = (gl + Vector3::new( 0.5, -0.5, -0.5)).into(); vertices.push(vertex);
                    }
                    if point.visibility & LEFT_VISIBLE > 0 {
                        vertex.face = [-1.0, 0.0, 0.0];
                        vertex.position = (gl + Vector3::new(-0.5, -0.5, -0.5)).into(); vertices.push(vertex);
                        vertex.position = (gl + Vector3::new(-0.5, -0.5,  0.5)).into(); vertices.push(vertex);
                        vertex.position = (gl + Vector3::new(-0.5,  0.5,  0.5)).into(); vertices.push(vertex);
                        vertex.position = (gl + Vector3::new(-0.5,  0.5,  0.5)).into(); vertices.push(vertex);
                        vertex.position = (gl + Vector3::new(-0.5,  0.5, -0.5)).into(); vertices.push(vertex);
                        vertex.position = (gl + Vector3::new(-0.5, -0.5, -0.5)).into(); vertices.push(vertex);
                    }
                    i += 1;
                }
            }
        }

        vertices
    }
}

pub struct KV6Model {
    pub vertex_buffer: glium::VertexBuffer<legacy::KV6Vertex>,
    pub indices: glium::index::NoIndices,
}

impl KV6Model {
    pub fn from_data(data: legacy::KV6Data, display: &glium::Display) -> KV6Model {
        let vertices = legacy::kv6_get_vertices(&data);

        KV6Model {
            vertex_buffer: glium::VertexBuffer::new(&*display, &vertices).unwrap(),
            indices: glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList)
        }
    }

    pub fn from_file(path: &str, display: &glium::Display) -> Result<KV6Model, std::io::Error> {
        let data = legacy::load_kv6(path)?;
        Ok(KV6Model::from_data(data, display))
    }
}