#[derive(Debug, Clone)]
pub struct MouseButtonState {
    pub clicked: bool,
    pub pressed: bool,
}

#[derive(Debug, Clone)]
pub struct MouseState {
    pub x: i16,
    pub y: i16,
    pub flags: MouseFlags,
}

#[derive(Clone)]
pub struct MouseFlags(u8);

impl MouseFlags {
    const LEFT_BUTTON: usize = 0;
    const RIGHT_BUTTON: usize = 1;
    const MIDDLE_BUTTON: usize = 2;
    const ALWAYS_1: usize = 3;
    const X_SIGN: usize = 4;
    const Y_SIGN: usize = 5;
    const X_OVERFLOW: usize = 6;
    const Y_OVERFLOW: usize = 7;

    pub const fn new(flags: u8) -> Self {
        Self(flags)
    }

    #[inline]
    pub fn as_u8(&self) -> u8 {
        self.0
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        self.0 & (1 << Self::ALWAYS_1) != 0
    }

    #[inline]
    pub fn left_button(&self) -> bool {
        self.0 & (1 << Self::LEFT_BUTTON) != 0
    }
    #[inline]
    pub fn right_button(&self) -> bool {
        self.0 & (1 << Self::RIGHT_BUTTON) != 0
    }
    #[inline]
    pub fn middle_button(&self) -> bool {
        self.0 & (1 << Self::MIDDLE_BUTTON) != 0
    }

    #[inline]
    pub fn x_sign(&self) -> bool {
        self.0 & (1 << Self::X_SIGN) != 0
    }
    #[inline]
    pub fn y_sign(&self) -> bool {
        self.0 & (1 << Self::Y_SIGN) != 0
    }

    #[inline]
    pub fn x_overflow(&self) -> bool {
        self.0 & (1 << Self::X_OVERFLOW) != 0
    }
    #[inline]
    pub fn y_overflow(&self) -> bool {
        self.0 & (1 << Self::Y_OVERFLOW) != 0
    }
}

impl core::fmt::Debug for MouseFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("MouseFlags")
            .field("left_button", &self.left_button())
            .field("right_button", &self.right_button())
            .field("middle_button", &self.middle_button())
            .field("x_sign", &self.x_sign())
            .field("y_sign", &self.y_sign())
            .field("x_overflow", &self.x_overflow())
            .field("y_overflow", &self.y_overflow())
            .finish()
    }
}
