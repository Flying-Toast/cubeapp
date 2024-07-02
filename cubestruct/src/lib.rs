// TODO: better & generic cubestate inner implementation (don't make it Qiyi specific) (use group theory definition!!!!)

#[derive(Debug, Clone)]
pub struct CubeState {
    pub facelets: [Color; 54],
}

impl CubeState {
    pub fn new_solved() -> Self {
        use Color::*;
        Self {
            facelets: [
                White, White, White, White, White, White, White, White, White, Red, Red, Red, Red,
                Red, Red, Red, Red, Red, Green, Green, Green, Green, Green, Green, Green, Green,
                Green, Yellow, Yellow, Yellow, Yellow, Yellow, Yellow, Yellow, Yellow, Yellow,
                Orange, Orange, Orange, Orange, Orange, Orange, Orange, Orange, Orange, Blue, Blue,
                Blue, Blue, Blue, Blue, Blue, Blue, Blue,
            ],
        }
    }

    pub fn is_solved(&self) -> bool {
        self.facelets == Self::new_solved().facelets
    }

    /// Returns the colors of the facelets on the face whose center piece is `center_color`.
    fn face_colors(&self, center_color: Color) -> [Color; 9] {
        let idx = center_color.state_index();
        self.facelets[idx..idx + 9].try_into().unwrap()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Color {
    Orange,
    Red,
    Yellow,
    White,
    Green,
    Blue,
}

impl Color {
    pub fn state_index(self) -> usize {
        match self {
            Self::White => 0,
            Self::Red => 9,
            Self::Green => 18,
            Self::Yellow => 27,
            Self::Orange => 36,
            Self::Blue => 45,
        }
    }
}
