/// Represents the various screens and logic states of the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    /// Initial screen where the user selects an action.
    MainMenu,

    /// Sub-menu for choosing between hosting or joining.
    PlayMenu,

    /// Interface for entering hosting parameters like port.
    HostInput,

    /// Interface for entering the host's IP address.
    JoinInput,

    /// Transition state while waiting for a network handshake.
    Connecting,

    /// Phase where the player positions their ships on the grid.
    Placing,

    /// The active combat phase of the game.
    Game,

    /// Terminal state showing the winner and final scores.
    GameOver,

    /// Error state triggered when the network peer leaves.
    OpponentDisconnected,
}