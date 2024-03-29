use super::board::{Coordinate, GamePiece, Move, PieceColor};

pub struct GameEngine {
    board: [[Option<GamePiece>; 8]; 8],
    current_turn: PieceColor,
    move_count: u32,
}

pub struct MoveResult {
    pub mv: Move,
    pub crowned: bool,
}

impl GameEngine {
    pub fn new() -> GameEngine {
        let mut engine = GameEngine {
            board: [[None; 8]; 8],
            current_turn: PieceColor::Black,
            move_count: 0,
        };

        engine.initialize_pieces();
        engine
    }

    pub fn initialize_pieces(&mut self) {
        [1, 3, 5, 7, 0, 2, 4, 6, 1, 3, 5, 7]
            .iter()
            .zip([0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2].iter())
            .map(|(a, b)| (*a as usize, *b as usize))
            .for_each(|(x, y)| {
                self.board[x][y] = Some(GamePiece::new(PieceColor::White));
            });

        [0, 2, 4, 6, 1, 3, 5, 7, 0, 2, 4, 6]
            .iter()
            .zip([5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7].iter())
            .map(|(a, b)| (*a as usize, *b as usize))
            .for_each(|(x, y)| {
                self.board[x][y] = Some(GamePiece::new(PieceColor::Black));
            });
    }

    pub fn move_piece(&mut self, mv: &Move) -> Result<MoveResult, ()> {
        let legal_moves = self.legal_moves();

        if !legal_moves.contains(&mv) {
            return Err(());
        }

        let Coordinate(from_x, from_y) = mv.from;
        let Coordinate(to_x, to_y) = mv.to;

        let piece = self.board[from_x][from_y].unwrap();

        let midpiece_coordinate = self.midpiece_coordinate(from_x, from_y, to_x, to_y);

        if let Some(Coordinate(x, y)) = midpiece_coordinate {
            self.board[x][y] = None; // remove the jumped piece
        }

        self.board[from_x][from_y] = None;
        self.board[to_x][to_y] = Some(piece);

        let crowned = if self.should_crown(piece, mv.to) {
            self.crown_piece(mv.to);
            true
        } else {
            false
        };

        self.advance_turn();

        Ok(MoveResult {
            crowned,
            mv: mv.clone(),
        })
    }

    pub fn get_piece(&self, coord: Coordinate) -> Result<Option<GamePiece>, ()> {
        let Coordinate(x, y) = coord;
        if x <= 7 && y <= 7 {
            Ok(self.board[x][y])
        } else {
            Err(())
        }
    }

    pub fn current_turn(&self) -> PieceColor {
        self.current_turn
    }

    fn advance_turn(&mut self) {
        if self.current_turn == PieceColor::Black {
            self.current_turn = PieceColor::White
        } else {
            self.current_turn = PieceColor::Black
        }
        self.move_count += 1;
    }

    // Black pieces in row 0 or White pieces in row 7 are crowned
    fn should_crown(&self, piece: GamePiece, coord: Coordinate) -> bool {
        let Coordinate(_x, y) = coord;

        (y == 0 && piece.color == PieceColor::Black) || (y == 7 && piece.color == PieceColor::White)
    }

    fn crown_piece(&mut self, coord: Coordinate) -> bool {
        let Coordinate(x, y) = coord;
        if let Some(piece) = self.board[x][y] {
            self.board[x][y] = Some(GamePiece::crowned(piece));
            true
        } else {
            false
        }
    }

    pub fn is_crowned(&self, coord: Coordinate) -> bool {
        let Coordinate(x, y) = coord;
        match self.board[x][y] {
            Some(piece) => piece.crowned,
            None => false,
        }
    }

    pub fn move_count(&self) -> u32 {
        self.move_count
    }

    fn legal_moves(&self) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::new();

        for col in 0..8 {
            for row in 0..8 {
                if let Some(piece) = self.board[row][col] {
                    if piece.color == self.current_turn {
                        let loc = Coordinate(row, col);
                        let mut valid_moves = self.valid_moves_from(loc);

                        moves.append(&mut valid_moves);
                    }
                }
            }
        }

        moves
    }

    fn valid_moves_from(&self, location: Coordinate) -> Vec<Move> {
        let Coordinate(x, y) = location;

        if let Some(piece) = self.board[x][y] {
            let mut jumps = location
                .jump_targets_from()
                .filter(|to| self.valid_jump(&piece, &location, &to))
                .map(|ref to| Move {
                    from: location.clone(),
                    to: to.clone(),
                })
                .collect::<Vec<Move>>();

            let mut moves = location
                .move_targets_from()
                .filter(|to| self.valid_move(&piece, &location, &to))
                .map(|ref to| Move {
                    from: location.clone(),
                    to: to.clone(),
                })
                .collect::<Vec<Move>>();

            jumps.append(&mut moves);
            jumps
        } else {
            Vec::new()
        }
    }

    fn midpiece_coordinate(&self, x: usize, y: usize, tx: usize, ty: usize) -> Option<Coordinate> {
        if tx == x + 2 && ty == y + 2 {
            Some(Coordinate(x + 1, y + 1))
        } else if x >= 2 && y >= 2 && tx == x - 2 && ty == y - 2 {
            Some(Coordinate(x - 1, y - 1))
        } else if x >= 2 && tx == x - 2 && ty == y + 2 {
            Some(Coordinate(x - 1, y + 1))
        } else if y >= 2 && tx == x + 2 && ty == y - 2 {
            Some(Coordinate(x + 1, y - 1))
        } else {
            None
        }
    }

    fn midpiece(&self, x: usize, y: usize, tx: usize, ty: usize) -> Option<GamePiece> {
        match self.midpiece_coordinate(x, y, tx, ty) {
            Some(Coordinate(x, y)) => self.board[x][y],
            None => None,
        }
    }

    fn valid_jump(&self, p: &GamePiece, from: &Coordinate, to: &Coordinate) -> bool {
        if !to.on_board() || !from.on_board() {
            false
        } else {
            let Coordinate(x, y) = *from;
            let Coordinate(tx, ty) = *to;

            let midpiece = self.midpiece(x, y, tx, ty);
            match midpiece {
                Some(mp) if mp.color != p.color => true,
                _ => false,
            }
        }
    }

    fn valid_move(&self, p: &GamePiece, from: &Coordinate, to: &Coordinate) -> bool {
        if !to.on_board() || !from.on_board() {
            false
        } else {
            let Coordinate(tx, ty) = *to;
            if let Some(_piece) = self.board[tx][ty] {
                false
            } else {
                let Coordinate(_fx, fy) = *from;
                let mut valid = false;
                if ty > fy && p.color == PieceColor::White {
                    // white moves down
                    valid = true;
                }
                if ty < fy && p.color == PieceColor::Black {
                    // black moves up
                    valid = true;
                }
                if ty > fy && p.color == PieceColor::Black && p.crowned {
                    // crowned black mv down
                    valid = true;
                }
                if ty < fy && p.color == PieceColor::White && p.crowned {
                    // crowned white mv up
                    valid = true;
                }
                valid
            }
        }
    }
}