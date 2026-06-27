use std::sync::LazyLock;

use crate::{
    board::Board,
    game::{Direction, Game},
};
use color_eyre::{Result, eyre::WrapErr};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Widget},
};

#[derive(Debug, Default)]
pub struct App {
    game: Game,
    exit: bool,
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        self.game.debug_log("/tmp/2048_debug.log".to_string());
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => self
                .handle_key_event(key_event)
                .wrap_err_with(|| format!("handling key event failed:\n{key_event:#?}")),
            _ => Ok(()),
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),

            KeyCode::Up => self.make_move(Direction::Up)?,
            KeyCode::Left => self.make_move(Direction::Left)?,
            KeyCode::Down => self.make_move(Direction::Down)?,
            KeyCode::Right => self.make_move(Direction::Right)?,

            KeyCode::Char('w') => self.make_move(Direction::Up)?,
            KeyCode::Char('a') => self.make_move(Direction::Left)?,
            KeyCode::Char('s') => self.make_move(Direction::Down)?,
            KeyCode::Char('d') => self.make_move(Direction::Right)?,
            _ => {}
        }
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn make_move(&mut self, direction: Direction) -> Result<()> {
        let _success = self.game.make_move(direction);
        // Making a move that doesn't change the board is not an error, so we don't need to handle the success value.
        Ok(())
    }
}

static EXPONENTS_TO_COLORED_NUMBERS: LazyLock<[Span<'static>; 16]> = LazyLock::new(|| {
    [
        Span::raw("     "),
        Span::raw("    2").fg(Color::Rgb(255, 85, 85)),
        Span::raw("    4").fg(Color::Rgb(255, 122, 85)),
        Span::raw("    8").fg(Color::Rgb(255, 162, 85)),
        Span::raw("   16").fg(Color::Rgb(255, 201, 85)),
        Span::raw("   32").fg(Color::Rgb(255, 242, 85)),
        Span::raw("   64").fg(Color::Rgb(216, 255, 85)),
        Span::raw("  128").fg(Color::Rgb(168, 255, 85)),
        Span::raw("  256").fg(Color::Rgb(112, 255, 85)),
        Span::raw("  512").fg(Color::Rgb(85, 255, 122)),
        Span::raw(" 1024").fg(Color::Rgb(85, 255, 162)),
        Span::raw(" 2048").fg(Color::Rgb(85, 255, 201)),
        Span::raw(" 4096").fg(Color::Rgb(85, 242, 255)),
        Span::raw(" 8192").fg(Color::Rgb(85, 201, 255)),
        Span::raw("16384").fg(Color::Rgb(85, 159, 255)),
        Span::raw("32768").fg(Color::Rgb(122, 85, 255)),
    ]
});

static TOP_SEPERATOR: LazyLock<Line> =
    LazyLock::new(|| Line::from("┏━━━━━━━┳━━━━━━━┳━━━━━━━┳━━━━━━━┓".to_string()));

static ROW_SEPERATOR: LazyLock<Line> =
    LazyLock::new(|| Line::from("┣━━━━━━━╋━━━━━━━╋━━━━━━━╋━━━━━━━┫".to_string()));

static BOTTOM_SEPERATOR: LazyLock<Line> =
    LazyLock::new(|| Line::from("┗━━━━━━━┻━━━━━━━┻━━━━━━━┻━━━━━━━┛".to_string()));

static EMPTY_ROW: LazyLock<Line> =
    LazyLock::new(|| Line::from("┃       ┃       ┃       ┃       ┃".to_string()));

/// renders the given board as a Text object
fn render_board(board: &Board) -> Text<'_> {
    let mut lines = Vec::with_capacity(9);

    // Top border
    lines.push(TOP_SEPERATOR.clone());

    for row in 0..4 {
        // let cells = board
        //     .get_row_tiles_exponents(row)
        //     .iter()
        //     .map(|&exponent| format_exponent_to_number(exponent))
        //     .collect::<Vec<_>>()
        //     .join(" ┃ ");
        lines.push(EMPTY_ROW.clone());

        let mut line = Line::default();
        line.push_span("┃ ");
        for &exponent in board.get_row_tiles_exponents(row).iter() {
            let span = EXPONENTS_TO_COLORED_NUMBERS[exponent as usize].clone();
            line.push_span(span);
            line.push_span(" ┃ ");
        }
        line.push_span(" ┃");
        lines.push(line);

        lines.push(EMPTY_ROW.clone());
        if !(row == 3) {
            lines.push(ROW_SEPERATOR.clone());
        }
    }

    lines.push(BOTTOM_SEPERATOR.clone());
    Text::from(lines)
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Play 2048 ".bold());
        let instructions = Line::from(vec![
            " Move ".into(),
            "WASD".blue().bold(),
            "/".into(),
            "←↑↓→".blue().bold(),
            " Quit ".into(),
            "Q ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        // Shrink the area
        let x_shrink = 8;
        let y_shrink = 3;
        block.render(
            Rect::new(
                x_shrink,
                y_shrink,
                area.width - 2 * x_shrink,
                area.height - 2 * y_shrink,
            ),
            buf,
        );

        let board_text = render_board(self.game.board());

        // Center only the board content
        let centered_area = area.centered(
            Constraint::Length(35), // board width
            Constraint::Length(17), // board height
        );

        // Render the board to a centered area.
        Paragraph::new(board_text).render(centered_area, buf);
    }
}
