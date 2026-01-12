use crate::math::examples::MathFunctionKind;
use crate::renderer::CameraMode;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Rng,
    Math,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Mode3D,
    Mode2D,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MathViewMode {
    Mode3D,
    Mode2D,
}

pub struct UiState {
    pub app_mode: AppMode,

    pub code: String,
    pub selected_example: usize,

    pub view_mode: ViewMode,
    pub camera_mode: CameraMode,
    pub vsync_enabled: bool,

    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
    pub max_points: usize,

    pub seed: i64,

    pub show_stats: bool,

    pub fps_cap_enabled: bool,
    pub fps_cap: u32,

    pub code_needs_compile: bool,

    pub grid_size: u32,

    pub math_code: String,
    pub math_selected_example: usize,
    pub math_function_type: MathFunctionKind,
    pub math_view_mode: MathViewMode,
    pub math_x_range: (f32, f32),
    pub math_y_range: (f32, f32),
    pub math_t_range: (f32, f32),
    pub math_resolution: u32,
    pub math_samples: u32,
    pub math_u_range: (f32, f32),
    pub math_v_range: (f32, f32),
    pub math_u_samples: u32,
    pub math_v_samples: u32,
    pub math_needs_compile: bool,
    pub show_grid: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            app_mode: AppMode::Rng,

            code: crate::rng::RNG_EXAMPLES[0].code.to_string(),
            selected_example: 0,

            view_mode: ViewMode::Mode3D,
            camera_mode: CameraMode::Free,
            vsync_enabled: false,

            bounds_min: [-500.0, -500.0, -500.0],
            bounds_max: [500.0, 500.0, 500.0],
            max_points: 1_000_000,

            seed: 12345,

            show_stats: true,

            fps_cap_enabled: false,
            fps_cap: 144,

            code_needs_compile: true,

            grid_size: 512,

            math_code: crate::math::MATH_EXAMPLES[0].code.to_string(),
            math_selected_example: 0,
            math_function_type: MathFunctionKind::Surface,
            math_view_mode: MathViewMode::Mode3D,
            math_x_range: (-6.28, 6.28),
            math_y_range: (-6.28, 6.28),
            math_t_range: (0.0, 6.28),
            math_resolution: 100,
            math_samples: 1000,
            math_u_range: (0.0, 6.28),
            math_v_range: (0.0, 6.28),
            math_u_samples: 50,
            math_v_samples: 50,
            math_needs_compile: true,
            show_grid: true,
        }
    }
}
