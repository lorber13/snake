use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Position, Rect},
    widgets::Widget,
};

enum Direction {
    North,
    East,
    South,
    West,
}

enum KeyPressed {
    None,
    Up,
    Down,
    Left,
    Right,
}

pub struct App {
    head_pos: Position,
    exit: bool,
    direction: Direction,
    timer: Instant,
    key_pressed: KeyPressed,
}

impl Default for App {
    fn default() -> Self {
        App {
            head_pos: Position::ORIGIN,
            exit: false,
            direction: Direction::East,
            timer: Instant::now(),
            key_pressed: KeyPressed::None,
        }
    }
}

const FRAME_DURATION: Duration = Duration::from_millis(100);

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        // the first frame will not update the position
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            while self.timer.elapsed() < FRAME_DURATION {
                if event::poll(FRAME_DURATION - self.timer.elapsed())? {
                    self.handle_event(event::read()?);
                }
            }
            self.timer = Instant::now();
            self.update_direction();
            self.update_position();
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event);
            }
            _ => {}
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => {
                self.exit();
            }
            KeyCode::Left => self.key_pressed = KeyPressed::Left,
            KeyCode::Right => self.key_pressed = KeyPressed::Right,
            KeyCode::Up => self.key_pressed = KeyPressed::Up,
            KeyCode::Down => self.key_pressed = KeyPressed::Down,
            _ => {}
        }
    }

    fn update_direction(&mut self) {
        match (&self.key_pressed, &self.direction) {
            (KeyPressed::Up, Direction::East) => self.direction = Direction::North,
            (KeyPressed::Up, Direction::West) => self.direction = Direction::North,
            (KeyPressed::Right, Direction::North) => self.direction = Direction::East,
            (KeyPressed::Right, Direction::South) => self.direction = Direction::East,
            (KeyPressed::Down, Direction::East) => self.direction = Direction::South,
            (KeyPressed::Down, Direction::West) => self.direction = Direction::South,
            (KeyPressed::Left, Direction::North) => self.direction = Direction::West,
            (KeyPressed::Left, Direction::South) => self.direction = Direction::West,
            _ => {}
        }
    }

    fn update_position(&mut self) {
        match self.direction {
            Direction::North => self.move_up(),
            Direction::East => self.move_right(),
            Direction::South => self.move_down(),
            Direction::West => self.move_left(),
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn move_left(&mut self) {
        self.head_pos.x = self.head_pos.x.saturating_sub(1);
    }

    fn move_right(&mut self) {
        self.head_pos.x = self.head_pos.x.saturating_add(1);
    }

    fn move_up(&mut self) {
        self.head_pos.y = self.head_pos.y.saturating_sub(1);
    }

    fn move_down(&mut self) {
        self.head_pos.y = self.head_pos.y.saturating_add(1);
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.head_pos.x < buf.area.width && self.head_pos.y < buf.area.height {
            buf[self.head_pos].set_symbol("█");
        }
    }
}
