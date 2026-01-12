pub struct TriangleMesh {
    pub vertices: Vec<f32>,
    pub normals: Vec<f32>,
    pub indices: Vec<u32>,
}

pub struct SurfaceMesh {
    pub mesh: TriangleMesh,
    pub z_min: f32,
    pub z_max: f32,
}

pub struct ParametricSurfaceMesh {
    pub mesh: TriangleMesh,
}

pub struct CurveMesh {
    pub vertices: Vec<f32>,
}
