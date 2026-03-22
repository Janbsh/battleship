use crate::game::GameState;
use crate::network::{Message, Peer};
use crate::state::AppState;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::widgets::ListState;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{self, Write};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

/// Holds the primary application state and handles the main event loop.
pub struct App {
    /// current active screen or logic state.
    pub state: AppState,

    /// flag to signal the application should terminate.
    pub exit: bool,

    /// labels for the currently active menu.
    pub menu_options: Vec<&'static str>,

    /// stateful tracking for the list widget selection.
    pub menu_state: ListState,

    /// buffer for text input in chat or connection screens.
    pub input_buffer: String,

    /// the port used when hosting a game.
    pub host_port: String,

    /// the remote address used when joining a game.
    pub join_addr: String,

    /// local simulation of the battleship boards and turns.
    pub game_state: Option<GameState>,

    /// channel to send messages to the background network thread.
    pub peer_sender: Option<Sender<Message>>,

    /// channel to receive messages from the background network thread.
    pub msg_receiver: Option<Receiver<Message>>,

    /// current (x, y) coordinate of the grid cursor.
    pub cursor_pos: (usize, usize),

    /// whether the user is currently typing in the radio buffer.
    pub chat_active: bool,

    /// history of sent and received chat messages.
    pub chat_history: Vec<String>,

    /// index of the ship currently being placed.
    pub placing_ship_idx: usize,

    /// toggle for ship placement orientation.
    pub placing_horizontal: bool,

    /// tracks if the local player won or lost.
    pub game_over_winner: Option<bool>,
}

impl Default for App {
    /// Initializes the app with default main menu settings.
    fn default() -> Self {
        let mut menu_state = ListState::default();
        menu_state.select(Some(0));
        Self {
            state: AppState::MainMenu,
            exit: false,
            menu_options: vec!["Play", "Exit"],
            menu_state,
            input_buffer: String::new(),
            host_port: String::new(),
            join_addr: String::new(),
            game_state: None,
            peer_sender: None,
            msg_receiver: None,
            cursor_pos: (0, 0),
            chat_active: false,
            chat_history: Vec::new(),
            placing_ship_idx: 0,
            placing_horizontal: true,
            game_over_winner: None,
        }
    }
}

impl App {
    /// Enters the main application loop, processing input and drawing frames.
    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
        while !self.exit {
            // check for incoming network messages before rendering.
            self.update();

            // draw the UI based on the current app state.
            terminal.draw(|f| crate::ui::render(self, f))?;

            // poll for keyboard events at a steady interval.
            if event::poll(Duration::from_millis(50))? {
                // drain all pending events to keep input responsive.
                while event::poll(Duration::from_millis(0))? {
                    let ev = event::read()?;
                    match ev {
                        Event::Key(key) => {
                            if key.kind == KeyEventKind::Press {
                                // route input handling based on the current screen.
                                match self.state {
                                    AppState::MainMenu | AppState::PlayMenu => self.handle_menu_input(key.code),
                                    AppState::HostInput | AppState::JoinInput => self.handle_text_input(key.code),
                                    AppState::Game | AppState::Placing => self.handle_game_input(key.code),
                                    AppState::GameOver => self.handle_game_over_input(key.code),
                                    AppState::OpponentDisconnected => self.handle_disconnected_input(key.code),
                                    AppState::Connecting => {
                                        if key.code == KeyCode::Esc {
                                            self.switch_to_main_menu();
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    /// Pulls all pending messages from the network receiver channel.
    fn update(&mut self) {
        let mut messages = Vec::new();
        if let Some(ref rx) = self.msg_receiver {
            // collect all messages available in the channel without blocking.
            while let Ok(msg) = rx.try_recv() {
                messages.push(msg);
            }
        }
        for msg in messages {
            self.handle_network_message(msg);
        }
    }

    /// Resolves logic for specific message types received from the peer.
    fn handle_network_message(&mut self, msg: Message) {
        match msg {
            Message::Ready => {
                // transition to ship placement once both players are connected.
                if self.state == AppState::Connecting {
                    self.state = AppState::Placing;
                }
            }
            Message::Attack { x, y } => {
                if let Some(ref mut state) = self.game_state {
                    if let Ok(result_cell) = state.handle_opponent_move(x, y) {
                        let is_hit = result_cell == crate::game::Cell::Hit;
                        let is_all_sunk = state.my_board.is_all_sunk();

                        // notify the opponent of the result of their shot.
                        self.send_network_message(Message::Result {
                            x,
                            y,
                            hit: is_hit,
                            sunk: is_all_sunk,
                        });

                        // end the game if all local ships are destroyed.
                        if is_all_sunk {
                            self.state = AppState::GameOver;
                            self.game_over_winner = Some(false);
                            self.menu_options = vec!["Rematch", "Main Menu"];
                            self.menu_state.select(Some(0));
                        }
                    }
                }
            }
            Message::Result { x, y, hit, sunk } => {
                // record the result of the local player's shot on the tracking board.
                if let Some(ref mut state) = self.game_state {
                    state.opponent_board.cells[y][x] = if hit {
                        crate::game::Cell::Hit
                    } else {
                        crate::game::Cell::Miss
                    };

                    // end the game if the opponent reports all ships sunk.
                    if sunk {
                        self.state = AppState::GameOver;
                        self.game_over_winner = Some(true);
                        self.menu_options = vec!["Rematch", "Main Menu"];
                        self.menu_state.select(Some(0));
                    }
                }
            }
            Message::ChatMessage(text) => {
                // append incoming chat messages to the local history.
                self.chat_history.push(format!("Opponent: {}", text));
            }
            Message::Disconnected => {
                // handle unexpected peer disconnection unless already at the menu.
                if self.state != AppState::MainMenu {
                    self.state = AppState::OpponentDisconnected;
                    self.menu_options = vec!["OK"];
                    self.menu_state.select(Some(0));
                }
            }
        }
    }

    /// Forwards a message to the background thread for network transmission.
    fn send_network_message(&self, msg: Message) {
        if let Some(ref tx) = self.peer_sender {
            let _ = tx.send(msg);
        }
    }

    /// Handles directional and selection input for menu screens.
    fn handle_menu_input(&mut self, code: KeyCode) {
        match code {
            KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => self.menu_prev(),
            KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => self.menu_next(),
            KeyCode::Enter => self.handle_menu_select(),
            KeyCode::Esc => {
                if matches!(self.state, AppState::PlayMenu) {
                    self.switch_to_main_menu();
                }
            }
            KeyCode::Char('q') => {
                self.send_network_message(Message::Disconnected);
                self.exit = true;
            }
            _ => {}
        }
    }

    /// Triggers actions based on the currently selected menu item.
    fn handle_menu_select(&mut self) {
        let selected = self.menu_state.selected().unwrap_or(0);
        match self.state {
            AppState::MainMenu => match selected {
                0 => self.switch_to_play_menu(),
                1 => {
                    self.send_network_message(Message::Disconnected);
                    self.exit = true;
                }
                _ => {}
            },
            AppState::PlayMenu => match selected {
                0 => {
                    self.state = AppState::HostInput;
                    self.input_buffer.clear();
                }
                1 => {
                    self.state = AppState::JoinInput;
                    self.input_buffer.clear();
                }
                2 => self.switch_to_main_menu(),
                _ => {}
            },
            _ => {}
        }
    }

    /// Processes keyboard input for IP and port entry.
    fn handle_text_input(&mut self, code: KeyCode) {
        match code {
            KeyCode::Enter => match self.state {
                AppState::HostInput => {
                    self.host_port = self.input_buffer.clone();
                    self.start_connection(true);
                }
                AppState::JoinInput => {
                    self.join_addr = self.input_buffer.clone();
                    self.start_connection(false);
                }
                _ => {}
            },
            KeyCode::Char(c) => self.input_buffer.push(c),
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Esc => self.switch_to_play_menu(),
            _ => {}
        }
    }

    /// Spawns background threads to handle peer-to-peer communication.
    fn start_connection(&mut self, host: bool) {
        let addr = if host {
            format!("0.0.0.0:{}", self.host_port)
        } else {
            self.join_addr.clone()
        };

        self.state = AppState::Connecting;
        let (tx_to_app, rx_from_peer) = mpsc::channel();
        let (tx_to_peer, rx_from_app) = mpsc::channel();

        self.msg_receiver = Some(rx_from_peer);
        self.peer_sender = Some(tx_to_peer);

        let tx_thread = tx_to_app.clone();

        thread::spawn(move || {
            // attempt to establish the tcp connection.
            match Peer::new(&addr, host) {
                Ok(peer) => {
                    let peer_receive = peer.stream.try_clone().expect("Failed to clone stream");
                    let mut peer_send = peer.stream;

                    // signal the main thread that the peer is connected.
                    let _ = tx_thread.send(Message::Ready);

                    // spawn a dedicated thread for continuous deserialization.
                    let rx_inner = tx_thread.clone();
                    thread::spawn(move || {
                        loop {
                            match bincode::deserialize_from::<_, Message>(&peer_receive) {
                                Ok(msg) => {
                                    if rx_inner.send(msg).is_err() {
                                        break;
                                    }
                                }
                                Err(_) => {
                                    let _ = rx_inner.send(Message::Disconnected);
                                    break;
                                }
                            }
                        }
                    });

                    // remain in the sender loop to push outgoing messages.
                    while let Ok(msg) = rx_from_app.recv() {
                        if bincode::serialize_into(&peer_send, &msg).is_err() {
                            break;
                        }
                        let _ = peer_send.flush();
                    }
                }
                Err(_) => {
                    let _ = tx_thread.send(Message::Disconnected);
                }
            }
        });

        // initialize the game state for the new session.
        self.game_state = Some(GameState::new(host));
    }

    /// Handles interaction during ship placement and active combat.
    fn handle_game_input(&mut self, code: KeyCode) {
        // divert input to the text buffer if the chat interface is active.
        if self.chat_active {
            match code {
                KeyCode::Enter => {
                    if !self.input_buffer.is_empty() {
                        let msg = self.input_buffer.clone();
                        self.send_network_message(Message::ChatMessage(msg.clone()));
                        self.chat_history.push(format!("You: {}", msg));
                        self.input_buffer.clear();
                    }
                    self.chat_active = false;
                }
                KeyCode::Esc => {
                    self.chat_active = false;
                    self.input_buffer.clear();
                }
                KeyCode::Char(c) => self.input_buffer.push(c),
                KeyCode::Backspace => {
                    self.input_buffer.pop();
                }
                _ => {}
            }
            return;
        }

        match code {
            // handle grid navigation with wasd or arrow keys.
            KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => {
                if self.cursor_pos.1 > 0 {
                    self.cursor_pos.1 -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
                if self.cursor_pos.1 < 9 {
                    self.cursor_pos.1 += 1;
                }
            }
            KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') => {
                if self.cursor_pos.0 > 0 {
                    self.cursor_pos.0 -= 1;
                }
            }
            KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => {
                if self.cursor_pos.0 < 9 {
                    self.cursor_pos.0 += 1;
                }
            }
            // rotate ship placement orientation.
            KeyCode::Char('r') | KeyCode::Char('R') | KeyCode::Char('x') | KeyCode::Char('X') => {
                self.placing_horizontal = !self.placing_horizontal;
            }
            // toggle the chat input overlay.
            KeyCode::Char('e') | KeyCode::Char('E') | KeyCode::Char('c') | KeyCode::Char('C') => {
                self.chat_active = true;
                self.input_buffer.clear();
            }
            // process primary actions like placing a ship or firing a shot.
            | KeyCode::Char('z') | KeyCode::Char('Z') | KeyCode::Enter => {
                let mut attack_to_send = None;
                if self.state == AppState::Game
                    && let Some(ref mut state) = self.game_state
                    && state.my_turn
                {
                    let (x, y) = self.cursor_pos;
                    if state.handle_my_move(x, y).is_ok() {
                        state.my_turn = false;
                        attack_to_send = Some((x, y));
                    }
                } else if self.state == AppState::Placing
                    && let Some(ref mut state) = self.game_state
                {
                    let (x, y) = self.cursor_pos;
                    let length = crate::game::SHIP_LENGTHS[self.placing_ship_idx];
                    if state
                        .my_board
                        .place_ship(x, y, length, self.placing_horizontal)
                        .is_ok()
                    {
                        self.placing_ship_idx += 1;
                        // notify peer that placement is done once all ships are set.
                        if self.placing_ship_idx >= crate::game::SHIP_LENGTHS.len() {
                            self.state = AppState::Game;
                            self.send_network_message(Message::Ready);
                        }
                    }
                }

                if let Some((x, y)) = attack_to_send {
                    self.send_network_message(Message::Attack { x, y });
                }
            }
            KeyCode::Esc => self.switch_to_main_menu(),
            _ => {}
        }
    }

    /// Handles menu navigation after a game has concluded.
    fn handle_game_over_input(&mut self, code: KeyCode) {
        match code {
            KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => self.menu_prev(),
            KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => self.menu_next(),
            KeyCode::Enter => match self.menu_state.selected() {
                Some(0) => self.reset_game_for_rematch(),
                Some(1) => self.switch_to_main_menu(),
                _ => {}
            },
            _ => {}
        }
    }

    /// Clears the disconnection notification and returns to the menu.
    fn handle_disconnected_input(&mut self, code: KeyCode) {
        match code {
            KeyCode::Enter | KeyCode::Esc | KeyCode::Char(' ') => {
                self.switch_to_main_menu();
            }
            _ => {}
        }
    }

    /// Prepares the local state for a new round with the same peer.
    fn reset_game_for_rematch(&mut self) {
        let is_host = !self.host_port.is_empty();
        self.game_state = Some(GameState::new(is_host));
        self.placing_ship_idx = 0;
        self.cursor_pos = (0, 0);
        self.state = AppState::Placing;
        self.game_over_winner = None;
        self.send_network_message(Message::Ready);
    }

    /// Resets the application to its initial state and drops networking.
    fn switch_to_main_menu(&mut self) {
        self.send_network_message(Message::Disconnected);
        self.state = AppState::MainMenu;
        self.menu_options = vec!["Play", "Exit"];
        self.menu_state.select(Some(0));
        self.game_state = None;
        self.peer_sender = None;
        self.msg_receiver = None;
        self.chat_history.clear();
        self.chat_active = false;
        self.game_over_winner = None;
    }

    /// Updates state and options for the hosting/joining sub-menu.
    fn switch_to_play_menu(&mut self) {
        self.state = AppState::PlayMenu;
        self.menu_options = vec!["Host Game", "Join Game", "Back"];
        self.menu_state.select(Some(0));
    }

    /// Selects the next item in the current menu with wrapping.
    fn menu_next(&mut self) {
        let i = match self.menu_state.selected() {
            Some(i) => {
                if i >= self.menu_options.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.menu_state.select(Some(i));
    }

    /// Selects the previous item in the current menu with wrapping.
    fn menu_prev(&mut self) {
        let i = match self.menu_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.menu_options.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.menu_state.select(Some(i));
    }
}