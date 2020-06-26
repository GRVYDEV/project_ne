use tetra::graphics::{self, Color};
use tetra::{Context, ContextBuilder, State};

struct GameState {}

impl State for GameState {}

fn main() -> tetra::Result {
    ContextBuilder::new("Neon", 600, 600)
        .quit_on_escape(true)
        .build()?
        .run(|_ctx| Ok(GameState {}))
}
