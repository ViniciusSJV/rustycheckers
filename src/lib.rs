mod board;
mod game;

#[macro_use]
extern crate lazy_static;
extern crate mut_static;

use board::{Coordinate, GamePiece, Move, PieceColor};
use game::GameEngine;
use mut_static::MutStatic;

lazy_static! {
    pub static ref GAME_ENGINE: MutStatic<GameEngine> = { MutStatic::from(GameEngine::new()) };
}

#[no_mangle]
pub extern "C" fn get_piece(x: i32, y: i32) -> i32 {
    let piece = GAME_ENGINE
        .read()
        .unwrap()
        .get_piece(Coordinate(x as usize, y as usize));

    match piece {
        Ok(Some(piece)) => piece.into(),
        Ok(None) => -1,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn get_current_turn() -> i32 {
    let engine = GAME_ENGINE.read().unwrap();

    GamePiece::new(engine.current_turn()).into()
}

#[no_mangle]
pub extern "C" fn move_piece(from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> i32 {
    let mut engine = GAME_ENGINE.write().unwrap();

    let mv = Move::new(
        (from_x as usize, from_y as usize),
        (to_x as usize, to_y as usize),
    );

    match engine.move_piece(&mv) {
        Ok(m) => {
            unsafe {
                notify_piecemoved(from_x, from_y, to_x, to_y);
            }

            if m.crowned {
                unsafe {
                    notify_piececrowned(to_x, to_y);
                }
            }

            1
        }
        Err(_) => 0,
    }
}

extern "C" {
    fn notify_piecemoved(fromX: i32, fromY: i32, toX: i32, toY: i32);
    fn notify_piececrowned(toX: i32, toY: i32);
}

const PIECEFLAG_BLACK: u8 = 1;
const PIECEFLAG_WHITE: u8 = 2;
const PIECEFLAG_CROWN: u8 = 4;

impl Into<i32> for GamePiece {
    fn into(self) -> i32 {
        let mut val: u8 = match self.color {
            PieceColor::Black => PIECEFLAG_BLACK,
            PieceColor::White => PIECEFLAG_WHITE,
        };

        if self.crowned {
            val += PIECEFLAG_CROWN;
        }

        val as i32
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn move_piece_test() {
        assert!(move_piece(0, 5, 1, 4) != 0)
    }
}