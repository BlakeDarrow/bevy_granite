use bevy::{
    color::Color,
    ecs::{component::Component, resource::Resource},
    math::{bool, Quat, Vec2, Vec3},
};

#[derive(Resource, Component, PartialEq, Clone)]
pub struct DragState {
    pub dragging: bool,
    pub raycast_position: Vec3,
    pub initial_cursor_position: Vec2,
    pub initial_selection_rotation: Quat,
    pub gizmo_position: Vec3,
    pub initial_gizmo_rotation: Quat,
    pub locked_axis: Option<GizmoAxis>,
    pub drag_ended: bool,
    pub accumulated_angle: f32,
    pub last_snapped: f32,
    pub prev_hit_dir: Vec3,
}

impl Default for DragState {
    fn default() -> Self {
        Self {
            dragging: false,
            raycast_position: Vec3::ZERO,
            initial_cursor_position: Vec2::ZERO,
            initial_gizmo_rotation: Quat::default(),
            gizmo_position: Vec3::ZERO,
            initial_selection_rotation: Quat::default(),
            locked_axis: Some(GizmoAxis::None),
            drag_ended: true,
            accumulated_angle: 0.,
            last_snapped: 0.,
            prev_hit_dir: Vec3::NAN,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum GizmoAxis {
    X,
    Y,
    Z,
    All,
    #[default]
    None,
}

impl GizmoAxis {
    pub fn to_vec3(self) -> Vec3 {
        match self {
            GizmoAxis::X => Vec3::X,
            GizmoAxis::Y => Vec3::Y,
            GizmoAxis::Z => Vec3::Z,
            GizmoAxis::All => Vec3::ONE,
            GizmoAxis::None => Vec3::ZERO,
        }
    }

    pub fn rotation(self) -> Quat {
        match self {
            GizmoAxis::X => Quat::from_rotation_z((90f32).to_radians()),
            GizmoAxis::Y => Quat::IDENTITY,
            GizmoAxis::Z => Quat::from_rotation_x((90f32).to_radians()),
            GizmoAxis::None | GizmoAxis::All => Quat::IDENTITY,
        }
    }

    pub fn color(self) -> Color {
        match self {
            GizmoAxis::X => Color::linear_rgba(1., 0., 0., 1.),
            GizmoAxis::Y => Color::linear_rgba(0., 1., 0., 1.),
            GizmoAxis::Z => Color::linear_rgba(0., 0., 1., 1.),
            GizmoAxis::All => Color::linear_rgba(1., 1., 1., 1.),
            GizmoAxis::None => Color::linear_rgba(0., 0., 0., 1.),
        }
    }

    pub fn plane(self) -> (GizmoAxis, GizmoAxis) {
        match self {
            GizmoAxis::X => (GizmoAxis::Y, GizmoAxis::Z),
            GizmoAxis::Y => (GizmoAxis::X, GizmoAxis::Z),
            GizmoAxis::Z => (GizmoAxis::X, GizmoAxis::Y),
            GizmoAxis::All => (GizmoAxis::All, GizmoAxis::All),
            GizmoAxis::None => (GizmoAxis::None, GizmoAxis::None),
        }
    }

    pub fn plane_as_vec3(self) -> Vec3 {
        self.plane().0.to_vec3() + self.plane().1.to_vec3()
    }
}
