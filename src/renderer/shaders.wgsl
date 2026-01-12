struct CameraUniform {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    _padding: f32,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) point_size: f32,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let world_pos = vec4<f32>(in.position, 1.0);
    out.clip_position = camera.view_proj * world_pos;

    let dist = distance(camera.camera_pos, in.position);
    out.point_size = clamp(8.0 / (dist * 0.1 + 1.0), 1.0, 8.0);

    let normalized = (in.position + vec3<f32>(1.0)) * 0.5;
    out.color = vec3<f32>(
        normalized.x * 0.6 + 0.2,
        normalized.y * 0.4 + 0.3,
        normalized.z * 0.8 + 0.2
    );

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}

struct Vertex2DInput {
    @location(0) position: vec2<f32>,
    @location(1) value: f32,
}

struct Vertex2DOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

@vertex
fn vs_2d_main(in: Vertex2DInput) -> Vertex2DOutput {
    var out: Vertex2DOutput;

    out.clip_position = vec4<f32>(in.position, 0.0, 1.0);

    let v = in.value;
    out.color = vec3<f32>(
        v * 0.8 + 0.1,
        v * 0.5 + 0.2,
        v * 0.9 + 0.1
    );

    return out;
}

@fragment
fn fs_2d_main(in: Vertex2DOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}

struct SurfaceVertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

struct SurfaceVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) height: f32,
}

struct SurfaceUniforms {
    z_min: f32,
    z_max: f32,
    _pad1: f32,
    _pad2: f32,
}

@group(0) @binding(1)
var<uniform> surface_uniforms: SurfaceUniforms;

fn viridis(t: f32) -> vec3<f32> {
    let c0 = vec3<f32>(0.267, 0.004, 0.329);
    let c1 = vec3<f32>(0.282, 0.140, 0.457);
    let c2 = vec3<f32>(0.254, 0.265, 0.529);
    let c3 = vec3<f32>(0.191, 0.407, 0.556);
    let c4 = vec3<f32>(0.127, 0.566, 0.550);
    let c5 = vec3<f32>(0.267, 0.678, 0.480);
    let c6 = vec3<f32>(0.478, 0.821, 0.318);
    let c7 = vec3<f32>(0.741, 0.873, 0.150);
    let c8 = vec3<f32>(0.993, 0.906, 0.144);

    let s = clamp(t, 0.0, 1.0) * 8.0;
    let i = floor(s);
    let f = fract(s);

    if i < 1.0 { return mix(c0, c1, f); }
    if i < 2.0 { return mix(c1, c2, f); }
    if i < 3.0 { return mix(c2, c3, f); }
    if i < 4.0 { return mix(c3, c4, f); }
    if i < 5.0 { return mix(c4, c5, f); }
    if i < 6.0 { return mix(c5, c6, f); }
    if i < 7.0 { return mix(c6, c7, f); }
    return mix(c7, c8, f);
}

@vertex
fn vs_surface_main(in: SurfaceVertexInput) -> SurfaceVertexOutput {
    var out: SurfaceVertexOutput;

    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.world_pos = in.position;
    out.normal = normalize(in.normal);
    out.height = in.position.y;

    return out;
}

@fragment
fn fs_surface_main(in: SurfaceVertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let view_dir = normalize(camera.camera_pos - in.world_pos);
    let normal = normalize(in.normal);

    let ambient = 0.15;
    let diffuse = max(dot(normal, light_dir), 0.0) * 0.7;

    let half_dir = normalize(light_dir + view_dir);
    let specular = pow(max(dot(normal, half_dir), 0.0), 32.0) * 0.3;

    let z_range = surface_uniforms.z_max - surface_uniforms.z_min;
    let t = clamp((in.height - surface_uniforms.z_min) / max(z_range, 0.001), 0.0, 1.0);
    let base_color = viridis(t);

    let lighting = ambient + diffuse;
    let final_color = base_color * lighting + vec3<f32>(specular);

    return vec4<f32>(final_color, 1.0);
}

struct CurveVertexInput {
    @location(0) position: vec3<f32>,
}

struct CurveVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) t_value: f32,
}

@vertex
fn vs_curve_main(in: CurveVertexInput, @builtin(vertex_index) idx: u32) -> CurveVertexOutput {
    var out: CurveVertexOutput;

    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);

    let total_verts = 1000.0;
    out.t_value = f32(idx) / total_verts;

    return out;
}

@fragment
fn fs_curve_main(in: CurveVertexOutput) -> @location(0) vec4<f32> {
    let color = viridis(in.t_value);
    return vec4<f32>(color, 1.0);
}

struct GridVertexInput {
    @location(0) position: vec3<f32>,
}

struct GridVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_grid_main(in: GridVertexInput) -> GridVertexOutput {
    var out: GridVertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);

    if abs(in.position.x) < 0.01 {
        out.color = vec4<f32>(0.2, 0.8, 0.2, 0.8);
    } else if abs(in.position.z) < 0.01 {
        out.color = vec4<f32>(0.8, 0.2, 0.2, 0.8);
    } else if abs(in.position.y) < 0.01 {
        out.color = vec4<f32>(0.2, 0.2, 0.8, 0.8);
    } else {
        out.color = vec4<f32>(0.3, 0.3, 0.3, 0.4);
    }

    return out;
}

@fragment
fn fs_grid_main(in: GridVertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}

struct Math2DVertexInput {
    @location(0) position: vec2<f32>,
    @location(1) value: f32,
}

struct Math2DVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

@vertex
fn vs_math_2d_main(in: Math2DVertexInput) -> Math2DVertexOutput {
    var out: Math2DVertexOutput;
    out.clip_position = vec4<f32>(in.position, 0.0, 1.0);
    out.color = viridis(in.value);
    return out;
}

@fragment
fn fs_math_2d_main(in: Math2DVertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}

struct Curve2DVertexInput {
    @location(0) position: vec2<f32>,
}

struct Curve2DVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) t_value: f32,
}

@vertex
fn vs_curve_2d_main(in: Curve2DVertexInput, @builtin(vertex_index) idx: u32) -> Curve2DVertexOutput {
    var out: Curve2DVertexOutput;
    out.clip_position = vec4<f32>(in.position, 0.0, 1.0);
    out.t_value = f32(idx) / 1000.0;
    return out;
}

@fragment
fn fs_curve_2d_main(in: Curve2DVertexOutput) -> @location(0) vec4<f32> {
    let color = viridis(in.t_value);
    return vec4<f32>(color, 1.0);
}
