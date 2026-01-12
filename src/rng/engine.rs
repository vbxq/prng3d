use aelys::{CallableFunction, VM, Value, get_function, new_vm, run_with_vm};
use crossbeam::channel::{self, Receiver, Sender, TrySendError};
use parking_lot::Mutex;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, AtomicUsize, Ordering};
use std::thread::{self, JoinHandle};

const TARGET_BATCH_TIME_MS: f32 = 5.0;
const MIN_BATCH_SIZE: usize = 1_000;
const MAX_BATCH_SIZE: usize = 500_000;
const CHANNEL_CAPACITY: usize = 4;

pub struct AtomicBounds {
    pub min_x: AtomicI64,
    pub max_x: AtomicI64,
    pub min_y: AtomicI64,
    pub max_y: AtomicI64,
    pub min_z: AtomicI64,
    pub max_z: AtomicI64,
}

impl Default for AtomicBounds {
    fn default() -> Self {
        Self {
            min_x: AtomicI64::new(-500),
            max_x: AtomicI64::new(500),
            min_y: AtomicI64::new(-500),
            max_y: AtomicI64::new(500),
            min_z: AtomicI64::new(-500),
            max_z: AtomicI64::new(500),
        }
    }
}

impl AtomicBounds {
    pub fn set(&self, min_x: i64, max_x: i64, min_y: i64, max_y: i64, min_z: i64, max_z: i64) {
        self.min_x.store(min_x, Ordering::Relaxed);
        self.max_x.store(max_x, Ordering::Relaxed);
        self.min_y.store(min_y, Ordering::Relaxed);
        self.max_y.store(max_y, Ordering::Relaxed);
        self.min_z.store(min_z, Ordering::Relaxed);
        self.max_z.store(max_z, Ordering::Relaxed);
    }
}

#[inline(always)]
fn normalize_value(value: i64, min: i64, max: i64) -> f32 {
    let v = value.abs() % (max - min).max(1);
    (min + v) as f32
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Bottleneck {
    CpuRng,
    GpuUpload,
    GpuRender,
    #[default]
    Balanced,
}

#[derive(Default)]
pub struct PerformanceStats {
    pub rng_calls_per_sec: AtomicU64,
    pub points_generated_per_sec: AtomicU64,
    pub avg_batch_time_ms: parking_lot::Mutex<f32>,
    pub current_batch_size: AtomicUsize,
    pub dropped_batches: AtomicU64,
    pub total_batches: AtomicU64,

    pub fps: parking_lot::Mutex<f32>,
    pub points_rendered: AtomicUsize,

    pub bottleneck: parking_lot::Mutex<Bottleneck>,
}

impl PerformanceStats {
    pub fn update_bottleneck(&self) {
        let dropped = self.dropped_batches.load(Ordering::Relaxed);
        let total = self.total_batches.load(Ordering::Relaxed);
        let fps = *self.fps.lock();
        let rng_rate = self.rng_calls_per_sec.load(Ordering::Relaxed);

        let bottleneck = if total > 0 && dropped as f64 / total as f64 > 0.1 {
            Bottleneck::GpuUpload
        } else if fps < 30.0 && dropped == 0 {
            Bottleneck::GpuRender
        } else if rng_rate < 1_000_000 {
            Bottleneck::CpuRng
        } else {
            Bottleneck::Balanced
        };

        *self.bottleneck.lock() = bottleneck;
    }
}

pub enum RngCommand {
    UpdateCode(String),
    Stop,
    Reset,
    SetSeed(i64),
    Pause,
    Resume,
}

pub struct RngEngine {
    tx_cmd: Sender<RngCommand>,
    rx_points: Receiver<Vec<f32>>,
    stats: Arc<PerformanceStats>,
    bounds: Arc<AtomicBounds>,
    paused: Arc<AtomicBool>,
    thread_handle: Option<JoinHandle<()>>,
    last_error: Arc<Mutex<Option<String>>>,
}

impl RngEngine {
    pub fn new() -> Self {
        let (tx_cmd, rx_cmd) = channel::unbounded::<RngCommand>();
        let (tx_points, rx_points) = channel::bounded::<Vec<f32>>(CHANNEL_CAPACITY);
        let stats = Arc::new(PerformanceStats::default());
        let bounds = Arc::new(AtomicBounds::default());
        let paused = Arc::new(AtomicBool::new(false));
        let last_error = Arc::new(Mutex::new(None));

        let stats_clone = Arc::clone(&stats);
        let bounds_clone = Arc::clone(&bounds);
        let paused_clone = Arc::clone(&paused);
        let last_error_clone = Arc::clone(&last_error);

        let thread_handle = thread::spawn(move || {
            rng_thread(
                rx_cmd,
                tx_points,
                stats_clone,
                bounds_clone,
                paused_clone,
                last_error_clone,
            );
        });

        Self {
            tx_cmd,
            rx_points,
            stats,
            bounds,
            paused,
            thread_handle: Some(thread_handle),
            last_error,
        }
    }

    pub fn update_code(&self, code: &str) {
        let _ = self.tx_cmd.send(RngCommand::UpdateCode(code.to_string()));
    }

    pub fn reset(&self) {
        let _ = self.tx_cmd.send(RngCommand::Reset);
    }

    pub fn set_seed(&self, seed: i64) {
        let _ = self.tx_cmd.send(RngCommand::SetSeed(seed));
    }

    pub fn stop(&self) {
        let _ = self.tx_cmd.send(RngCommand::Stop);
    }

    pub fn pause(&self) {
        self.paused.store(true, Ordering::Relaxed);
        let _ = self.tx_cmd.send(RngCommand::Pause);
    }

    pub fn resume(&self) {
        self.paused.store(false, Ordering::Relaxed);
        let _ = self.tx_cmd.send(RngCommand::Resume);
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }

    pub fn try_recv_batch(&self) -> Option<Vec<f32>> {
        self.rx_points.try_recv().ok()
    }

    pub fn stats(&self) -> &Arc<PerformanceStats> {
        &self.stats
    }

    pub fn bounds(&self) -> &Arc<AtomicBounds> {
        &self.bounds
    }

    pub fn last_error(&self) -> Option<String> {
        self.last_error.lock().clone()
    }
}

impl Drop for RngEngine {
    fn drop(&mut self) {
        let _ = self.tx_cmd.send(RngCommand::Stop);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

fn rng_thread(
    rx_cmd: Receiver<RngCommand>,
    tx_points: Sender<Vec<f32>>,
    stats: Arc<PerformanceStats>,
    bounds: Arc<AtomicBounds>,
    paused: Arc<AtomicBool>,
    last_error: Arc<Mutex<Option<String>>>,
) {
    let mut vm: Option<VM> = None;
    let mut rng_func: Option<CallableFunction> = None;
    let mut current_state = Value::int(12345);
    let mut batch_size = 10_000usize;
    let mut running = false;

    let mut calls_this_sec = 0u64;
    let mut points_this_sec = 0u64;
    let mut last_stats_update = std::time::Instant::now();
    let mut batch_times = Vec::with_capacity(20);

    loop {
        while let Ok(cmd) = rx_cmd.try_recv() {
            match cmd {
                RngCommand::UpdateCode(code) => {
                    *last_error.lock() = None;
                    running = false;

                    match compile_rng(&code) {
                        Ok((new_vm, func)) => {
                            vm = Some(new_vm);
                            rng_func = Some(func);
                            current_state = Value::int(12345);
                            batch_size = 10_000;
                            running = true;
                        }
                        Err(e) => {
                            *last_error.lock() = Some(e);
                            vm = None;
                            rng_func = None;
                        }
                    }
                }
                RngCommand::Stop => {
                    return;
                }
                RngCommand::Reset => {
                    current_state = Value::int(12345);
                    batch_size = 10_000;
                }
                RngCommand::SetSeed(seed) => {
                    current_state = Value::int(seed);
                }
                RngCommand::Pause => {}
                RngCommand::Resume => {}
            }
        }

        if !running || paused.load(Ordering::Relaxed) {
            thread::sleep(std::time::Duration::from_millis(10));
            continue;
        }

        let (Some(vm_instance), Some(func)) = (&mut vm, &rng_func) else {
            thread::sleep(std::time::Duration::from_millis(10));
            continue;
        };

        let batch_start = std::time::Instant::now();

        let min_x = bounds.min_x.load(Ordering::Relaxed);
        let max_x = bounds.max_x.load(Ordering::Relaxed);
        let min_y = bounds.min_y.load(Ordering::Relaxed);
        let max_y = bounds.max_y.load(Ordering::Relaxed);
        let min_z = bounds.min_z.load(Ordering::Relaxed);
        let max_z = bounds.max_z.load(Ordering::Relaxed);

        let mut batch = Vec::with_capacity(batch_size * 3);
        let mut state = current_state;
        let mut batch_calls = 0u64;
        let mut error_occurred = false;

        for _ in 0..batch_size {
            let x_result = func.call(vm_instance, &[state]);
            let x_state = match x_result {
                Ok(v) => v,
                Err(e) => {
                    *last_error.lock() = Some(e.to_string());
                    running = false;
                    error_occurred = true;
                    break;
                }
            };
            let x = x_state.as_int().unwrap_or(0);
            batch.push(normalize_value(x, min_x, max_x));
            batch_calls += 1;

            let y_result = func.call(vm_instance, &[x_state]);
            let y_state = match y_result {
                Ok(v) => v,
                Err(e) => {
                    *last_error.lock() = Some(e.to_string());
                    running = false;
                    error_occurred = true;
                    break;
                }
            };
            let y = y_state.as_int().unwrap_or(0);
            batch.push(normalize_value(y, min_y, max_y));
            batch_calls += 1;

            let z_result = func.call(vm_instance, &[y_state]);
            let z_state = match z_result {
                Ok(v) => v,
                Err(e) => {
                    *last_error.lock() = Some(e.to_string());
                    running = false;
                    error_occurred = true;
                    break;
                }
            };
            let z = z_state.as_int().unwrap_or(0);
            batch.push(normalize_value(z, min_z, max_z));
            batch_calls += 1;

            state = z_state;
        }

        if error_occurred {
            continue;
        }

        current_state = state;

        let elapsed_ms = batch_start.elapsed().as_secs_f32() * 1000.0;
        batch_times.push(elapsed_ms);
        if batch_times.len() > 20 {
            batch_times.remove(0);
        }

        if elapsed_ms < TARGET_BATCH_TIME_MS * 0.8 {
            batch_size = ((batch_size as f32 * 1.2) as usize).min(MAX_BATCH_SIZE);
        } else if elapsed_ms > TARGET_BATCH_TIME_MS * 1.2 {
            batch_size = ((batch_size as f32 * 0.8) as usize).max(MIN_BATCH_SIZE);
        }

        calls_this_sec += batch_calls;
        points_this_sec += (batch.len() / 3) as u64;

        stats.total_batches.fetch_add(1, Ordering::Relaxed);

        match tx_points.try_send(batch) {
            Ok(_) => {}
            Err(TrySendError::Full(_)) => {
                stats.dropped_batches.fetch_add(1, Ordering::Relaxed);
            }
            Err(TrySendError::Disconnected(_)) => {
                return;
            }
        }

        if last_stats_update.elapsed().as_secs_f32() >= 1.0 {
            stats
                .rng_calls_per_sec
                .store(calls_this_sec, Ordering::Relaxed);
            stats
                .points_generated_per_sec
                .store(points_this_sec, Ordering::Relaxed);
            stats
                .current_batch_size
                .store(batch_size, Ordering::Relaxed);

            if !batch_times.is_empty() {
                let avg = batch_times.iter().sum::<f32>() / batch_times.len() as f32;
                *stats.avg_batch_time_ms.lock() = avg;
            }

            stats.update_bottleneck();

            calls_this_sec = 0;
            points_this_sec = 0;
            last_stats_update = std::time::Instant::now();
        }
    }
}

fn compile_rng(code: &str) -> Result<(VM, CallableFunction), String> {
    let mut vm = new_vm().map_err(|e| format!("VM init error: {}", e))?;

    run_with_vm(&mut vm, code, "rng_def").map_err(|e| format!("{}", e))?;

    let func = get_function(&vm, "rng").map_err(|e| format!("{}", e))?;

    if func.arity() != 1 {
        return Err(format!(
            "Function 'rng' must take exactly 1 argument, got {}",
            func.arity()
        ));
    }

    Ok((vm, func))
}
