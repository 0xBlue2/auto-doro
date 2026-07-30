use bevy::input::mouse::MouseMotion;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::window::{
    CompositeAlphaMode, PrimaryWindow, Window, WindowLevel, WindowPlugin, WindowPosition,
    WindowResolution,
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};

#[derive(Default, PartialEq)]
enum DragMode {
    #[default]
    None,
    Rotate,
    MoveWindow,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgba(0.0, 0.1, 0.3, 0.6)))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "McLaren 720S Viewer".into(),
                        resolution: WindowResolution::new(300., 300.),
                        decorations: false, // border‑less viewer
                        transparent: true,
                        position: WindowPosition::At(IVec2::new(0, 0)),
                        #[cfg(target_os = "macos")]
                        composite_alpha_mode: CompositeAlphaMode::PostMultiplied,
                        window_level: WindowLevel::AlwaysOnTop,
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    filter: "info,wgpu_core=warn,wgpu_hal=warn,bevy_gltf=error,bevy_winit=error"
                        .into(),
                    level: bevy::log::Level::INFO,
                    ..default()
                }),
        )
        // We removed the duplicate ClearColor resource insertion
        .add_plugins(EguiPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (update_camera, ui_metadata, exit_on_esc, sync_windows),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load placeholder model (user should place mclaren.glb in assets/)
    commands.spawn(SceneBundle {
        // "2017 Mclaren 720s" (https://skfb.ly/ozQR7) by _shobh19 is licensed under Creative Commons Attribution (http://creativecommons.org/licenses/by/4.0/).
        scene: asset_server.load("mclaren.glb#Scene0"),
        ..default()
    });

    // Main camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 1.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        OrbitCamera::default(),
    ));

    // Light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 2000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // Secondary window for car info
    let info_window_id = commands
        .spawn(Window {
            title: "Car Info".into(),
            resolution: WindowResolution::new(300., 200.),
            decorations: false,
            transparent: true,
            position: WindowPosition::At(IVec2::new(320, 0)),
            window_level: WindowLevel::AlwaysOnTop,
            ..default()
        })
        .id();
    commands.insert_resource(InfoWindowId(info_window_id));
}

#[derive(Component)]
struct InfoWindowMarker;

#[derive(Resource)]
struct InfoWindowId(Entity);

#[derive(Component)]
struct OrbitCamera {
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

fn update_camera(
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

fn ui_metadata(mut egui_contexts: EguiContexts, info_id: Res<InfoWindowId>) {
    // Get Egui context for the info window
    if let Some(ctx) = egui_contexts.try_ctx_for_window_mut(info_id.0) {
        egui::Window::new("Car Specifications")
            .title_bar(false)
            .resizable(false)
            .frame(egui::Frame::none().fill(egui::Color32::from_rgba_unmultiplied(0, 26, 77, 153))) // Translucent blue background
            .anchor(egui::Align2::LEFT_TOP, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.label("🚗 McLaren 720S");
                ui.separator();
                ui.label("Drivetrain: MR");
                ui.label("Horsepower: 710 hp");
                ui.label("Release Year: 2017");
            });
    }
}

fn exit_on_esc(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut app_exit_events: EventWriter<bevy::app::AppExit>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        app_exit_events.send(bevy::app::AppExit);
    }
}

fn sync_windows(
    primary_query: Query<&Window, With<PrimaryWindow>>,
    mut info_query: Query<&mut Window, (Without<PrimaryWindow>, With<InfoWindowMarker>)>,
) {
    if let Ok(primary) = primary_query.get_single() {
        if let Ok(mut info_window) = info_query.get_single_mut() {
            if let WindowPosition::At(pos) = primary.position {
                // Position the metadata window slightly above the primary window
                info_window.position = WindowPosition::At(pos + IVec2::new(0, -220));
            }
        }
    }
}
