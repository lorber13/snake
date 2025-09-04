use std::{
    collections::VecDeque,
    io, thread,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
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

impl Direction {
    const fn from_key(key: KeyCode) -> Option<Self> {
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
    food_pos: Position,
}

impl Default for App {
    fn default() -> Self {
        App {
            head_pos: Position { x: 14, y: 29 },
            exit: false,
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
        while !self.exit {
            // DRAW
            terminal.draw(|frame| self.draw(frame))?;

            App::wait_for_next_tick(&timer, FRAME_DURATION);
            timer = Instant::now();

            assert!(!self.segments.is_empty());
            let mut next_direction = self.segments.back().unwrap().direction;
            while let Some(event) = App::read_event()? {
                App::handle_event(
                    event,
                    || self.exit(),
                    |key_code| {
                        App::update_direction(
                            &mut next_direction,
                            &Direction::from_key(key_code).unwrap(),
                        );
                    },
                );
            }
            self.move_snake(&next_direction);
        }
        Ok(())
    }

    const fn exit(&mut self) {
        self.exit = true;
    }

    fn wait_for_next_tick(prev_tick: &Instant, tick_duration: Duration) {
        if prev_tick.elapsed() < tick_duration {
            thread::sleep(tick_duration - prev_tick.elapsed());
        }
    }

    fn handle_event<F: FnMut(), G: FnMut(KeyCode)>(
        event: Event,
        on_q_press: F,
        on_arrow_key_press: G,
    ) {
        match event {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                App::handle_key_event(key_event, on_q_press, on_arrow_key_press);
            }
            _ => {}
        }
    }

    fn handle_key_event<F: FnMut(), G: FnMut(KeyCode)>(
        key_event: KeyEvent,
        mut on_q_press: F,
        mut on_arrow_key_press: G,
    ) {
        match key_event.code {
            KeyCode::Char('q') => on_q_press(),
            KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down => {
                on_arrow_key_press(key_event.code)
            }
            _ => {}
        }
    }

    fn read_event() -> io::Result<Option<Event>> {
        if event::poll(Duration::from_secs(0))? {
            Ok(Some(event::read()?))
        } else {
            Ok(None)
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

    fn move_head(&mut self, direction: &Direction) {
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
    }

    fn move_snake(&mut self, direction: &Direction) {
        self.move_head(direction);
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
                start_pos.shift(&segment.direction.opposite());
            }
        }
        // TODO: handle food near snake color merging (bg and fg)
    }
}
