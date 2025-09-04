use std::{
    collections::VecDeque,
    io, thread,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame, buffer::Buffer, layout::Rect, style::Color, widgets::Widget,
};

#[derive(PartialEq, Eq, Clone, Copy)]
enum Direction {
    North,
    East,
    South,
    West,
}

#[derive(PartialEq, Eq, Clone, Copy)]
struct Position {
    x: u16,
    y: u16,
}

impl Position {
    const fn shift(&mut self, direction: &Direction) {
        match direction {
            Direction::North => self.y -= 1,
            Direction::East => self.x += 1,
            Direction::South => self.y += 1,
            Direction::West => self.x -= 1,
        }
    }

    const fn shift_opposite(&mut self, direction: &Direction) {
        match direction {
            Direction::North => self.y += 1,
            Direction::East => self.x -= 1,
            Direction::South => self.y -= 1,
            Direction::West => self.x += 1,
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

struct Segment {
    direction: Direction,
    length: usize, // todo: define real size
}

pub struct App {
    head_pos: Position,
    segments: VecDeque<Segment>,
    exit: bool,
    direction: Direction,
    food_pos: Position,
}

impl Default for App {
    fn default() -> Self {
        App {
            head_pos: Position { x: 14, y: 29 },
            exit: false,
            direction: Direction::East,
            food_pos: Position { x: 17, y: 3 }, // todo: randomize
            segments: VecDeque::from([
                Segment {
                    direction: Direction::West,
                    length: 1,
                },
                Segment {
                    direction: Direction::North,
                    length: 5,
                },
                Segment {
                    direction: Direction::East,
                    length: 2,
                },
                Segment {
                    direction: Direction::South,
                    length: 3,
                },
                Segment {
                    direction: Direction::East,
                    length: 3,
                },
            ]),
        }
    }
}

const FRAME_DURATION: Duration = Duration::from_millis(100);

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        // the first frame will not update the position
        let mut timer = Instant::now();
        let mut events = Vec::new();
        while !self.exit {
            // DRAW
            terminal.draw(|frame| self.draw(frame))?;

            if timer.elapsed() < FRAME_DURATION {
                thread::sleep(FRAME_DURATION - timer.elapsed());
            }
            timer = Instant::now();

            events.clear();
            while event::poll(Duration::from_secs(0))? {
                events.push(event::read()?); // todo: probably a vector isn't needed here
            }

            // STATE
            let mut key_pressed = None;
            events.iter().for_each(|event| match event {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    match key_event.code {
                        KeyCode::Char('q') => {
                            self.exit = true;
                        }
                        KeyCode::Left => key_pressed = Some(KeyCode::Left),
                        KeyCode::Right => key_pressed = Some(KeyCode::Right),
                        KeyCode::Up => key_pressed = Some(KeyCode::Up),
                        KeyCode::Down => key_pressed = Some(KeyCode::Down),
                        _ => {}
                    }
                }
                _ => {}
            });
            if let Some(key_code) = key_pressed {
                self.update_direction(key_code);
            }
            self.move_snake();
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    const fn update_direction(&mut self, key_pressed: KeyCode) {
        match (key_pressed, &self.direction) {
            (KeyCode::Up, Direction::East) => self.direction = Direction::North,
            (KeyCode::Up, Direction::West) => self.direction = Direction::North,
            (KeyCode::Right, Direction::North) => self.direction = Direction::East,
            (KeyCode::Right, Direction::South) => self.direction = Direction::East,
            (KeyCode::Down, Direction::East) => self.direction = Direction::South,
            (KeyCode::Down, Direction::West) => self.direction = Direction::South,
            (KeyCode::Left, Direction::North) => self.direction = Direction::West,
            (KeyCode::Left, Direction::South) => self.direction = Direction::West,
            _ => {}
        }
    }

    fn move_head(&mut self) {
        self.head_pos.shift(&self.direction);
        assert!(!self.segments.is_empty());
        let last_segment = self.segments.back_mut().unwrap();
        if last_segment.direction != self.direction {
            self.segments.push_back(Segment {
                direction: self.direction, // here a copy happens
                length: 1,
            });
        } else {
            last_segment.length += 1;
        }
    }

    fn move_snake(&mut self) {
        self.move_head();
        if self.head_pos == self.food_pos {
            self.food_pos = Position {
                x: rand::random::<u16>() % 25,
                y: rand::random::<u16>() % 25,
            }
        } else {
            self.move_tail();
        }
    }

    fn move_tail(&mut self) {
        let first_segment = self.segments.front_mut().unwrap();
        first_segment.length -= 1;
        if first_segment.length == 0 {
            self.segments.pop_front();
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf[self.food_pos]
            // .set_symbol(&self.food_pos.character().to_string())
            .set_symbol("█")
            .set_fg(Color::Green);
        let mut start_pos = self.head_pos; // todo: this can become a iterator
        for segment in self.segments.iter().rev() {
            for _ in 0..segment.length {
                buf[start_pos].set_symbol("█").set_fg(Color::Yellow);
                start_pos.shift_opposite(&segment.direction);
            }
        }
        // TODO: handle food near snake color merging (bg and fg)
    }
}
