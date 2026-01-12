use egui::{Color32, Context, RichText, ScrollArea, TextEdit, Ui};
use std::sync::atomic::Ordering;

use crate::math::MATH_EXAMPLES;
use crate::math::examples::MathFunctionKind;
use crate::renderer::CameraMode;
use crate::rng::{Bottleneck, PerformanceStats, RNG_EXAMPLES};
use crate::ui::state::{AppMode, MathViewMode, UiState, ViewMode};
use crate::ui::theme::*;

pub struct UiActions {
    pub compile_code: bool,
    pub reset_rng: bool,
    pub set_seed: Option<i64>,
    pub toggle_pause: bool,
    pub clear_points: bool,
    pub compile_math: bool,
}

impl Default for UiActions {
    fn default() -> Self {
        Self {
            compile_code: false,
            reset_rng: false,
            set_seed: None,
            toggle_pause: false,
            clear_points: false,
            compile_math: false,
        }
    }
}

pub fn draw_side_panel(
    ctx: &Context,
    state: &mut UiState,
    stats: &PerformanceStats,
    last_error: &Option<String>,
    is_paused: bool,
) -> UiActions {
    let mut actions = UiActions::default();

    egui::SidePanel::right("control_panel")
        .min_width(340.0)
        .max_width(420.0)
        .default_width(360.0)
        .frame(egui::Frame::default().fill(BG_PANEL).inner_margin(16.0))
        .show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.heading(RichText::new("PRNG 3D").strong());
                ui.add_space(4.0);
                ui.label(RichText::new("Visualizer & Math Plotter").color(TEXT_MUTED).size(11.0));
                ui.add_space(16.0);

                ui.label(RichText::new("MODE").color(TEXT_MUTED).size(11.0).strong());
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    let rng_btn = egui::Button::new(
                        RichText::new("RNG").color(if state.app_mode == AppMode::Rng { BG_PURE_BLACK } else { TEXT_PRIMARY })
                    ).fill(if state.app_mode == AppMode::Rng { ACCENT_PURPLE } else { BG_WIDGET })
                    .min_size(egui::vec2(80.0, 32.0));

                    let math_btn = egui::Button::new(
                        RichText::new("Math").color(if state.app_mode == AppMode::Math { BG_PURE_BLACK } else { TEXT_PRIMARY })
                    ).fill(if state.app_mode == AppMode::Math { ACCENT_BLUE } else { BG_WIDGET })
                    .min_size(egui::vec2(80.0, 32.0));

                    if ui.add(rng_btn).clicked() && state.app_mode != AppMode::Rng {
                        state.app_mode = AppMode::Rng;
                        actions.clear_points = true;
                    }

                    if ui.add(math_btn).clicked() && state.app_mode != AppMode::Math {
                        state.app_mode = AppMode::Math;
                        if state.math_needs_compile {
                            actions.compile_math = true;
                        }
                    }
                });
                ui.add_space(16.0);
                ui.separator();
                ui.add_space(12.0);

                match state.app_mode {
                    AppMode::Rng => {
                        ui.horizontal(|ui| {
                            let (text, color) = if is_paused { ("Resume", ACCENT_GREEN) } else { ("Pause", ACCENT_ORANGE) };
                            if ui.add(egui::Button::new(RichText::new(text).color(BG_PURE_BLACK))
                                .fill(color).min_size(egui::vec2(80.0, 32.0))).clicked() {
                                actions.toggle_pause = true;
                            }
                            if ui.button("Clear").clicked() {
                                actions.clear_points = true;
                            }
                            if ui.button("Reset").clicked() {
                                actions.reset_rng = true;
                                actions.clear_points = true;
                            }
                        });
                        ui.add_space(16.0);

                        section_header(ui, "PRESET");
                        egui::ComboBox::from_id_salt("rng_examples")
                            .selected_text(RNG_EXAMPLES[state.selected_example].name)
                            .width(ui.available_width())
                            .show_ui(ui, |ui| {
                                for (i, ex) in RNG_EXAMPLES.iter().enumerate() {
                                    if ui.selectable_label(state.selected_example == i, ex.name).clicked() {
                                        state.selected_example = i;
                                        state.code = ex.code.to_string();
                                        state.code_needs_compile = true;
                                    }
                                }
                            });
                        if state.selected_example < RNG_EXAMPLES.len() {
                            ui.add_space(4.0);
                            ui.label(RichText::new(RNG_EXAMPLES[state.selected_example].description)
                                .color(TEXT_MUTED).size(11.0).italics());
                        }
                        ui.add_space(16.0);

                        section_header(ui, "AELYS CODE");
                        code_editor(ui, &mut state.code, last_error);
                        ui.add_space(8.0);
                        let btn_text = if state.code_needs_compile { "Compile & Run" } else { "Running..." };
                        let btn_color = if state.code_needs_compile { ACCENT_GREEN } else { BG_WIDGET };
                        let text_color = if state.code_needs_compile { BG_PURE_BLACK } else { ACCENT_GREEN };
                        if ui.add(egui::Button::new(RichText::new(btn_text).color(text_color))
                            .fill(btn_color).min_size(egui::vec2(ui.available_width(), 32.0))).clicked()
                            && state.code_needs_compile {
                            actions.compile_code = true;
                            actions.clear_points = true;
                            state.code_needs_compile = false;
                        }
                        ui.add_space(16.0);

                        ui.separator();
                        ui.add_space(12.0);

                        section_header(ui, "VIEW");
                        ui.horizontal(|ui| {
                            ui.label("Mode:");
                            if ui.selectable_label(state.view_mode == ViewMode::Mode3D, "3D").clicked() {
                                state.view_mode = ViewMode::Mode3D;
                            }
                            if ui.selectable_label(state.view_mode == ViewMode::Mode2D, "2D").clicked() {
                                state.view_mode = ViewMode::Mode2D;
                            }
                        });
                        if state.view_mode == ViewMode::Mode3D {
                            camera_controls(ui, &mut state.camera_mode);
                        } else {
                            ui.horizontal(|ui| {
                                ui.label("Grid:");
                                ui.add(egui::Slider::new(&mut state.grid_size, 128..=1024).suffix("px"));
                            });
                        }
                        ui.add_space(16.0);

                        section_header(ui, "BOUNDS");
                        bounds_grid(ui, &mut state.bounds_min, &mut state.bounds_max);
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.label("Max points:");
                            let mut k = (state.max_points / 1000) as u32;
                            if ui.add(egui::Slider::new(&mut k, 10..=4000).suffix("K")).changed() {
                                state.max_points = (k as usize) * 1000;
                            }
                        });
                        ui.add_space(16.0);

                        section_header(ui, "SEED");
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(&mut state.seed).speed(1.0));
                            if ui.button("Apply").clicked() {
                                actions.set_seed = Some(state.seed);
                                actions.clear_points = true;
                            }
                            if ui.button("Random").clicked() {
                                state.seed = rand_seed();
                                actions.set_seed = Some(state.seed);
                                actions.clear_points = true;
                            }
                        });
                        ui.add_space(16.0);

                        perf_controls(ui, state);
                        ui.add_space(16.0);

                        ui.separator();
                        ui.add_space(12.0);

                        if state.show_stats {
                            stats_panel(ui, stats, is_paused);
                        }
                    }
                    AppMode::Math => {
                        section_header(ui, "FUNCTION TYPE");
                        ui.horizontal(|ui| {
                            if ui.selectable_label(state.math_function_type == MathFunctionKind::Surface, "Surface z=f(x,y)").clicked() {
                                state.math_function_type = MathFunctionKind::Surface;
                                state.math_needs_compile = true;
                            }
                            if ui.selectable_label(state.math_function_type == MathFunctionKind::ParametricCurve, "Parametric Curve").clicked() {
                                state.math_function_type = MathFunctionKind::ParametricCurve;
                                state.math_needs_compile = true;
                            }
                            if ui.selectable_label(state.math_function_type == MathFunctionKind::ParametricSurface, "Parametric Surface").clicked() {
                                state.math_function_type = MathFunctionKind::ParametricSurface;
                                state.math_needs_compile = true;
                            }
                        });
                        ui.add_space(12.0);

                        section_header(ui, "PRESET");
                        let filtered: Vec<_> = MATH_EXAMPLES.iter().enumerate()
                            .filter(|(_, ex)| ex.function_type == state.math_function_type).collect();

                        if !filtered.is_empty() {
                            let name = if state.math_selected_example < MATH_EXAMPLES.len() {
                                MATH_EXAMPLES[state.math_selected_example].name
                            } else { "Select..." };

                            egui::ComboBox::from_id_salt("math_examples")
                                .selected_text(name).width(ui.available_width())
                                .show_ui(ui, |ui| {
                                    for (i, ex) in &filtered {
                                        if ui.selectable_label(state.math_selected_example == *i, ex.name).clicked() {
                                            state.math_selected_example = *i;
                                            state.math_code = ex.code.to_string();
                                            state.math_x_range = (ex.x_range.0 as f32, ex.x_range.1 as f32);
                                            state.math_y_range = (ex.y_range.0 as f32, ex.y_range.1 as f32);
                                            state.math_t_range = (ex.t_range.0 as f32, ex.t_range.1 as f32);
                                            state.math_u_range = (ex.u_range.0 as f32, ex.u_range.1 as f32);
                                            state.math_v_range = (ex.v_range.0 as f32, ex.v_range.1 as f32);
                                            state.math_u_samples = ex.u_samples as u32;
                                            state.math_v_samples = ex.v_samples as u32;
                                            state.math_needs_compile = true;
                                            actions.compile_math = true;
                                        }
                                    }
                                });

                            if state.math_selected_example < MATH_EXAMPLES.len() {
                                ui.add_space(4.0);
                                ui.label(RichText::new(MATH_EXAMPLES[state.math_selected_example].description)
                                    .color(TEXT_MUTED).size(11.0).italics());
                            }
                        }
                        ui.add_space(16.0);

                        section_header(ui, "AELYS CODE");
                        let hint = match state.math_function_type {
                            MathFunctionKind::Surface => "Define: fn f(x: float, y: float) -> float",
                            MathFunctionKind::ParametricCurve => "Define: fn fx(t), fy(t), fz(t) -> float",
                            MathFunctionKind::ParametricSurface => "Define: fn fx(u, v), fy(u, v), fz(u, v) -> float",
                        };
                        ui.label(RichText::new(hint).color(TEXT_MUTED).size(10.0).italics());
                        ui.add_space(4.0);
                        code_editor(ui, &mut state.math_code, last_error);
                        ui.add_space(8.0);

                        let (btn_text, btn_color, text_color) = if state.math_needs_compile {
                            ("Plot", ACCENT_BLUE, BG_PURE_BLACK)
                        } else {
                            ("Plotted", BG_WIDGET, ACCENT_BLUE)
                        };
                        if ui.add(egui::Button::new(RichText::new(btn_text).color(text_color))
                            .fill(btn_color).min_size(egui::vec2(ui.available_width(), 32.0))).clicked()
                            && state.math_needs_compile {
                            actions.compile_math = true;
                            state.math_needs_compile = false;
                        }
                        ui.add_space(16.0);

                        ui.separator();
                        ui.add_space(12.0);

                        section_header(ui, "PARAMETERS");
                        let mut changed = false;
                        match state.math_function_type {
                            MathFunctionKind::Surface => {
                                changed |= range_controls(ui, "X", &mut state.math_x_range);
                                changed |= range_controls(ui, "Y", &mut state.math_y_range);
                                ui.add_space(8.0);
                                ui.horizontal(|ui| {
                                    ui.label("Resolution:");
                                    changed |= ui.add(egui::Slider::new(&mut state.math_resolution, 20..=200)).changed();
                                });
                            }
                            MathFunctionKind::ParametricCurve => {
                                ui.horizontal(|ui| {
                                    ui.label("t min:");
                                    changed |= ui.add(egui::DragValue::new(&mut state.math_t_range.0).speed(0.1)).changed();
                                    ui.label("max:");
                                    changed |= ui.add(egui::DragValue::new(&mut state.math_t_range.1).speed(0.1)).changed();
                                });
                                ui.add_space(8.0);
                                ui.horizontal(|ui| {
                                    ui.label("Samples:");
                                    changed |= ui.add(egui::Slider::new(&mut state.math_samples, 100..=5000)).changed();
                                });
                            }
                            MathFunctionKind::ParametricSurface => {
                                ui.label("U Range:");
                                changed |= range_controls_inline(ui, &mut state.math_u_range);
                                ui.label("V Range:");
                                changed |= range_controls_inline(ui, &mut state.math_v_range);
                                ui.label("U Samples:");
                                changed |= ui.add(egui::Slider::new(&mut state.math_u_samples, 10..=200)).changed();
                                ui.label("V Samples:");
                                changed |= ui.add(egui::Slider::new(&mut state.math_v_samples, 10..=200)).changed();
                            }
                        }
                        if changed {
                            state.math_needs_compile = true;
                            actions.compile_math = true;
                        }
                        ui.add_space(16.0);

                        section_header(ui, "VIEW");
                        ui.horizontal(|ui| {
                            ui.label("Mode:");
                            if ui.selectable_label(state.math_view_mode == MathViewMode::Mode3D, "3D").clicked() {
                                state.math_view_mode = MathViewMode::Mode3D;
                            }
                            if ui.selectable_label(state.math_view_mode == MathViewMode::Mode2D, "2D").clicked() {
                                state.math_view_mode = MathViewMode::Mode2D;
                            }
                        });
                        if state.math_view_mode == MathViewMode::Mode3D {
                            ui.checkbox(&mut state.show_grid, "Show Grid");
                            camera_controls(ui, &mut state.camera_mode);
                        }
                        ui.add_space(16.0);

                        perf_controls(ui, state);
                    }
                }
            });
        });

    actions
}

fn section_header(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).color(TEXT_MUTED).size(11.0).strong());
    ui.add_space(4.0);
}

fn code_editor(ui: &mut Ui, code: &mut String, error: &Option<String>) {
    let frame = egui::Frame::default()
        .fill(BG_PURE_BLACK)
        .stroke(egui::Stroke::new(1.0, BORDER_SUBTLE))
        .rounding(6.0)
        .inner_margin(8.0);

    frame.show(ui, |ui| {
        ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
            ui.horizontal_top(|ui| {
                let lines = code.lines().count().max(1);
                let line_nums: String = (1..=lines).map(|n| format!("{:3}\n", n)).collect();
                ui.add(egui::Label::new(
                    RichText::new(line_nums.trim_end())
                        .color(TEXT_MUTED)
                        .family(egui::FontFamily::Monospace)
                        .size(12.0)
                ));
                ui.add_space(8.0);
                ui.add(
                    TextEdit::multiline(code)
                        .font(egui::FontId::new(12.0, egui::FontFamily::Monospace))
                        .code_editor()
                        .desired_width(f32::INFINITY)
                        .desired_rows(10)
                        .lock_focus(true)
                        .text_color(TEXT_PRIMARY)
                );
            });
        });
    });

    if let Some(err) = error {
        ui.add_space(6.0);
        egui::Frame::default()
            .fill(Color32::from_rgb(40, 15, 15))
            .stroke(egui::Stroke::new(1.0, ACCENT_RED))
            .rounding(4.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.label(RichText::new(err).color(ACCENT_RED).size(11.0));
            });
    }
}

fn camera_controls(ui: &mut Ui, mode: &mut CameraMode) {
    ui.horizontal(|ui| {
        ui.label("Camera:");
        if ui.selectable_label(*mode == CameraMode::Free, "Free").clicked() {
            *mode = CameraMode::Free;
        }
        if ui.selectable_label(*mode == CameraMode::Orbital, "Orbital").clicked() {
            *mode = CameraMode::Orbital;
        }
    });
}

fn bounds_grid(ui: &mut Ui, mins: &mut [f32; 3], maxs: &mut [f32; 3]) {
    egui::Grid::new("bounds").num_columns(3).spacing([8.0, 4.0]).show(ui, |ui| {
        ui.label("");
        ui.label(RichText::new("Min").color(TEXT_MUTED).size(10.0));
        ui.label(RichText::new("Max").color(TEXT_MUTED).size(10.0));
        ui.end_row();

        for (i, label) in ["X", "Y", "Z"].iter().enumerate() {
            ui.label(*label);
            ui.add(egui::DragValue::new(&mut mins[i]).speed(5.0));
            ui.add(egui::DragValue::new(&mut maxs[i]).speed(5.0));
            ui.end_row();
        }
    });
}

fn range_controls(ui: &mut Ui, label: &str, range: &mut (f32, f32)) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(format!("{}:", label));
        changed |= ui.add(egui::DragValue::new(&mut range.0).speed(0.1)).changed();
        ui.label("to");
        changed |= ui.add(egui::DragValue::new(&mut range.1).speed(0.1)).changed();
    });
    changed
}

fn range_controls_inline(ui: &mut Ui, range: &mut (f32, f32)) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        changed |= ui.add(egui::DragValue::new(&mut range.0).speed(0.1)).changed();
        ui.label("to");
        changed |= ui.add(egui::DragValue::new(&mut range.1).speed(0.1)).changed();
    });
    changed
}

fn perf_controls(ui: &mut Ui, state: &mut UiState) {
    section_header(ui, "PERFORMANCE");
    ui.horizontal(|ui| {
        ui.checkbox(&mut state.vsync_enabled, "VSync");
        ui.checkbox(&mut state.show_stats, "Stats");
    });
    ui.horizontal(|ui| {
        ui.checkbox(&mut state.fps_cap_enabled, "FPS Cap:");
        ui.add_enabled(state.fps_cap_enabled,
            egui::DragValue::new(&mut state.fps_cap).range(30..=500).suffix(" fps"));
    });
}

fn stats_panel(ui: &mut Ui, stats: &PerformanceStats, paused: bool) {
    section_header(ui, "STATISTICS");
    egui::Frame::default()
        .fill(BG_WIDGET)
        .stroke(egui::Stroke::new(1.0, BORDER_SUBTLE))
        .rounding(6.0)
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::new(11.0, egui::FontFamily::Monospace));

            let fps = *stats.fps.lock();
            let fps_color = if fps >= 60.0 { ACCENT_GREEN } else if fps >= 30.0 { ACCENT_ORANGE } else { ACCENT_RED };

            egui::Grid::new("stats").num_columns(2).spacing([20.0, 4.0]).show(ui, |ui| {
                ui.label(RichText::new("FPS").color(TEXT_MUTED));
                ui.label(RichText::new(format!("{:.0}", fps)).color(fps_color));
                ui.end_row();

                ui.label(RichText::new("RNG/s").color(TEXT_MUTED));
                ui.label(RichText::new(fmt_num(stats.rng_calls_per_sec.load(Ordering::Relaxed) as usize)).color(ACCENT_BLUE));
                ui.end_row();

                ui.label(RichText::new("Points/s").color(TEXT_MUTED));
                ui.label(RichText::new(fmt_num(stats.points_generated_per_sec.load(Ordering::Relaxed) as usize)).color(ACCENT_PURPLE));
                ui.end_row();

                ui.label(RichText::new("Rendered").color(TEXT_MUTED));
                ui.label(RichText::new(fmt_num(stats.points_rendered.load(Ordering::Relaxed))).color(TEXT_PRIMARY));
                ui.end_row();

                ui.label(RichText::new("Batch").color(TEXT_MUTED));
                ui.label(RichText::new(fmt_num(stats.current_batch_size.load(Ordering::Relaxed))).color(TEXT_PRIMARY));
                ui.end_row();

                ui.label(RichText::new("Batch ms").color(TEXT_MUTED));
                ui.label(RichText::new(format!("{:.1}", *stats.avg_batch_time_ms.lock())).color(TEXT_PRIMARY));
                ui.end_row();
            });

            ui.add_space(8.0);

            let status = if paused {
                RichText::new("PAUSED").color(ACCENT_ORANGE).strong()
            } else {
                let (text, color) = match *stats.bottleneck.lock() {
                    Bottleneck::CpuRng => ("CPU Limited", ACCENT_ORANGE),
                    Bottleneck::GpuUpload => ("GPU Upload", ACCENT_RED),
                    Bottleneck::GpuRender => ("GPU Render", ACCENT_RED),
                    Bottleneck::Balanced => ("Balanced", ACCENT_GREEN),
                };
                RichText::new(text).color(color)
            };

            ui.horizontal(|ui| {
                ui.label(RichText::new("Status:").color(TEXT_MUTED));
                ui.label(status);
            });
        });
}

pub fn draw_help_overlay(ctx: &Context, pos: [f32; 3], speed: f32) {
    egui::Area::new(egui::Id::new("help_overlay"))
        .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(12.0, -12.0))
        .show(ctx, |ui| {
            egui::Frame::default()
                .fill(Color32::from_black_alpha(180))
                .rounding(6.0)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.style_mut().override_font_id = Some(egui::FontId::new(11.0, egui::FontFamily::Monospace));
                    ui.label(RichText::new("WASD - Move | RMB+Drag - Look | Scroll - Speed").color(TEXT_MUTED));
                    ui.label(RichText::new(format!("Pos: ({:.0}, {:.0}, {:.0}) | Speed: {:.0}", pos[0], pos[1], pos[2], speed)).color(TEXT_MUTED));
                });
        });
}

fn fmt_num(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{:.2}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        format!("{}", n)
    }
}

fn rand_seed() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    ((nanos % 0x7FFF_FFFF_FFFF) as i64).max(1)
}
