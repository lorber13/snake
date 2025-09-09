use std::collections::VecDeque;

use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Margin, Rect},
    style::Color,
    widgets::{Block, BorderType, Widget},
};

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
    fn random_range(area: Rect) -> Self {
        Self {
            x: rand::random_range(area.x..area.width),
            y: rand::random_range(area.y..area.height),
        }
    }
    const fn shift(&mut self, direction: Direction) {
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

pub struct Snake {
    direction: Direction,
    shape: VecDeque<Position>,
    food_pos: Position,
    area: Rect,
    food_color: Color,
    shape_color: Color,
}

impl Snake {
    pub fn new(
        direction: Direction,
        shape: VecDeque<Position>,
        food_pos: Position,
        area: Rect,
        food_color: Color,
        shape_color: Color,
    ) -> Self {
        Self {
            direction,
            shape,
            food_pos,
            area,
            food_color,
            shape_color,
        }
    }

    pub fn head_direction(&self) -> Direction {
        self.direction
    }

    fn legal_direction(&self, direction: Direction) -> Direction {
        match (self.direction, direction) {
            (Direction::North, Direction::East) => direction,
            (Direction::North, Direction::West) => direction,
            (Direction::East, Direction::North) => direction,
            (Direction::East, Direction::South) => direction,
            (Direction::South, Direction::East) => direction,
            (Direction::South, Direction::West) => direction,
            (Direction::West, Direction::North) => direction,
            (Direction::West, Direction::South) => direction,
            _ => self.direction,
        }
    }

    fn head_pos(&self) -> Position {
        *self.shape.back().unwrap()
    }

    pub fn update_snake_position(&mut self, input: Direction) {
        self.move_snake(self.legal_direction(input));
    }

    fn move_snake(&mut self, direction: Direction) {
        self.direction = direction;
        self.shift_head(direction);
        if self.head_pos() == self.food_pos {
            self.food_pos = Position::random_range(self.area.inner(Margin {
                horizontal: 1,
                vertical: 1,
            }))
        } else {
            self.shift_tail();
        }
    }

    fn shift_tail(&mut self) {
        self.shape.pop_front();
    }

    pub fn touches_border(&self) -> bool {
        !self
            .area
            .inner(Margin {
                horizontal: 1,
                vertical: 1,
            })
            .contains(self.head_pos().into())
    }

    pub fn has_self_intersection(&self) -> bool {
        self.shape
            .iter()
            .rev()
            .skip(1)
            .any(|pos| self.head_pos() == *pos)
    }

    fn shift_head(&mut self, direction: Direction) {
        let mut new_pos = self.head_pos();
        new_pos.shift(direction);
        self.shape.push_back(new_pos);
    }
}

impl Widget for &Snake {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf[self.food_pos].set_symbol("█").set_fg(self.food_color);
        for pos in &self.shape {
            buf[*pos].set_symbol("█").set_fg(self.shape_color);
        }
        Block::bordered()
            .border_type(BorderType::Thick)
            .render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    const ORIGIN: Position = Position { x: 0, y: 0 };

    #[test]
    fn square_self_intersection() {
        let snake = Snake {
            direction: Direction::East,
            shape: VecDeque::from([
                Position { x: 0, y: 0 },
                Position { x: 1, y: 0 },
                Position { x: 1, y: 1 },
                Position { x: 1, y: 0 },
                Position { x: 0, y: 0 },
            ]),
            food_pos: ORIGIN,
            area: Rect::ZERO,
            food_color: Color::Black,
            shape_color: Color::Black,
        };
        assert!(snake.has_self_intersection())
    }

    #[test]
    fn rectangle_self_intersection() {
        let snake = Snake {
            direction: Direction::East,
            shape: VecDeque::from([
                Position { x: 0, y: 0 },
                Position { x: 1, y: 0 },
                Position { x: 1, y: 1 },
                Position { x: 1, y: 2 },
                Position { x: 0, y: 2 },
                Position { x: 0, y: 1 },
                Position { x: 0, y: 0 },
            ]),
            food_pos: ORIGIN,
            area: Rect::ZERO,
            food_color: Color::Black,
            shape_color: Color::Black,
        };
        assert!(snake.has_self_intersection())
    }

    #[test]
    fn p_self_intersection() {
        let snake = Snake {
            direction: Direction::East,
            shape: VecDeque::from([
                Position { x: 0, y: 0 },
                Position { x: 1, y: 0 },
                Position { x: 2, y: 0 },
                Position { x: 3, y: 0 },
                Position { x: 3, y: 1 },
                Position { x: 3, y: 2 },
                Position { x: 2, y: 2 },
                Position { x: 1, y: 2 },
                Position { x: 1, y: 1 },
                Position { x: 1, y: 1 },
                Position { x: 1, y: 0 },
            ]),
            food_pos: ORIGIN,
            area: Rect::ZERO,
            food_color: Color::Black,
            shape_color: Color::Black,
        };
        assert!(snake.has_self_intersection())
    }

    #[test]
    fn c_self_intersection() {
        let snake = Snake {
            direction: Direction::East,
            shape: VecDeque::from([
                Position { x: 2, y: 0 },
                Position { x: 1, y: 0 },
                Position { x: 0, y: 0 },
                Position { x: 0, y: 1 },
                Position { x: 0, y: 2 },
                Position { x: 0, y: 3 },
                Position { x: 1, y: 3 },
                Position { x: 2, y: 3 },
            ]),
            food_pos: ORIGIN,
            area: Rect::ZERO,
            food_color: Color::Black,
            shape_color: Color::Black,
        };
        assert!(!snake.has_self_intersection())
    }
}
