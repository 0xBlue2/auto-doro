use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct OrbitCamera {
    pub radius: f32,
    pub alpha: f32,
    pub beta: f32,
    pub focus: Vec3,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            radius: 10.0,
            alpha: 0.0,
            beta: std::f32::consts::PI / 6.0,
            focus: Vec3::ZERO,
        }
    }
}

pub fn update_camera(
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut motion: EventReader<MouseMotion>,
    mut scroll: EventReader<bevy::input::mouse::MouseWheel>,
    mut drag_mode: Local<DragMode>,
    mut query: Query<(&mut Transform, &mut OrbitCamera)>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let mut window = windows.single_mut();
    let width = window.resolution.width();
    let height = window.resolution.height();

    if mouse.just_pressed(MouseButton::Left) {
        if let Some(cursor_pos) = window.cursor_position() {
            let margin_x = width * 0.2;
            let margin_y = height * 0.2;
            if cursor_pos.x > margin_x
                && cursor_pos.x < width - margin_x
                && cursor_pos.y > margin_y
                && cursor_pos.y < height - margin_y
            {
                *drag_mode = DragMode::Rotate;
            } else {
                *drag_mode = DragMode::MoveWindow;
            }
        }
    } else if mouse.just_released(MouseButton::Left) {
        *drag_mode = DragMode::None;
    }

    let mut delta = Vec2::ZERO;
    for ev in motion.read() {
        delta += ev.delta;
    }

    if *drag_mode == DragMode::MoveWindow && delta != Vec2::ZERO {
        if let WindowPosition::At(pos) = window.position {
            window.position = WindowPosition::At(pos + delta.as_ivec2());
        }
    }

    let mut zoom = 0.0;
    for ev in scroll.read() {
        zoom += ev.y;
    }

    for (mut transform, mut cam) in query.iter_mut() {
        // Automatic slow spin
        cam.alpha += 0.2 * time.delta_seconds();

        // Mouse drag overrides
        if *drag_mode == DragMode::Rotate && delta != Vec2::ZERO {
            cam.alpha -= delta.x * 0.005;
            cam.beta += delta.y * 0.005;
        }

        // Scroll zoom
        if zoom != 0.0 {
            // zoom distance scales with current radius
            cam.radius -= zoom * cam.radius * 0.1;
            cam.radius = cam.radius.max(0.5);
        }

        cam.beta = cam.beta.clamp(0.01, std::f32::consts::PI - 0.01);

        let x = cam.radius * cam.beta.sin() * cam.alpha.cos();
        let y = cam.radius * cam.beta.cos();
        let z = cam.radius * cam.beta.sin() * cam.alpha.sin();

        *transform = Transform::from_xyz(cam.focus.x + x, cam.focus.y + y, cam.focus.z + z)
            .looking_at(cam.focus, Vec3::Y);
    }
}

#[derive(Default, PartialEq)]
pub enum DragMode {
    #[default]
    None,
    Rotate,
    MoveWindow,
}
