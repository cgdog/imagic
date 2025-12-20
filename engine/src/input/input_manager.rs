use crate::math::Vec2;

pub struct MouseData {
    // pub last_mouse_pos: Vec2,
    pub cur_mouse_pos: Vec2,
    pub last_mouse_pos: Vec2,
    pub is_left_button_down: bool,
    pub is_right_button_down: bool,
    pub is_middle_button_down: bool,
}

impl Default for MouseData {
    fn default() -> Self {
        Self {
            // last_mouse_pos: Vec2::ZERO,
            cur_mouse_pos: Vec2::ZERO,
            last_mouse_pos: Vec2::ZERO,
            is_left_button_down: false,
            is_right_button_down: false,
            is_middle_button_down: false,
        }
    }
}

impl MouseData {
    pub fn new() -> Self {
        Self::default()
    }
}

pub struct InputManager {
   pub mouse_data: MouseData,

}

impl Default for InputManager {
    fn default() -> Self {
        Self {
            mouse_data: MouseData::default(),
        }
    }
}

impl InputManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on_mouse_move(&mut self, new_pos: &Vec2) {
        // self.mouse_data.last_mouse_pos = self.mouse_data.cur_mouse_pos;
        self.set_cur_mouse_pos(new_pos);
    }

    pub fn on_mouse_left_button_down(&mut self, new_pos: &Vec2) {
        self.mouse_data.is_left_button_down = true;
        self.set_cur_mouse_pos(new_pos);
        self.set_last_mouse_pos(new_pos);
    }

    pub fn on_mouse_left_button_up(&mut self, new_pos: &Vec2) {
        self.mouse_data.is_left_button_down = false;
        self.set_cur_mouse_pos(new_pos);
    }

    pub fn on_mouse_right_button_down(&mut self, new_pos: &Vec2) {
        self.mouse_data.is_right_button_down = true;
        self.set_cur_mouse_pos(new_pos);
        self.set_last_mouse_pos(new_pos);
    }

    pub fn on_mouse_right_button_up(&mut self, new_pos: &Vec2) {
        self.mouse_data.is_right_button_down = false;
        self.set_cur_mouse_pos(new_pos);
    }

    pub fn on_mouse_middle_button_down(&mut self, new_pos: &Vec2) {
        self.mouse_data.is_middle_button_down = true;
        self.set_cur_mouse_pos(new_pos);
        self.set_last_mouse_pos(new_pos);
    }

    pub fn on_mouse_middle_button_up(&mut self, new_pos: &Vec2) {
        self.mouse_data.is_middle_button_down = false;
        self.set_cur_mouse_pos(new_pos);
    }

    fn set_cur_mouse_pos(&mut self, new_pos: &Vec2) {
        self.mouse_data.cur_mouse_pos.x = new_pos.x;
        self.mouse_data.cur_mouse_pos.y = new_pos.y;
    }

     fn set_last_mouse_pos(&mut self, new_pos: &Vec2) {
        self.mouse_data.last_mouse_pos.x = new_pos.x;
        self.mouse_data.last_mouse_pos.y = new_pos.y;
    }
}