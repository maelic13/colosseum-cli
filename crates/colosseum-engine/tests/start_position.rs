//! A start position that does not parse is refused, never replaced by the
//! standard one: the engines would be sent the given FEN while the harness
//! judged a game from the standard start.

use std::time::Duration;

use colosseum_core::{AdjudicationConfig, EngineId, GameId, TimeControl};
use colosseum_engine::runner::{EngineGameSpec, GameFault, GameSpec, run_game};
use colosseum_uci::SpawnOptions;

/// An executable that does not exist: a game that tried to start its engines
/// would fail with `engine-spawn`, so the refusal is shown to come first.
fn absent_engine(name: &str) -> EngineGameSpec {
    EngineGameSpec {
        id: EngineId::from_uuid(uuid::Uuid::new_v4()),
        name: name.to_string(),
        spawn: SpawnOptions::new("colosseum-no-such-engine"),
        options: Vec::new(),
        allocated_cpus: colosseum_application::CpuAllocation::Unrestricted,
    }
}

fn game(round: u32, fen: &str) -> GameSpec {
    GameSpec {
        game_id: GameId::from_uuid(uuid::Uuid::new_v4()),
        event: "Start Position".into(),
        site: "Local".into(),
        date: "2026.11.18".into(),
        round,
        white: absent_engine("White"),
        black: absent_engine("Black"),
        start_fen: Some(fen.to_string()),
        opening_moves: vec!["e2e4".into()],
        white_time_control: TimeControl::PerMove { ms: 20 },
        black_time_control: TimeControl::PerMove { ms: 20 },
        time_control_label: "movetime/20ms".into(),
        adjudication: AdjudicationConfig::default(),
        ponder: false,
        white_time_margin: Duration::from_secs(2),
        black_time_margin: Duration::from_secs(2),
        handshake_timeout: Duration::from_secs(5),
        identity: None,
        slot: None,
    }
}

#[tokio::test]
async fn an_unparsable_start_position_is_an_error_naming_the_game_and_the_fen() {
    let invalid = [
        // Not a FEN at all.
        "not a position",
        // A FEN whose position is not legal: White has no king.
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQ1BNR w kq - 0 1",
    ];
    for (round, fen) in (3..).zip(invalid) {
        let game = game(round, fen);
        let live = colosseum_engine::LiveGameState::new_handle(
            game.game_id,
            game.round,
            (game.white.id, game.white.name.clone()),
            (game.black.id, game.black.name.clone()),
            game.start_fen.clone(),
            game.white_time_control,
        );
        let report = run_game(game, live).await;

        assert!(!report.scorable, "{fen}: {report:?}");
        assert!(report.uci_moves.is_empty(), "{fen}: {:?}", report.uci_moves);
        let Some(GameFault::Infrastructure { operation, message }) = &report.fault else {
            panic!("{fen}: not an infrastructure fault: {:?}", report.fault);
        };
        assert_eq!(operation, "start-position", "{message}");
        assert!(
            message.starts_with(&format!("game {round}: the start position `{fen}`")),
            "{message}"
        );
        assert_eq!(report.error.as_deref(), Some(message.as_str()));
    }
}
