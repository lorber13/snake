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
    const fn opposite(self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::East => Direction::West,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
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
            x: rand::random_range(0..area.width),
            y: rand::random_range(0..area.height),
        }
    }
    const fn shift(&mut self, direction: &Direction) {
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

pub struct Segment {
    pub direction: Direction,
    pub length: usize, // todo: define real size
}

pub struct Snake {
    head_pos: Position,
    segments: VecDeque<Segment>,
    food_pos: Position,
    area: Rect,
    food_color: Color,
    snake_color: Color,
}

#[derive(Debug)]
pub struct OutOfRange;

impl Snake {
    pub fn new(
        area: Rect,
        segments: VecDeque<Segment>,
        food_pos: Position,
        head_pos: Position,
        snake_color: Color,
        food_color: Color,
    ) -> Self {
        Self {
            area,
            segments,
            food_pos,
            head_pos,
            snake_color,
            food_color,
        }
    }

    pub fn head_direction(&self) -> Direction {
        assert!(!self.segments.is_empty());
        self.segments.back().unwrap().direction
    }

    pub fn move_snake(&mut self, direction: &Direction) -> Result<(), OutOfRange> {
        self.shift_head(direction)?;
        if self.head_pos == self.food_pos {
            self.food_pos = Position::random_range(self.area)
        } else {
            self.shift_tail();
        }
        Ok(())
    }

    fn shift_tail(&mut self) {
        let first_segment = self.segments.front_mut().unwrap();
        first_segment.length -= 1;
        if first_segment.length == 0 {
            self.segments.pop_front();
        }
    }

    fn shift_head(&mut self, direction: &Direction) -> Result<(), OutOfRange> {
        let mut new_pos = self.head_pos;
        new_pos.shift(direction);
        if !self
            .area
            .inner(Margin {
                horizontal: 1,
                vertical: 1,
            })
            .contains(new_pos.into())
        {
            return Err(OutOfRange);
        }

        self.head_pos.shift(direction);
        assert!(!self.segments.is_empty());
        let last_segment = self.segments.back_mut().unwrap();
        if last_segment.direction != *direction {
            self.segments.push_back(Segment {
                direction: *direction, // here a copy happens
                length: 1,
            });
        } else {
            last_segment.length += 1;
        }
        Ok(())
    }
}

impl Widget for &Snake {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf[self.food_pos]
            // .set_symbol(&self.food_pos.character().to_string())
            .set_symbol("█")
            .set_fg(self.food_color);
        let mut start_pos = self.head_pos; // todo: this can become a iterator
        for segment in self.segments.iter().rev() {
            for _ in 0..segment.length {
                buf[start_pos].set_symbol("█").set_fg(self.snake_color);
                start_pos.shift(&segment.direction.opposite());
            }
        }
        Block::bordered()
            .border_type(BorderType::Thick)
            .render(area, buf);
    }
}
