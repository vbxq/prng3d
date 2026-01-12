use aelys::{Value, get_function, new_vm, run_with_vm};
use crossbeam::channel::{self, Receiver, Sender};
use parking_lot::Mutex;
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use crate::math::mesh::{CurveMesh, ParametricSurfaceMesh, SurfaceMesh, TriangleMesh};

pub enum MathCommand {
    CompileSurface {
        code: String,
        x_range: (f64, f64),
        y_range: (f64, f64),
        resolution: usize,
    },
    CompileParametricCurve {
        code: String,
        t_range: (f64, f64),
        samples: usize,
    },
    CompileParametricSurface {
        code: String,
        u_range: (f64, f64),
        v_range: (f64, f64),
        u_samples: usize,
        v_samples: usize,
    },
    Stop,
}

pub enum MathResult {
    Surface(SurfaceMesh),
    ParametricCurve(CurveMesh),
    ParametricSurface(ParametricSurfaceMesh),
    Error(String),
}

pub struct MathEngine {
    tx_cmd: Sender<MathCommand>,
    rx_result: Receiver<MathResult>,
    last_error: Arc<Mutex<Option<String>>>,
    thread_handle: Option<JoinHandle<()>>,
}

impl MathEngine {
    pub fn new() -> Self {
        let (tx_cmd, rx_cmd) = channel::unbounded::<MathCommand>();
        let (tx_result, rx_result) = channel::bounded::<MathResult>(2);
        let last_error = Arc::new(Mutex::new(None));
        let last_error_clone = Arc::clone(&last_error);

        let thread_handle = thread::spawn(move || {
            math_thread(rx_cmd, tx_result, last_error_clone);
        });

        Self {
            tx_cmd,
            rx_result,
            last_error,
            thread_handle: Some(thread_handle),
        }
    }

    pub fn compile_surface(
        &self,
        code: &str,
        x_range: (f64, f64),
        y_range: (f64, f64),
        resolution: usize,
    ) {
        let _ = self.tx_cmd.send(MathCommand::CompileSurface {
            code: code.to_string(),
            x_range,
            y_range,
            resolution,
        });
    }

    pub fn compile_parametric_curve(&self, code: &str, t_range: (f64, f64), samples: usize) {
        let _ = self.tx_cmd.send(MathCommand::CompileParametricCurve {
            code: code.to_string(),
            t_range,
            samples,
        });
    }

    pub fn compile_parametric_surface(
        &self,
        code: &str,
        u_range: (f64, f64),
        v_range: (f64, f64),
        u_samples: usize,
        v_samples: usize,
    ) {
        let _ = self.tx_cmd.send(MathCommand::CompileParametricSurface {
            code: code.to_string(),
            u_range,
            v_range,
            u_samples,
            v_samples,
        });
    }

    pub fn try_recv_result(&self) -> Option<MathResult> {
        self.rx_result.try_recv().ok()
    }

    pub fn last_error(&self) -> Option<String> {
        self.last_error.lock().clone()
    }

    pub fn stop(&self) {
        let _ = self.tx_cmd.send(MathCommand::Stop);
    }
}

impl Drop for MathEngine {
    fn drop(&mut self) {
        let _ = self.tx_cmd.send(MathCommand::Stop);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

fn math_thread(
    rx_cmd: Receiver<MathCommand>,
    tx_result: Sender<MathResult>,
    last_error: Arc<Mutex<Option<String>>>,
) {
    loop {
        let cmd = match rx_cmd.recv() {
            Ok(c) => c,
            Err(_) => return,
        };

        match cmd {
            MathCommand::CompileSurface {
                code,
                x_range,
                y_range,
                resolution,
            } => {
                *last_error.lock() = None;

                match compile_and_sample_surface(&code, x_range, y_range, resolution) {
                    Ok(mesh) => {
                        let _ = tx_result.send(MathResult::Surface(mesh));
                    }
                    Err(e) => {
                        *last_error.lock() = Some(e.clone());
                        let _ = tx_result.send(MathResult::Error(e));
                    }
                }
            }
            MathCommand::CompileParametricCurve {
                code,
                t_range,
                samples,
            } => {
                *last_error.lock() = None;

                match compile_and_sample_parametric(&code, t_range, samples) {
                    Ok(mesh) => {
                        let _ = tx_result.send(MathResult::ParametricCurve(mesh));
                    }
                    Err(e) => {
                        *last_error.lock() = Some(e.clone());
                        let _ = tx_result.send(MathResult::Error(e));
                    }
                }
            }
            MathCommand::CompileParametricSurface {
                code,
                u_range,
                v_range,
                u_samples,
                v_samples,
            } => {
                *last_error.lock() = None;

                match compile_and_sample_parametric_surface(
                    &code, u_range, v_range, u_samples, v_samples,
                ) {
                    Ok(mesh) => {
                        let _ = tx_result.send(MathResult::ParametricSurface(mesh));
                    }
                    Err(e) => {
                        *last_error.lock() = Some(e.clone());
                        let _ = tx_result.send(MathResult::Error(e));
                    }
                }
            }
            MathCommand::Stop => return,
        }
    }
}

fn compile_and_sample_surface(
    code: &str,
    x_range: (f64, f64),
    y_range: (f64, f64),
    resolution: usize,
) -> Result<SurfaceMesh, String> {
    let mut vm = new_vm().map_err(|e| format!("VM init error: {}", e))?;

    let full_code = format!("needs std.math;\n{}", code);
    run_with_vm(&mut vm, &full_code, "math_surface").map_err(|e| format!("{}", e))?;

    let func = get_function(&vm, "f").map_err(|e| format!("{}", e))?;

    if func.arity() != 2 {
        return Err(format!(
            "Function 'f' must take 2 arguments (x, y), got {}",
            func.arity()
        ));
    }

    let mut vertices = Vec::with_capacity(resolution * resolution * 3);
    let mut normals = Vec::with_capacity(resolution * resolution * 3);
    let mut indices = Vec::new();

    let dx = (x_range.1 - x_range.0) / (resolution - 1) as f64;
    let dy = (y_range.1 - y_range.0) / (resolution - 1) as f64;

    let mut z_values = vec![vec![0.0f64; resolution]; resolution];
    let mut z_min = f64::MAX;
    let mut z_max = f64::MIN;

    for i in 0..resolution {
        for j in 0..resolution {
            let x = x_range.0 + i as f64 * dx;
            let y = y_range.0 + j as f64 * dy;

            let result = func
                .call(&mut vm, &[Value::float(x), Value::float(y)])
                .map_err(|e| format!("Evaluation error at ({}, {}): {}", x, y, e))?;

            let z = result
                .as_float()
                .unwrap_or_else(|| result.as_int().unwrap_or(0) as f64);
            z_values[i][j] = z;

            if z.is_finite() {
                z_min = z_min.min(z);
                z_max = z_max.max(z);
            }
        }
    }

    let z_range = (z_max - z_min).max(0.001);
    let scale = 100.0 / z_range;
    let z_offset = (z_min + z_max) / 2.0;

    for i in 0..resolution {
        for j in 0..resolution {
            let x = x_range.0 + i as f64 * dx;
            let y = y_range.0 + j as f64 * dy;
            let z = z_values[i][j];

            let scaled_x = (x / (x_range.1 - x_range.0).abs().max(0.001)) * 200.0;
            let scaled_y = (y / (y_range.1 - y_range.0).abs().max(0.001)) * 200.0;
            let scaled_z = if z.is_finite() {
                (z - z_offset) * scale
            } else {
                0.0
            };

            vertices.push(scaled_x as f32);
            vertices.push(scaled_z as f32);
            vertices.push(scaled_y as f32);

            let nx = if i > 0 && i < resolution - 1 {
                (z_values[i + 1][j] - z_values[i - 1][j]) / (2.0 * dx)
            } else {
                0.0
            };
            let ny = if j > 0 && j < resolution - 1 {
                (z_values[i][j + 1] - z_values[i][j - 1]) / (2.0 * dy)
            } else {
                0.0
            };

            let len = (nx * nx + ny * ny + 1.0).sqrt();
            normals.push((-nx / len) as f32);
            normals.push((1.0 / len) as f32);
            normals.push((-ny / len) as f32);
        }
    }

    for i in 0..resolution - 1 {
        for j in 0..resolution - 1 {
            let tl = (i * resolution + j) as u32;
            let tr = (i * resolution + j + 1) as u32;
            let bl = ((i + 1) * resolution + j) as u32;
            let br = ((i + 1) * resolution + j + 1) as u32;

            indices.push(tl);
            indices.push(bl);
            indices.push(tr);

            indices.push(tr);
            indices.push(bl);
            indices.push(br);
        }
    }

    Ok(SurfaceMesh {
        mesh: TriangleMesh {
            vertices,
            normals,
            indices,
        },
        z_min: z_min as f32,
        z_max: z_max as f32,
    })
}

fn compile_and_sample_parametric(
    code: &str,
    t_range: (f64, f64),
    samples: usize,
) -> Result<CurveMesh, String> {
    let mut vm = new_vm().map_err(|e| format!("VM init error: {}", e))?;

    let full_code = format!("needs std.math;\n{}", code);
    run_with_vm(&mut vm, &full_code, "math_parametric").map_err(|e| format!("{}", e))?;

    let func_x = get_function(&vm, "fx").map_err(|e| format!("fx: {}", e))?;
    let func_y = get_function(&vm, "fy").map_err(|e| format!("fy: {}", e))?;
    let func_z = get_function(&vm, "fz").map_err(|e| format!("fz: {}", e))?;

    let mut vertices = Vec::with_capacity(samples * 3);
    let dt = (t_range.1 - t_range.0) / (samples - 1) as f64;

    for i in 0..samples {
        let t = t_range.0 + i as f64 * dt;
        let t_val = Value::float(t);

        let x = func_x
            .call(&mut vm, &[t_val])
            .map_err(|e| format!("fx error: {}", e))?
            .as_float()
            .unwrap_or(0.0);
        let y = func_y
            .call(&mut vm, &[t_val])
            .map_err(|e| format!("fy error: {}", e))?
            .as_float()
            .unwrap_or(0.0);
        let z = func_z
            .call(&mut vm, &[t_val])
            .map_err(|e| format!("fz error: {}", e))?
            .as_float()
            .unwrap_or(0.0);

        vertices.push((x * 50.0) as f32);
        vertices.push((y * 50.0) as f32);
        vertices.push((z * 50.0) as f32);
    }

    Ok(CurveMesh { vertices })
}

fn compile_and_sample_parametric_surface(
    code: &str,
    u_range: (f64, f64),
    v_range: (f64, f64),
    u_samples: usize,
    v_samples: usize,
) -> Result<ParametricSurfaceMesh, String> {
    let mut vm = new_vm().map_err(|e| format!("VM init error: {}", e))?;

    let full_code = format!("needs std.math;\n{}", code);
    run_with_vm(&mut vm, &full_code, "math_parametric_surface").map_err(|e| format!("{}", e))?;

    let func_x = get_function(&vm, "fx").map_err(|e| format!("fx: {}", e))?;
    let func_y = get_function(&vm, "fy").map_err(|e| format!("fy: {}", e))?;
    let func_z = get_function(&vm, "fz").map_err(|e| format!("fz: {}", e))?;

    if func_x.arity() != 2 || func_y.arity() != 2 || func_z.arity() != 2 {
        return Err("Functions fx, fy, fz must each take 2 arguments (u, v)".to_string());
    }

    let du = (u_range.1 - u_range.0) / (u_samples - 1) as f64;
    let dv = (v_range.1 - v_range.0) / (v_samples - 1) as f64;

    let mut positions = vec![vec![(0.0f64, 0.0f64, 0.0f64); v_samples]; u_samples];

    for i in 0..u_samples {
        for j in 0..v_samples {
            let u = u_range.0 + i as f64 * du;
            let v = v_range.0 + j as f64 * dv;

            let x = func_x
                .call(&mut vm, &[Value::float(u), Value::float(v)])
                .map_err(|e| format!("fx error at ({}, {}): {}", u, v, e))?
                .as_float()
                .unwrap_or(0.0);
            let y = func_y
                .call(&mut vm, &[Value::float(u), Value::float(v)])
                .map_err(|e| format!("fy error at ({}, {}): {}", u, v, e))?
                .as_float()
                .unwrap_or(0.0);
            let z = func_z
                .call(&mut vm, &[Value::float(u), Value::float(v)])
                .map_err(|e| format!("fz error at ({}, {}): {}", u, v, e))?
                .as_float()
                .unwrap_or(0.0);

            positions[i][j] = (x, y, z);
        }
    }

    let mut vertices = Vec::with_capacity(u_samples * v_samples * 3);
    let mut normals = Vec::with_capacity(u_samples * v_samples * 3);

    for i in 0..u_samples {
        for j in 0..v_samples {
            let (x, y, z) = positions[i][j];

            let tangent_u = if i > 0 && i < u_samples - 1 {
                let p_plus = positions[i + 1][j];
                let p_minus = positions[i - 1][j];
                (
                    (p_plus.0 - p_minus.0) / (2.0 * du),
                    (p_plus.1 - p_minus.1) / (2.0 * du),
                    (p_plus.2 - p_minus.2) / (2.0 * du),
                )
            } else if i == 0 {
                let p_next = positions[i + 1][j];
                (
                    (p_next.0 - x) / du,
                    (p_next.1 - y) / du,
                    (p_next.2 - z) / du,
                )
            } else {
                let p_prev = positions[i - 1][j];
                (
                    (x - p_prev.0) / du,
                    (y - p_prev.1) / du,
                    (z - p_prev.2) / du,
                )
            };

            let tangent_v = if j > 0 && j < v_samples - 1 {
                let p_plus = positions[i][j + 1];
                let p_minus = positions[i][j - 1];
                (
                    (p_plus.0 - p_minus.0) / (2.0 * dv),
                    (p_plus.1 - p_minus.1) / (2.0 * dv),
                    (p_plus.2 - p_minus.2) / (2.0 * dv),
                )
            } else if j == 0 {
                let p_next = positions[i][j + 1];
                (
                    (p_next.0 - x) / dv,
                    (p_next.1 - y) / dv,
                    (p_next.2 - z) / dv,
                )
            } else {
                let p_prev = positions[i][j - 1];
                (
                    (x - p_prev.0) / dv,
                    (y - p_prev.1) / dv,
                    (z - p_prev.2) / dv,
                )
            };

            let nx = tangent_u.1 * tangent_v.2 - tangent_u.2 * tangent_v.1;
            let ny = tangent_u.2 * tangent_v.0 - tangent_u.0 * tangent_v.2;
            let nz = tangent_u.0 * tangent_v.1 - tangent_u.1 * tangent_v.0;
            let len = (nx * nx + ny * ny + nz * nz).sqrt().max(0.0001);

            vertices.push((x * 50.0) as f32);
            vertices.push((y * 50.0) as f32);
            vertices.push((z * 50.0) as f32);

            normals.push((nx / len) as f32);
            normals.push((ny / len) as f32);
            normals.push((nz / len) as f32);
        }
    }

    let mut indices = Vec::new();
    for i in 0..u_samples - 1 {
        for j in 0..v_samples - 1 {
            let tl = (i * v_samples + j) as u32;
            let tr = (i * v_samples + j + 1) as u32;
            let bl = ((i + 1) * v_samples + j) as u32;
            let br = ((i + 1) * v_samples + j + 1) as u32;

            indices.push(tl);
            indices.push(bl);
            indices.push(tr);

            indices.push(tr);
            indices.push(bl);
            indices.push(br);
        }
    }

    Ok(ParametricSurfaceMesh {
        mesh: TriangleMesh {
            vertices,
            normals,
            indices,
        },
    })
}
