use std::collections::VecDeque;

use rand::{rng, seq::IndexedRandom};
use ratatui::{
    buffer::Buffer,
    layout::{Margin, Rect},
    style::Color,
    widgets::{Block, BorderType, Widget},
};

use super::grid::{Direction, Position};

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

    fn area_no_border(&self) -> Rect {
        self.area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        })
    }

    fn update_food_pos(&mut self) {
        let mut available_positions =
            Vec::with_capacity(((self.area.width - 2) * (self.area.height - 2)) as usize);
        for char_pos in self.area_no_border().positions() {
            let pos = char_pos.into();
            if !self.shape.contains(&pos) {
                available_positions.push(pos);
            }
        }
        self.food_pos = *available_positions.choose(&mut rng()).unwrap();
    }

    fn move_snake(&mut self, direction: Direction) {
        self.direction = direction;
        self.shift_head(direction);
        if self.head_pos() == self.food_pos {
            self.update_food_pos();
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
    use super::*;

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
            food_pos: Position::ORIGIN,
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
            food_pos: Position::ORIGIN,
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
            food_pos: Position::ORIGIN,
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
            food_pos: Position::ORIGIN,
            area: Rect::ZERO,
            food_color: Color::Black,
            shape_color: Color::Black,
        };
        assert!(!snake.has_self_intersection())
    }
}
