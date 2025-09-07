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

    fn legal_direction(&self, direction: Direction) -> Direction {
        let curr_direction = self.head_direction();
        match (curr_direction, direction) {
            (Direction::North, Direction::East) => direction,
            (Direction::North, Direction::West) => direction,
            (Direction::East, Direction::North) => direction,
            (Direction::East, Direction::South) => direction,
            (Direction::South, Direction::East) => direction,
            (Direction::South, Direction::West) => direction,
            (Direction::West, Direction::North) => direction,
            (Direction::West, Direction::South) => direction,
            _ => curr_direction,
        }
    }

    pub fn update_snake_position(&mut self, input: Direction) {
        self.move_snake(self.legal_direction(input));
    }

    fn move_snake(&mut self, direction: Direction) {
        self.shift_head(direction);
        if self.head_pos == self.food_pos {
            self.food_pos = Position::random_range(self.area.inner(Margin {
                horizontal: 1,
                vertical: 1,
            }))
        } else {
            self.shift_tail();
        }
    }

    fn shift_tail(&mut self) {
        let first_segment = self.segments.front_mut().unwrap();
        first_segment.length -= 1;
        if first_segment.length == 0 {
            self.segments.pop_front();
        }
    }

    pub fn touches_border(&self) -> bool {
        !self
            .area
            .inner(Margin {
                horizontal: 1,
                vertical: 1,
            })
            .contains(self.head_pos.into())
    }

    pub fn has_self_intersection(&self) -> bool {
        let (mut horizontal_counter, mut vertical_counter) = (0, 0);
        for segment in self.segments.iter().rev() {
            let (prev_horizontal_counter, prev_vertical_counter) =
                (horizontal_counter, vertical_counter);
            match segment.direction {
                Direction::North => {
                    vertical_counter += segment.length as isize;
                    if horizontal_counter == 0
                        && (vertical_counter == 0
                            || (vertical_counter < 0 && prev_vertical_counter > 0)
                            || (vertical_counter > 0 && prev_vertical_counter < 0))
                    {
                        return true;
                    }
                }
                Direction::East => {
                    horizontal_counter += segment.length as isize;
                    if vertical_counter == 0
                        && (horizontal_counter == 0
                            || (horizontal_counter < 0 && prev_horizontal_counter > 0)
                            || (horizontal_counter > 0 && prev_horizontal_counter < 0))
                    {
                        return true;
                    }
                }
                Direction::South => {
                    vertical_counter -= segment.length as isize;
                    if horizontal_counter == 0
                        && (vertical_counter == 0
                            || (vertical_counter < 0 && prev_vertical_counter > 0)
                            || (vertical_counter > 0 && prev_vertical_counter < 0))
                    {
                        return true;
                    }
                }
                Direction::West => {
                    horizontal_counter -= segment.length as isize;
                    if vertical_counter == 0
                        && (horizontal_counter == 0
                            || (horizontal_counter < 0 && prev_horizontal_counter > 0)
                            || (horizontal_counter > 0 && prev_horizontal_counter < 0))
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn shift_head(&mut self, direction: Direction) {
        self.head_pos.shift(direction);
        assert!(!self.segments.is_empty());
        let last_segment = self.segments.back_mut().unwrap();
        if last_segment.direction != direction {
            self.segments.push_back(Segment {
                direction, // here a copy happens
                length: 1,
            });
        } else {
            last_segment.length += 1;
        }
    }
}

impl Widget for &Snake {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf[self.food_pos].set_symbol("█").set_fg(self.food_color);
        let mut start_pos = self.head_pos; // todo: this can become a iterator
        for segment in self.segments.iter().rev() {
            for _ in 0..segment.length {
                buf[start_pos].set_symbol("█").set_fg(self.snake_color);
                start_pos.shift(segment.direction.opposite());
            }
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
            head_pos: ORIGIN,
            segments: VecDeque::from([
                Segment {
                    direction: Direction::North,
                    length: 1,
                },
                Segment {
                    direction: Direction::East,
                    length: 1,
                },
                Segment {
                    direction: Direction::South,
                    length: 1,
                },
                Segment {
                    direction: Direction::West,
                    length: 1,
                },
            ]),
            food_pos: ORIGIN,
            area: Rect::ZERO,
            food_color: Color::Black,
            snake_color: Color::Black,
        };
        assert!(snake.has_self_intersection())
    }

    #[test]
    fn rectangle_self_intersection() {
        let snake = Snake {
            head_pos: ORIGIN,
            segments: VecDeque::from([
                Segment {
                    direction: Direction::North,
                    length: 2,
                },
                Segment {
                    direction: Direction::East,
                    length: 1,
                },
                Segment {
                    direction: Direction::South,
                    length: 2,
                },
                Segment {
                    direction: Direction::West,
                    length: 1,
                },
            ]),
            food_pos: ORIGIN,
            area: Rect::ZERO,
            food_color: Color::Black,
            snake_color: Color::Black,
        };
        assert!(snake.has_self_intersection())
    }

    #[test]
    fn p_self_intersection() {
        let snake = Snake {
            head_pos: ORIGIN,
            segments: VecDeque::from([
                Segment {
                    direction: Direction::North,
                    length: 2,
                },
                Segment {
                    direction: Direction::East,
                    length: 1,
                },
                Segment {
                    direction: Direction::South,
                    length: 1,
                },
                Segment {
                    direction: Direction::West,
                    length: 1,
                },
            ]),
            food_pos: ORIGIN,
            area: Rect::ZERO,
            food_color: Color::Black,
            snake_color: Color::Black,
        };
        assert!(snake.has_self_intersection())
    }

    #[test]
    fn c_self_intersection() {
        let snake = Snake {
            head_pos: ORIGIN,
            segments: VecDeque::from([
                Segment {
                    direction: Direction::North,
                    length: 2,
                },
                Segment {
                    direction: Direction::East,
                    length: 1,
                },
                Segment {
                    direction: Direction::South,
                    length: 1,
                },
            ]),
            food_pos: ORIGIN,
            area: Rect::ZERO,
            food_color: Color::Black,
            snake_color: Color::Black,
        };
        assert!(!snake.has_self_intersection())
    }
}
