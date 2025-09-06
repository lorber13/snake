use std::{
    collections::VecDeque,
    io, thread,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame, buffer::Buffer, layout::Rect, style::Color, widgets::Widget,
};

use crate::app::snake::{Direction, Position, Segment, Snake};

mod snake;

pub struct App {
    exit: bool,
    snake: Snake,
}

struct EventReader;

impl EventReader {
    fn is_event_available() -> bool {
        event::poll(Duration::from_secs(0)).unwrap_or(false)
    }

    fn try_read_event() -> Option<Event> {
        if EventReader::is_event_available() {
            event::read().ok()
        } else {
            None
        }
    }
}

impl Iterator for EventReader {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        EventReader::try_read_event()
    }
}

const FRAME_DURATION: Duration = Duration::from_millis(100);

impl App {
    pub fn new(area: Rect) -> Self {
        App {
            exit: false,
            snake: Snake::new(
                area,
                VecDeque::from([
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
                Position { x: 17, y: 3 }, // todo: randomize
                Position { x: 14, y: 29 },
                Color::Yellow,
                Color::Green,
            ),
        }
    }
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        // the first frame will not update the position
        let mut timer = Instant::now();
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;

            App::wait_for_next_tick(&timer, FRAME_DURATION);
            timer = Instant::now();

            self.update_state(EventReader);
        }
        Ok(())
    }

    fn update_state(&mut self, events: EventReader) {
        let mut next_direction = self.snake.head_direction();
        events.for_each(|event| match event {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                App::handle_key_press(
                    key_event.code,
                    || self.exit(),
                    |key| {
                        App::update_direction(
                            &mut next_direction,
                            &Direction::from_key(key).unwrap(),
                        )
                    },
                );
            }
            Event::Resize(x, y) => todo!(),
            Event::FocusLost => todo!(),
            Event::FocusGained => todo!(),
            _ => {}
        });
        if self.snake.move_snake(&next_direction).is_err() {
            self.exit();
        }
    }

    const fn exit(&mut self) {
        self.exit = true;
    }

    fn wait_for_next_tick(prev_tick: &Instant, tick_duration: Duration) {
        if prev_tick.elapsed() < tick_duration {
            thread::sleep(tick_duration - prev_tick.elapsed());
        }
    }

    fn handle_key_press<F: FnMut(), G: FnMut(KeyCode)>(
        key: KeyCode,
        mut on_q_press: F,
        mut on_arrow_key_press: G,
    ) {
        match key {
            KeyCode::Char('q') => on_q_press(),
            KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down => on_arrow_key_press(key),
            _ => {}
        }
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    const fn update_direction(old_direction: &mut Direction, new_direction: &Direction) {
        match (&old_direction, new_direction) {
            (Direction::North, Direction::East) => *old_direction = Direction::East,
            (Direction::North, Direction::West) => *old_direction = Direction::West,
            (Direction::East, Direction::North) => *old_direction = Direction::North,
            (Direction::East, Direction::South) => *old_direction = Direction::South,
            (Direction::South, Direction::East) => *old_direction = Direction::East,
            (Direction::South, Direction::West) => *old_direction = Direction::West,
            (Direction::West, Direction::North) => *old_direction = Direction::North,
            (Direction::West, Direction::South) => *old_direction = Direction::South,
            _ => {}
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.snake.render(area, buf);
    }
}
