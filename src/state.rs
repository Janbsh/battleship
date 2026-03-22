/// App states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    // Main menu.
    MainMenu,

    // Play menu.
    PlayMenu,

    // Host input.
    HostInput,

    // Join input.
    JoinInput,

    // Connecting.
    Connecting,

    // Ship placement.
    Placing,

    // Battle.
    Game,

    // Game over.
    GameOver,

    // Disconnected.
    OpponentDisconnected,
}
