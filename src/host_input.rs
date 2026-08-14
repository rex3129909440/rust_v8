#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct HostClickInput {
    pub client_x: f64,
    pub client_y: f64,
    pub button: i16,
    pub ctrl_key: bool,
    pub shift_key: bool,
    pub alt_key: bool,
    pub meta_key: bool,
}

impl HostClickInput {
    pub fn primary(client_x: f64, client_y: f64) -> Self {
        Self {
            client_x,
            client_y,
            button: 0,
            ctrl_key: false,
            shift_key: false,
            alt_key: false,
            meta_key: false,
        }
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        if !self.client_x.is_finite() || !self.client_y.is_finite() {
            return Err("host click coordinates must be finite".to_owned());
        }
        if !(0..=4).contains(&self.button) {
            return Err("host click button must be between 0 and 4".to_owned());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct HostKeyboardInput {
    pub key: String,
    pub code: String,
    pub text: Option<String>,
    pub location: u32,
    pub repeat: bool,
    pub ctrl_key: bool,
    pub shift_key: bool,
    pub alt_key: bool,
    pub meta_key: bool,
}

impl HostKeyboardInput {
    pub fn printable(key: impl Into<String>, code: impl Into<String>) -> Self {
        let key = key.into();
        Self {
            text: Some(key.clone()),
            key,
            code: code.into(),
            location: 0,
            repeat: false,
            ctrl_key: false,
            shift_key: false,
            alt_key: false,
            meta_key: false,
        }
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.key.is_empty() {
            return Err("host keyboard key cannot be empty".to_owned());
        }
        if self.key.len() > 256 || self.code.len() > 256 {
            return Err("host keyboard key/code exceeds 256 UTF-8 bytes".to_owned());
        }
        if self.text.as_ref().is_some_and(|value| value.len() > 65_536) {
            return Err("host keyboard text exceeds 65536 UTF-8 bytes".to_owned());
        }
        if self.location > 3 {
            return Err("host keyboard location must be between 0 and 3".to_owned());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct HostWheelInput {
    pub client_x: f64,
    pub client_y: f64,
    pub delta_x: f64,
    pub delta_y: f64,
    pub delta_z: f64,
    pub delta_mode: u32,
    pub ctrl_key: bool,
    pub shift_key: bool,
    pub alt_key: bool,
    pub meta_key: bool,
}

impl HostWheelInput {
    pub fn pixels(client_x: f64, client_y: f64, delta_x: f64, delta_y: f64) -> Self {
        Self {
            client_x,
            client_y,
            delta_x,
            delta_y,
            delta_z: 0.0,
            delta_mode: 0,
            ctrl_key: false,
            shift_key: false,
            alt_key: false,
            meta_key: false,
        }
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        if ![
            self.client_x,
            self.client_y,
            self.delta_x,
            self.delta_y,
            self.delta_z,
        ]
        .into_iter()
        .all(f64::is_finite)
        {
            return Err("host wheel coordinates and deltas must be finite".to_owned());
        }
        if self.delta_mode > 2 {
            return Err("host wheel delta_mode must be between 0 and 2".to_owned());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct HostDragPoint {
    pub client_x: f64,
    pub client_y: f64,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct HostDragInput {
    pub points: Vec<HostDragPoint>,
    pub ctrl_key: bool,
    pub shift_key: bool,
    pub alt_key: bool,
    pub meta_key: bool,
}

impl HostDragInput {
    pub fn between(start_x: f64, start_y: f64, end_x: f64, end_y: f64) -> Self {
        Self {
            points: vec![
                HostDragPoint {
                    client_x: start_x,
                    client_y: start_y,
                },
                HostDragPoint {
                    client_x: end_x,
                    client_y: end_y,
                },
            ],
            ctrl_key: false,
            shift_key: false,
            alt_key: false,
            meta_key: false,
        }
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        if !(2..=4096).contains(&self.points.len()) {
            return Err("host drag requires between 2 and 4096 points".to_owned());
        }
        if !self
            .points
            .iter()
            .all(|point| point.client_x.is_finite() && point.client_y.is_finite())
        {
            return Err("host drag coordinates must be finite".to_owned());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum HostTouchPhase {
    Start,
    Move,
    End,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct HostTouchInput {
    pub phase: HostTouchPhase,
    pub identifier: i32,
    pub client_x: f64,
    pub client_y: f64,
    pub radius_x: f64,
    pub radius_y: f64,
    pub rotation_angle: f64,
    pub force: f64,
    pub ctrl_key: bool,
    pub shift_key: bool,
    pub alt_key: bool,
    pub meta_key: bool,
}

impl HostTouchInput {
    pub fn start(identifier: i32, client_x: f64, client_y: f64) -> Self {
        Self {
            phase: HostTouchPhase::Start,
            identifier,
            client_x,
            client_y,
            radius_x: 1.0,
            radius_y: 1.0,
            rotation_angle: 0.0,
            force: 1.0,
            ctrl_key: false,
            shift_key: false,
            alt_key: false,
            meta_key: false,
        }
    }

    pub fn end(identifier: i32, client_x: f64, client_y: f64) -> Self {
        Self {
            phase: HostTouchPhase::End,
            ..Self::start(identifier, client_x, client_y)
        }
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        if ![
            self.client_x,
            self.client_y,
            self.radius_x,
            self.radius_y,
            self.rotation_angle,
            self.force,
        ]
        .into_iter()
        .all(f64::is_finite)
        {
            return Err("host touch coordinates and geometry must be finite".to_owned());
        }
        if self.radius_x < 0.0 || self.radius_y < 0.0 {
            return Err("host touch radii cannot be negative".to_owned());
        }
        if !(0.0..=1.0).contains(&self.force) {
            return Err("host touch force must be between 0 and 1".to_owned());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum HostPenPhase {
    Hover,
    Down,
    Move,
    Up,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct HostPenInput {
    pub phase: HostPenPhase,
    pub client_x: f64,
    pub client_y: f64,
    pub width: f64,
    pub height: f64,
    pub pressure: f64,
    pub tangential_pressure: f64,
    pub tilt_x: i32,
    pub tilt_y: i32,
    pub twist: u32,
    pub button: i16,
    pub ctrl_key: bool,
    pub shift_key: bool,
    pub alt_key: bool,
    pub meta_key: bool,
}

impl HostPenInput {
    pub fn hover(client_x: f64, client_y: f64) -> Self {
        Self {
            phase: HostPenPhase::Hover,
            client_x,
            client_y,
            width: 1.0,
            height: 1.0,
            pressure: 0.0,
            tangential_pressure: 0.0,
            tilt_x: 0,
            tilt_y: 0,
            twist: 0,
            button: 0,
            ctrl_key: false,
            shift_key: false,
            alt_key: false,
            meta_key: false,
        }
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        if ![
            self.client_x,
            self.client_y,
            self.width,
            self.height,
            self.pressure,
            self.tangential_pressure,
        ]
        .into_iter()
        .all(f64::is_finite)
        {
            return Err("host pen coordinates and geometry must be finite".to_owned());
        }
        if self.width < 0.0 || self.height < 0.0 {
            return Err("host pen width and height cannot be negative".to_owned());
        }
        if !(0.0..=1.0).contains(&self.pressure) {
            return Err("host pen pressure must be between 0 and 1".to_owned());
        }
        if !(-1.0..=1.0).contains(&self.tangential_pressure) {
            return Err("host pen tangential_pressure must be between -1 and 1".to_owned());
        }
        if !(-90..=90).contains(&self.tilt_x) || !(-90..=90).contains(&self.tilt_y) {
            return Err("host pen tilt values must be between -90 and 90".to_owned());
        }
        if self.twist > 359 {
            return Err("host pen twist must be between 0 and 359".to_owned());
        }
        if !(0..=5).contains(&self.button) {
            return Err("host pen button must be between 0 and 5".to_owned());
        }
        Ok(())
    }
}
