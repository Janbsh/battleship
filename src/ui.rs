use crate::app::App;
use crate::game::{BOARD_SIZE, Board, Cell, SHIP_LENGTHS};
use crate::state::AppState;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

const BATTLESHIP_TITLE: &str = r#"  ____        _   _   _           _     _
 | __ )  __ _| |_| |_| | ___  ___| |__ (_)_ __
 |  _ \ / _` | __| __| |/ _ \/ __| '_ \| | '_ \
 | |_) | (_| | |_| |_| |  __/\__ \ | | | | |_) |
 |____/ \__,_|\__|\__|_|\___||___/_| |_|_| .__/
                                         |_|    "#;

const SHIP_NAMES: [&str; 5] = ["Carrier", "Battleship", "Cruiser", "Submarine", "Destroyer"];

/// Render UI.
pub fn render(app: &mut App, f: &mut Frame) {
    let area = f.area();

    let target_width = 100;
    let target_height = 30;

    // Center container.
    let horizontal_chunks = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(target_width),
        Constraint::Fill(1),
    ])
    .split(area);

    let vertical_chunks = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(target_height),
        Constraint::Fill(1),
    ])
    .split(horizontal_chunks[1]);

    let container_area = vertical_chunks[1];

    // Outer border.
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .title(" BATTLESHIP ")
        .bold()
        .cyan();

    let main_area = outer_block.inner(container_area);
    f.render_widget(outer_block, container_area);

    // Render current state.
    match app.state {
        AppState::MainMenu | AppState::PlayMenu => render_menu(app, f, main_area),
        AppState::HostInput | AppState::JoinInput => render_input_screen(app, f, main_area),
        AppState::Connecting => render_message_screen(
            f,
            main_area,
            "SEARCHING FOR AN OPPONENT...",
            "We're waiting for someone to join the battle! (Esc to go back)",
        ),
        AppState::Placing | AppState::Game => render_game(app, f, main_area),
        AppState::GameOver => render_game_over(app, f, main_area),
        AppState::OpponentDisconnected => render_disconnected_screen(app, f, main_area),
    }
}

/// Render menu.
fn render_menu(app: &App, f: &mut Frame, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(6),
        Constraint::Length(2),
        Constraint::Length(4),
        Constraint::Fill(1),
    ])
    .split(area);

    // Title.
    let [_, title_area, _] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(48),
        Constraint::Fill(1),
    ])
    .areas(chunks[1]);
    f.render_widget(Paragraph::new(BATTLESHIP_TITLE).cyan(), title_area);

    // Menu options.
    let lines: Vec<Line> = app
        .menu_options
        .iter()
        .enumerate()
        .map(|(i, opt)| {
            let is_selected = Some(i) == app.menu_state.selected();
            let content = if is_selected {
                format!(">> {} <<", opt)
            } else {
                format!("   {}   ", opt)
            };

            let mut line = Line::from(content).alignment(Alignment::Center);
            if is_selected {
                line = line.style(Style::default().fg(Color::Yellow).bold());
            }
            line
        })
        .collect();

    f.render_widget(Paragraph::new(lines), chunks[3]);
}

/// Render message.
fn render_message_screen(f: &mut Frame, area: Rect, title: &str, subtitle: &str) {
    let chunks = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Fill(1),
    ])
    .split(area);

    f.render_widget(
        Paragraph::new(title)
            .alignment(Alignment::Center)
            .cyan()
            .bold(),
        chunks[1],
    );
    f.render_widget(
        Paragraph::new(subtitle)
            .alignment(Alignment::Center)
            .dark_gray(),
        chunks[2],
    );
}

/// Render disconnect screen.
fn render_disconnected_screen(app: &App, f: &mut Frame, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(3),
        Constraint::Length(2),
        Constraint::Length(3),
        Constraint::Fill(1),
    ])
    .split(area);

    f.render_widget(
        Paragraph::new("THE OTHER CAPTAIN HAS LEFT THE BATTLE.")
            .alignment(Alignment::Center)
            .red()
            .bold(),
        chunks[1],
    );

    let lines: Vec<Line> = app
        .menu_options
        .iter()
        .map(|opt| {
            Line::from(format!(">> {} <<", opt))
                .alignment(Alignment::Center)
                .yellow()
                .bold()
        })
        .collect();

    f.render_widget(Paragraph::new(lines), chunks[3]);
}

/// Render input screen.
fn render_input_screen(app: &App, f: &mut Frame, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Fill(1),
    ])
    .split(area);

    let (title, placeholder) = match app.state {
        AppState::HostInput => (
            "On which port should we listen for the opponent?",
            " Port Number ",
        ),
        AppState::JoinInput => ("What is the IP address and port of the host?", " IP:Port "),
        _ => ("", ""),
    };

    f.render_widget(
        Paragraph::new(title)
            .alignment(Alignment::Center)
            .cyan()
            .bold(),
        chunks[1],
    );

    // Input with cursor.
    let input_para = Paragraph::new(format!("{}_", app.input_buffer))
        .block(Block::bordered().title(placeholder))
        .alignment(Alignment::Center)
        .yellow();

    let [_, input_area, _] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(40),
        Constraint::Fill(1),
    ])
    .areas(chunks[2]);

    f.render_widget(input_para, input_area);
}

/// Render game.
fn render_game(app: &App, f: &mut Frame, area: Rect) {
    let h_layout = Layout::horizontal([
        Constraint::Min(64),
        Constraint::Length(24),
    ])
    .split(area);

    let left_v_layout = Layout::vertical([
        Constraint::Length(14),
        Constraint::Fill(1),
    ])
    .split(h_layout[0]);

    // Center boards.
    let boards_layout = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(36),
        Constraint::Length(0),
        Constraint::Length(36),
        Constraint::Fill(1),
    ])
    .split(left_v_layout[0]);

    let right_v_layout = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(3),
    ])
    .split(h_layout[1]);

    if let Some(ref state) = app.game_state {
        let mut my_cursor = None;
        let mut opp_cursor = None;
        let mut preview = None;

        if app.state == AppState::Placing {
            // Placement preview.
            my_cursor = Some(app.cursor_pos);
            if app.placing_ship_idx < SHIP_LENGTHS.len() {
                preview = Some((SHIP_LENGTHS[app.placing_ship_idx], app.placing_horizontal));
            }
        } else {
            // Aiming cursor.
            opp_cursor = Some(app.cursor_pos);
        }

        draw_grid(
            f,
            boards_layout[1],
            " Your Fleet ",
            &state.my_board,
            my_cursor,
            preview,
        );
        draw_grid(
            f,
            boards_layout[3],
            " Enemy Waters ",
            &state.opponent_board,
            opp_cursor,
            None,
        );

        // Status indicator.
        let info_color = if app.state == AppState::Placing {
            Color::Yellow
        } else if state.my_turn {
            Color::Green
        } else {
            Color::Red
        };

        let current_activity = if app.state == AppState::Placing {
            if app.placing_ship_idx < SHIP_NAMES.len() {
                format!(
                    "PLANNING: DEPLOYING THE {} (SIZE {})",
                    SHIP_NAMES[app.placing_ship_idx].to_uppercase(),
                    SHIP_LENGTHS[app.placing_ship_idx]
                )
            } else {
                "ALL SHIPS DEPLOYED! WAITING FOR THE OPPONENT.".to_string()
            }
        } else if state.my_turn {
            "YOUR TURN: SELECT A TARGET AND FIRE!".to_string()
        } else {
            "RELOADING: WAITING FOR THE ENEMY'S MOVE.".to_string()
        };

        let mut terminal_text = vec![Line::from(current_activity.fg(info_color).bold())];
        if app.state == AppState::Placing {
            terminal_text.push(Line::from(
                "WASD: Move | R: Rotate | Enter: Place".dark_gray(),
            ));
        } else {
            terminal_text.push(Line::from(
                "WASD: Aim | E: Open Radio | Enter: Fire!".dark_gray(),
            ));
        }

        f.render_widget(
            Paragraph::new(terminal_text).block(Block::bordered().title(" Control Log ")),
            left_v_layout[1],
        );
    }

    // Chat history.
    let chat_messages: Vec<Line> = app
        .chat_history
        .iter()
        .rev()
        .take(15)
        .rev()
        .map(|s| Line::from(s.clone()))
        .collect();

    f.render_widget(
        Paragraph::new(chat_messages).block(Block::bordered().title(" Radio History ")),
        right_v_layout[0],
    );

    // Chat input.
    let input_title = if app.chat_active {
        " Sending Message... "
    } else {
        " Press 'E' to Radio "
    };
    let input_style = if app.chat_active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().dark_gray()
    };
    let input_text = if app.chat_active {
        format!("{}_", app.input_buffer)
    } else {
        "".to_string()
    };

    f.render_widget(
        Paragraph::new(input_text).block(
            Block::bordered()
                .title(input_title)
                .border_style(input_style),
        ),
        right_v_layout[1],
    );
}

/// Render game over screen.
fn render_game_over(app: &App, f: &mut Frame, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(4),
        Constraint::Length(2),
        Constraint::Length(3),
        Constraint::Fill(1),
    ])
    .split(area);

    let (result_text, result_color) = if app.game_over_winner == Some(true) {
        (
            "VICTORY! THE ENEMY FLEET IS AT THE BOTTOM OF THE SEA!",
            Color::Green,
        )
    } else {
        ("DEFEAT! OUR SHIPS HAVE BEEN DESTROYED.", Color::Red)
    };

    f.render_widget(
        Paragraph::new(result_text)
            .alignment(Alignment::Center)
            .fg(result_color)
            .bold(),
        chunks[1],
    );

    // Options.
    let lines: Vec<Line> = app
        .menu_options
        .iter()
        .enumerate()
        .map(|(i, opt)| {
            let is_selected = Some(i) == app.menu_state.selected();
            let content = if is_selected {
                format!(">> {} <<", opt)
            } else {
                format!("   {}   ", opt)
            };

            let mut line = Line::from(content).alignment(Alignment::Center);
            if is_selected {
                line = line.style(Style::default().fg(Color::Yellow).bold());
            }
            line
        })
        .collect();

    f.render_widget(Paragraph::new(lines), chunks[3]);
}

/// Draw board grid.
fn draw_grid(
    f: &mut Frame,
    area: Rect,
    title: &str,
    board: &Board,
    cursor: Option<(usize, usize)>,
    preview: Option<(usize, bool)>,
) {
    let block = Block::bordered().title(title).bold();
    let inner = block.inner(area);
    f.render_widget(block, area);

    // X-axis labels.
    let mut rows = Vec::new();
    rows.push(
        Line::from("    A  B  C  D  E  F  G  H  I  J")
            .dark_gray()
            .bold(),
    );

    for y in 0..BOARD_SIZE {
        // Y-axis labels.
        let mut spans = vec![Span::styled(
            format!("{:2} ", y + 1),
            Style::default().dark_gray(),
        )];

        for x in 0..BOARD_SIZE {
            let is_cursor = cursor == Some((x, y));
            let mut is_preview = false;

            // Preview check.
            if let (Some((cx, cy)), Some((len, horizontal))) = (cursor, preview) {
                if horizontal {
                    if y == cy && x >= cx && x < cx + len {
                        is_preview = true;
                    }
                } else {
                    if x == cx && y >= cy && y < cy + len {
                        is_preview = true;
                    }
                }
            }

            // Cell styling.
            let (char, mut style) = match board.cells[y][x] {
                Cell::Ship => (" S ", Style::default().fg(Color::White).bold()),
                Cell::Miss => (" O ", Style::default().fg(Color::Blue)),
                Cell::Hit => (" X ", Style::default().fg(Color::Red).bold()),
                _ => (" . ", Style::default().fg(Color::DarkGray)),
            };

            if is_preview {
                style = style.fg(Color::Cyan);
                if is_cursor {
                    style = style
                        .add_modifier(Modifier::REVERSED)
                        .add_modifier(Modifier::BOLD);
                }
            } else if is_cursor {
                style = style.add_modifier(Modifier::REVERSED).fg(Color::Yellow);
            }

            spans.push(Span::styled(char, style));
        }
        rows.push(Line::from(spans));
    }

    f.render_widget(Paragraph::new(rows), inner);
}
