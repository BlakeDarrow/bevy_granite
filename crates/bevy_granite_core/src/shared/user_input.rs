use bevy::{math::bool, prelude::*};
use bevy_egui::EguiContexts;

#[derive(Resource, Debug, Default)]
pub struct CursorWindowPos {
    pub position: Vec2,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InputTypes {
    Mouse(MouseButton),
    Button(KeyCode),
}

#[derive(PartialEq, Default, Resource, Clone, Debug)]
pub struct UserInput {
    pub current_button_inputs: Vec<InputTypes>,
    pub mouse_pos: Vec2,
    pub mouse_over_egui: bool,
    pub mouse_left: UserButtonState,
    pub mouse_right: UserButtonState,
    pub mouse_middle: UserButtonState,
    pub shift_left: UserButtonState,
    pub ctrl_left: UserButtonState,
    pub alt_left: UserButtonState,
    pub key_w: UserButtonState,
    pub key_u: UserButtonState,
    pub key_f1: UserButtonState,
    pub key_f2: UserButtonState,
    pub key_h: UserButtonState,
    pub key_e: UserButtonState,
    pub key_s: UserButtonState,
    pub key_o: UserButtonState,
    pub key_a: UserButtonState,
    pub key_d: UserButtonState,
    pub key_q: UserButtonState,
    pub key_z: UserButtonState,
    pub key_f: UserButtonState,
    pub key_p: UserButtonState,
    pub key_delete: UserButtonState,
    pub key_space: UserButtonState,
}

#[derive(PartialEq, Default, Clone, Copy, Debug)]
pub struct UserButtonState {
    pub just_pressed: bool,
    pub pressed: bool,
    pub just_released: bool,
    pub any: bool,
}

impl UserButtonState {
    pub fn update_mouse(
        &mut self,
        input: &ButtonInput<MouseButton>,
        button: MouseButton,
        user_input: &mut UserInput,
    ) {
        self.just_pressed = input.just_pressed(button);
        self.pressed = input.pressed(button);
        self.just_released = input.just_released(button);
        self.any = self.just_pressed || self.pressed || self.just_released;

        if self.just_pressed || self.pressed {
            user_input
                .current_button_inputs
                .push(InputTypes::Mouse(button));
        }
    }

    pub fn update_key(
        &mut self,
        input: &ButtonInput<KeyCode>,
        button: KeyCode,
        user_input: &mut UserInput,
    ) {
        self.just_pressed = input.just_pressed(button);
        self.pressed = input.pressed(button);
        self.just_released = input.just_released(button);
        self.any = self.just_pressed || self.pressed || self.just_released;

        if self.just_pressed || self.pressed {
            user_input
                .current_button_inputs
                .push(InputTypes::Button(button));
        }

        if self.just_released {
            user_input
                .current_button_inputs
                .retain(|i| i != &InputTypes::Button(button));
        }
    }
}

pub fn capture_input_events(
    mut user_input: ResMut<UserInput>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    windows: Query<&Window>,
) {
    user_input.current_button_inputs.clear();

    // Capture mouse position
    if let Ok(window) = windows.single() {
        if let Some(cursor_pos) = window.cursor_position() {
            user_input.mouse_pos = cursor_pos;
        }
    }

    // Split borrow manually by cloning pieces out
    let mut mouse_left = user_input.mouse_left;
    let mut mouse_right = user_input.mouse_right;
    let mut mouse_middle = user_input.mouse_middle;
    let mut shift_left = user_input.shift_left;
    let mut ctrl_left = user_input.ctrl_left;
    let mut alt_left = user_input.alt_left;
    let mut key_w = user_input.key_w;
    let mut key_o = user_input.key_o;
    let mut key_p = user_input.key_p;
    let mut key_u = user_input.key_u;
    let mut key_h = user_input.key_h;
    let mut key_e = user_input.key_e;
    let mut key_s = user_input.key_s;
    let mut key_a = user_input.key_a;
    let mut key_d = user_input.key_d;
    let mut key_f1 = user_input.key_f1;
    let mut key_f2 = user_input.key_f2;
    let mut key_delete = user_input.key_delete;
    let mut key_q = user_input.key_q;
    let mut key_z = user_input.key_z;
    let mut key_f = user_input.key_f;
    let mut key_space = user_input.key_space;

    // Update state
    mouse_left.update_mouse(&mouse_input, MouseButton::Left, &mut user_input);
    mouse_right.update_mouse(&mouse_input, MouseButton::Right, &mut user_input);
    mouse_middle.update_mouse(&mouse_input, MouseButton::Middle, &mut user_input);
    shift_left.update_key(&keyboard_input, KeyCode::ShiftLeft, &mut user_input);
    ctrl_left.update_key(&keyboard_input, KeyCode::ControlLeft, &mut user_input);
    alt_left.update_key(&keyboard_input, KeyCode::AltLeft, &mut user_input);
    key_w.update_key(&keyboard_input, KeyCode::KeyW, &mut user_input);
    key_p.update_key(&keyboard_input, KeyCode::KeyP, &mut user_input);
    key_z.update_key(&keyboard_input, KeyCode::KeyZ, &mut user_input);
    key_delete.update_key(&keyboard_input, KeyCode::Delete, &mut user_input);
    key_u.update_key(&keyboard_input, KeyCode::KeyU, &mut user_input);
    key_e.update_key(&keyboard_input, KeyCode::KeyE, &mut user_input);
    key_s.update_key(&keyboard_input, KeyCode::KeyS, &mut user_input);
    key_a.update_key(&keyboard_input, KeyCode::KeyA, &mut user_input);
    key_d.update_key(&keyboard_input, KeyCode::KeyD, &mut user_input);
    key_o.update_key(&keyboard_input, KeyCode::KeyO, &mut user_input);
    key_h.update_key(&keyboard_input, KeyCode::KeyH, &mut user_input);
    key_f1.update_key(&keyboard_input, KeyCode::F1, &mut user_input);
    key_f2.update_key(&keyboard_input, KeyCode::F2, &mut user_input);
    key_q.update_key(&keyboard_input, KeyCode::KeyQ, &mut user_input);
    key_f.update_key(&keyboard_input, KeyCode::KeyF, &mut user_input);
    key_space.update_key(&keyboard_input, KeyCode::Space, &mut user_input);

    // Write the updated states back
    user_input.key_delete = key_delete;
    user_input.mouse_left = mouse_left;
    user_input.mouse_right = mouse_right;
    user_input.mouse_middle = mouse_middle;
    user_input.shift_left = shift_left;
    user_input.ctrl_left = ctrl_left;
    user_input.key_w = key_w;
    user_input.key_p = key_p;
    user_input.key_u = key_u;
    user_input.key_f1 = key_f1;
    user_input.key_f2 = key_f2;
    user_input.key_f = key_f;
    user_input.key_e = key_e;
    user_input.key_o = key_o;
    user_input.key_h = key_h;
    user_input.key_s = key_s;
    user_input.key_a = key_a;
    user_input.key_d = key_d;
    user_input.key_q = key_q;
    user_input.key_z = key_z;
    user_input.key_space = key_space;
    user_input.alt_left = alt_left;

    if let Ok(ctx) = contexts.ctx_mut() {
        // Check if the mouse is over an Egui area
        user_input.mouse_over_egui = ctx.is_pointer_over_area();
    } else {
        // If we can't get the context, assume not over Egui
        user_input.mouse_over_egui = false;
    }
}

pub fn update_mouse_pos(windows: Query<&Window>, mut cursor: ResMut<CursorWindowPos>) {
    let Ok(window) = windows.single() else {
        // todo: log to user multiple windows not supported
        return;
    };
    if let Some(cursor_position) = window.cursor_position() {
        cursor.position = Vec2::new(
            (cursor_position.x / window.width() * 2.0) - 1.0,
            -((cursor_position.y / window.height() * 2.0) - 1.0),
        );
    }
}

pub fn mouse_to_world_delta(
    cursor_position: Vec2,
    initial_cursor_position: Vec2,
    camera_transform: &Transform,
    plane_normal: Option<Vec3>,
) -> Vec3 {
    let cursor_delta_2d = cursor_position - initial_cursor_position;

    if cursor_delta_2d == Vec2::ZERO {
        return Vec3::ZERO;
    }
    let camera_right = camera_transform.rotation * Vec3::X;
    let camera_up = camera_transform.rotation * Vec3::Y;
    let raw_world_delta = (camera_right * cursor_delta_2d.x) + (camera_up * cursor_delta_2d.y);

    if let Some(plane_normal) = plane_normal {
        Vec3::new(
            if plane_normal.x.abs() > 0.5 {
                raw_world_delta.x
            } else {
                0.0
            },
            if plane_normal.y.abs() > 0.5 {
                raw_world_delta.y
            } else {
                0.0
            },
            if plane_normal.z.abs() > 0.5 {
                raw_world_delta.z
            } else {
                0.0
            },
        )
    } else {
        raw_world_delta
    }
}
