use crossterm::event::KeyCode;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    pub const fn from_key(key: KeyCode) -> Option<Self> {
        match key {
            KeyCode::Up => Some(Direction::North),
            KeyCode::Down => Some(Direction::South),
            KeyCode::Left => Some(Direction::West),
            KeyCode::Right => Some(Direction::East),
            _ => None,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

impl Position {
    pub const ORIGIN: Position = Position { x: 0, y: 0 };
    pub const fn shift(&mut self, direction: Direction) {
        match direction {
            Direction::North => self.y -= 1,
            Direction::East => self.x += 1, // todo: saturating sub?
            Direction::South => self.y += 1,
            Direction::West => self.x -= 1,
        }
    }
}

impl From<Position> for ratatui::layout::Position {
    fn from(value: Position) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<ratatui::layout::Position> for Position {
    fn from(value: ratatui::layout::Position) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}
