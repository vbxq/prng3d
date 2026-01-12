pub struct MathExample {
    pub name: &'static str,
    pub description: &'static str,
    pub function_type: MathFunctionKind,
    pub code: &'static str,
    pub x_range: (f64, f64),
    pub y_range: (f64, f64),
    pub t_range: (f64, f64),
    pub u_range: (f64, f64),
    pub v_range: (f64, f64),
    pub u_samples: usize,
    pub v_samples: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MathFunctionKind {
    Surface,
    ParametricCurve,
    ParametricSurface,
}

pub const MATH_EXAMPLES: &[MathExample] = &[
    MathExample {
        name: "Sine Wave",
        description: "Basic sine wave",
        function_type: MathFunctionKind::Surface,
        code: r#"fn f(x: float, y: float) -> float {
    math.sin(x) + math.sin(y)
}"#,
        x_range: (-6.28, 6.28),
        y_range: (-6.28, 6.28),
        t_range: (0.0, 1.0),
        u_range: (0.0, 1.0),
        v_range: (0.0, 1.0),
        u_samples: 50,
        v_samples: 50,
    },
    MathExample {
        name: "Ripple",
        description: "Radial wave pattern",
        function_type: MathFunctionKind::Surface,
        code: r#"fn f(x: float, y: float) -> float {
    let r = math.sqrt(x*x + y*y)
    math.sin(r * 2.0) / (r + 1.0)
}"#,
        x_range: (-5.0, 5.0),
        y_range: (-5.0, 5.0),
        t_range: (0.0, 1.0),
        u_range: (0.0, 1.0),
        v_range: (0.0, 1.0),
        u_samples: 50,
        v_samples: 50,
    },
    MathExample {
        name: "Saddle",
        description: "x² - y²",
        function_type: MathFunctionKind::Surface,
        code: r#"fn f(x: float, y: float) -> float {
    x*x - y*y
}"#,
        x_range: (-3.0, 3.0),
        y_range: (-3.0, 3.0),
        t_range: (0.0, 1.0),
        u_range: (0.0, 1.0),
        v_range: (0.0, 1.0),
        u_samples: 50,
        v_samples: 50,
    },
    MathExample {
        name: "Peaks",
        description: "Multiple gaussian bumps",
        function_type: MathFunctionKind::Surface,
        code: r#"fn f(x: float, y: float) -> float {
    let t1 = 3.0 * (1.0-x) * (1.0-x) * math.exp(-x*x - (y+1.0)*(y+1.0))
    let t2 = -10.0 * (x/5.0 - x*x*x - y*y*y*y*y) * math.exp(-x*x - y*y)
    let t3 = -1.0/3.0 * math.exp(-(x+1.0)*(x+1.0) - y*y)
    t1 + t2 + t3
}"#,
        x_range: (-3.0, 3.0),
        y_range: (-3.0, 3.0),
        t_range: (0.0, 1.0),
        u_range: (0.0, 1.0),
        v_range: (0.0, 1.0),
        u_samples: 50,
        v_samples: 50,
    },
    MathExample {
        name: "Helix",
        description: "Spiral in 3D",
        function_type: MathFunctionKind::ParametricCurve,
        code: r#"fn fx(t: float) -> float { math.cos(t * 4.0) }
fn fy(t: float) -> float { t }
fn fz(t: float) -> float { math.sin(t * 4.0) }"#,
        x_range: (-1.0, 1.0),
        y_range: (-1.0, 1.0),
        t_range: (0.0, 6.28),
        u_range: (0.0, 1.0),
        v_range: (0.0, 1.0),
        u_samples: 50,
        v_samples: 50,
    },
    MathExample {
        name: "Trefoil Knot",
        description: "Classic knot",
        function_type: MathFunctionKind::ParametricCurve,
        code: r#"fn fx(t: float) -> float { math.sin(t) + 2.0 * math.sin(2.0*t) }
fn fy(t: float) -> float { math.cos(t) - 2.0 * math.cos(2.0*t) }
fn fz(t: float) -> float { -math.sin(3.0*t) }"#,
        x_range: (-1.0, 1.0),
        y_range: (-1.0, 1.0),
        t_range: (0.0, 6.28),
        u_range: (0.0, 1.0),
        v_range: (0.0, 1.0),
        u_samples: 50,
        v_samples: 50,
    },
    MathExample {
        name: "Lissajous",
        description: "3D lissajous figure",
        function_type: MathFunctionKind::ParametricCurve,
        code: r#"fn fx(t: float) -> float { math.sin(3.0*t) }
fn fy(t: float) -> float { math.sin(4.0*t) }
fn fz(t: float) -> float { math.sin(5.0*t) }"#,
        x_range: (-1.0, 1.0),
        y_range: (-1.0, 1.0),
        t_range: (0.0, 6.28),
        u_range: (0.0, 1.0),
        v_range: (0.0, 1.0),
        u_samples: 50,
        v_samples: 50,
    },
    MathExample {
        name: "Torus Knot",
        description: "Wraps around a torus",
        function_type: MathFunctionKind::ParametricCurve,
        code: r#"fn fx(t: float) -> float {
    let r = 2.0 + math.cos(3.0*t)
    r * math.cos(2.0*t)
}
fn fy(t: float) -> float { math.sin(3.0*t) }
fn fz(t: float) -> float {
    let r = 2.0 + math.cos(3.0*t)
    r * math.sin(2.0*t)
}"#,
        x_range: (-1.0, 1.0),
        y_range: (-1.0, 1.0),
        t_range: (0.0, 6.28),
        u_range: (0.0, 1.0),
        v_range: (0.0, 1.0),
        u_samples: 50,
        v_samples: 50,
    },
    MathExample {
        name: "Sphere",
        description: "Unit sphere",
        function_type: MathFunctionKind::ParametricSurface,
        code: r#"fn fx(u: float, v: float) -> float { math.sin(u) * math.cos(v) }
fn fy(u: float, v: float) -> float { math.cos(u) }
fn fz(u: float, v: float) -> float { math.sin(u) * math.sin(v) }"#,
        x_range: (-1.0, 1.0),
        y_range: (-1.0, 1.0),
        t_range: (0.0, 1.0),
        u_range: (0.0, 3.14159),
        v_range: (0.0, 6.28318),
        u_samples: 40,
        v_samples: 80,
    },
    MathExample {
        name: "Torus",
        description: "Donut",
        function_type: MathFunctionKind::ParametricSurface,
        code: r#"fn fx(u: float, v: float) -> float { (2.0 + math.cos(v)) * math.cos(u) }
fn fy(u: float, v: float) -> float { math.sin(v) }
fn fz(u: float, v: float) -> float { (2.0 + math.cos(v)) * math.sin(u) }"#,
        x_range: (-1.0, 1.0),
        y_range: (-1.0, 1.0),
        t_range: (0.0, 1.0),
        u_range: (0.0, 6.28318),
        v_range: (0.0, 6.28318),
        u_samples: 60,
        v_samples: 40,
    },
    MathExample {
        name: "Möbius Strip",
        description: "One-sided surface",
        function_type: MathFunctionKind::ParametricSurface,
        code: r#"fn fx(u: float, v: float) -> float {
    (1.0 + v * math.cos(u / 2.0)) * math.cos(u)
}
fn fy(u: float, v: float) -> float {
    v * math.sin(u / 2.0)
}
fn fz(u: float, v: float) -> float {
    (1.0 + v * math.cos(u / 2.0)) * math.sin(u)
}"#,
        x_range: (-1.0, 1.0),
        y_range: (-1.0, 1.0),
        t_range: (0.0, 1.0),
        u_range: (0.0, 6.28318),
        v_range: (-0.5, 0.5),
        u_samples: 80,
        v_samples: 20,
    },
];
