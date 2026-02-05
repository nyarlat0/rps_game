use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreBoard
{
    pub t: usize,
    pub space: [[Vec<PieceView>; 8]; 8],
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Board
{
    pub age: usize,
    pub pieces: Vec<PieceView>,
    pub occ: [[Option<usize>; 8]; 8],
}

impl PreBoard
{
    pub fn new(t: usize) -> Self
    {
        let space = std::array::from_fn(|_| std::array::from_fn(|_| Vec::new()));

        Self { t, space }
    }

    pub fn is_valid(&self) -> bool
    {
        for row in self.space.iter() {
            for pieces in row.iter() {
                if pieces.len() > 1 {
                    return false;
                }
            }
        }

        true
    }

    pub fn try_into_board(&self) -> Option<Board>
    {
        if !self.is_valid() {
            return None;
        }

        let preboard = self;
        let t = preboard.t;
        let mut pieces = Vec::new();
        let mut occ = [[None; 8]; 8];

        for (y, row) in preboard.space.iter().enumerate() {
            for (x, prep) in row.iter().enumerate() {
                if prep.len() > 1 {
                    return None;
                } else {
                    if let Some(p) = prep.get(0) {
                        let idx = pieces.len();
                        pieces.push(*p);
                        occ[x][y] = Some(idx);
                    }
                }
            }
        }

        Some(Board { pieces,
                     occ,
                     age: t })
    }

    pub fn is_empty(&self) -> bool
    {
        for row in self.space.iter() {
            for pieces in row.iter() {
                if !pieces.is_empty() {
                    return false;
                }
            }
        }

        true
    }

    pub fn find_piv(&self, origin: usize, age: usize) -> Option<PieceView>
    {
        for row in self.space.iter() {
            for pieces in row.iter() {
                let mb_p = pieces.iter()
                                 .find(|p| p.origin == origin && p.age == age)
                                 .cloned();
                if mb_p.is_some() {
                    return mb_p;
                }
            }
        }

        None
    }
}

impl Default for Board
{
    fn default() -> Self
    {
        let mut board = Self { age: 0,
                               pieces: Vec::new(),
                               occ: [[None; 8]; 8] };

        // helper to insert piece
        let mut next_id = 0;
        let mut add = |board: &mut Board, kind: PieceKind, color: Color, sq: Sq| {
            let id = next_id;
            next_id += 1;

            let piece = PieceView { origin: id,
                                    age: 0,
                                    t: 0,
                                    fut_seen: 0,
                                    loop_turn: 0,
                                    sq,
                                    kind,
                                    color,
                                    inverted: false,
                                    active: true };

            board.occ[sq.0 as usize][sq.1 as usize] = Some(id);
            board.pieces.insert(id, piece);
        };

        // --- WHITE ---
        let white = Color::White;

        // White major pieces (rank 0)
        add(&mut board, PieceKind::R { moved: false }, white, Sq(0, 0)); // a1
        add(&mut board, PieceKind::N, white, Sq(1, 0)); // b1
        add(&mut board, PieceKind::B, white, Sq(2, 0)); // c1
        add(&mut board, PieceKind::Q, white, Sq(3, 0)); // d1
        add(&mut board, PieceKind::K { moved: false }, white, Sq(4, 0)); // e1
        add(&mut board, PieceKind::B, white, Sq(5, 0)); // f1
        add(&mut board, PieceKind::N, white, Sq(6, 0)); // g1
        add(&mut board, PieceKind::R { moved: false }, white, Sq(7, 0)); // h1

        // White pawns (rank 1)
        for x in 0..8 {
            add(&mut board, PieceKind::P { moved: false }, white, Sq(x, 1));
        }

        // --- BLACK ---
        let black = Color::Black;

        // Black major pieces (rank 7)
        add(&mut board, PieceKind::R { moved: false }, black, Sq(0, 7)); // a8
        add(&mut board, PieceKind::N, black, Sq(1, 7)); // b8
        add(&mut board, PieceKind::B, black, Sq(2, 7)); // c8
        add(&mut board, PieceKind::Q, black, Sq(3, 7)); // d8
        add(&mut board, PieceKind::K { moved: false }, black, Sq(4, 7)); // e8
        add(&mut board, PieceKind::B, black, Sq(5, 7)); // f8
        add(&mut board, PieceKind::N, black, Sq(6, 7)); // g8
        add(&mut board, PieceKind::R { moved: false }, black, Sq(7, 7)); // h8

        // Black pawns (rank 6)
        for x in 0..8 {
            add(&mut board, PieceKind::P { moved: false }, black, Sq(x, 6));
        }

        board
    }
}

impl Board
{
    pub fn is_empty(&self, sq: Sq) -> bool
    {
        let Sq(x, y) = sq;
        self.occ[x as usize][y as usize].is_none()
    }

    pub fn is_fut_me(&self, piece: &PieceView, sq: Sq) -> bool
    {
        if let Some(p) = self.get_piece(sq.0, sq.1) {
            (p.age == piece.age + 1) && (p.origin == piece.origin)
        } else {
            false
        }
    }

    pub fn is_me(&self, piece: &PieceView, sq: Sq) -> bool
    {
        if let Some(p) = self.get_piece(sq.0, sq.1) {
            (p.age == piece.age) && (p.origin == piece.origin)
        } else {
            false
        }
    }

    pub fn color_at(&self, sq: Sq) -> Option<Color>
    {
        let Sq(x, y) = sq;
        self.occ[x as usize][y as usize].map(|i| self.pieces.get(i).map(|p| p.color))
                                        .flatten()
    }

    pub fn is_enemy(&self, who: Color, sq: Sq) -> bool
    {
        match self.color_at(sq) {
            Some(c) => c != who,
            None => false,
        }
    }

    pub fn is_inv(&self, sq: Sq) -> bool
    {
        if let Some(p) = self.get_piece(sq.0, sq.1) {
            p.inverted
        } else {
            false
        }
    }

    pub fn get_piece(&self, x: usize, y: usize) -> Option<&PieceView>
    {
        (x < 8 && y < 8).then(|| self.occ[x][y])
                        .flatten()
                        .and_then(|key| self.pieces.get(key))
    }

    pub fn is_inv_attacked(&self, fut_board: &Board, by: Color, sq: Sq) -> bool
    {
        let now_board = self;

        // pawns
        let dir = if matches!(by, Color::White) { 1 } else { -1 };
        for &dx in &[-1, 1] {
            if let Some(Sq(x, y)) = sq.shift(dx, -dir) {
                if let Some(p) = fut_board.get_piece(x, y) {
                    if p.color == by && matches!(p.kind, PieceKind::P { .. }) && p.inverted {
                        return true;
                    }
                }
            }
        }

        // knights
        for &(dx, dy) in KNIGHT_DELTAS {
            if let Some(Sq(x, y)) = sq.shift(dx, dy) {
                if let Some(p) = fut_board.get_piece(x, y) {
                    if p.color == by && matches!(p.kind, PieceKind::N) && p.inverted {
                        return true;
                    }
                }
            }
        }

        // kings
        for &(dx, dy) in KING_DELTAS {
            if let Some(Sq(x, y)) = sq.shift(dx, dy) {
                if let Some(p) = fut_board.get_piece(x, y) {
                    if p.color == by && matches!(p.kind, PieceKind::K { .. }) && p.inverted {
                        return true;
                    }
                }
            }
        }

        // bishops/queens
        for &(dx, dy) in BISHOP_DIRS {
            let mut cur = sq;
            loop {
                match cur.shift(dx, dy) {
                    Some(nsq @ Sq(x, y)) => {
                        if let Some(p) = fut_board.get_piece(x, y) {
                            if p.color == by
                               && matches!(p.kind, PieceKind::B | PieceKind::Q)
                               && p.inverted
                            {
                                return true;
                            } else if now_board.get_piece(x, y).is_some() {
                                break;
                            }
                        }
                        cur = nsq;
                    }
                    None => break,
                }
            }
        }

        // rooks/queens
        for &(dx, dy) in ROOK_DIRS {
            let mut cur = sq;
            loop {
                match cur.shift(dx, dy) {
                    Some(nsq @ Sq(x, y)) => {
                        if let Some(p) = fut_board.get_piece(x, y) {
                            if p.color == by
                               && matches!(p.kind, PieceKind::R { .. } | PieceKind::Q)
                               && p.inverted
                            {
                                return true;
                            } else if now_board.get_piece(x, y).is_some() {
                                break;
                            }
                        }
                        cur = nsq;
                    }
                    None => break,
                }
            }
        }

        false
    }

    pub fn is_attacked(&self, by: Color, sq: Sq) -> bool
    {
        let board = self;

        // pawns
        let dir = if matches!(by, Color::White) { 1 } else { -1 };
        for &dx in &[-1, 1] {
            if let Some(Sq(x, y)) = sq.shift(dx, -dir) {
                if let Some(p) = board.get_piece(x, y) {
                    if p.color == by && matches!(p.kind, PieceKind::P { .. }) {
                        return true;
                    }
                }
            }
        }

        // knights
        for &(dx, dy) in KNIGHT_DELTAS {
            if let Some(Sq(x, y)) = sq.shift(dx, dy) {
                if let Some(p) = board.get_piece(x, y) {
                    if p.color == by && matches!(p.kind, PieceKind::N) {
                        return true;
                    }
                }
            }
        }

        // kings
        for &(dx, dy) in KING_DELTAS {
            if let Some(Sq(x, y)) = sq.shift(dx, dy) {
                if let Some(p) = board.get_piece(x, y) {
                    if p.color == by && matches!(p.kind, PieceKind::K { .. }) {
                        return true;
                    }
                }
            }
        }

        // bishops/queens
        for &(dx, dy) in BISHOP_DIRS {
            let mut cur = sq;
            loop {
                match cur.shift(dx, dy) {
                    Some(nsq @ Sq(x, y)) => {
                        if let Some(p) = board.get_piece(x, y) {
                            if p.color == by && matches!(p.kind, PieceKind::B | PieceKind::Q) {
                                return true;
                            }
                            break; // blocked
                        }
                        cur = nsq;
                    }
                    None => break,
                }
            }
        }

        // rooks/queens
        for &(dx, dy) in ROOK_DIRS {
            let mut cur = sq;
            loop {
                match cur.shift(dx, dy) {
                    Some(nsq @ Sq(x, y)) => {
                        if let Some(p) = board.get_piece(x, y) {
                            if p.color == by && matches!(p.kind, PieceKind::R { .. } | PieceKind::Q)
                            {
                                return true;
                            }
                            break;
                        }
                        cur = nsq;
                    }
                    None => break,
                }
            }
        }

        false
    }

    pub fn find_by_origin(&self, origin: usize) -> Vec<PieceView>
    {
        self.pieces
            .iter()
            .filter(|p| p.origin == origin)
            .map(|p| p.clone())
            .collect()
    }

    pub fn calc_info(&mut self) -> Vec<(usize, usize)>
    {
        let side = Color::from_board(self.age);
        let mut inf = Vec::new();

        for p in self.pieces.iter() {
            if p.color != side {
                continue;
            }

            let v = p.moves(self);

            for sq in v.iter() {
                if self.is_enemy(side, *sq) {
                    let enemy = self.get_piece(sq.0, sq.1).unwrap();
                    inf.push((enemy.origin, enemy.age));
                }
            }
        }

        inf
    }

    pub fn is_playable(&self) -> bool
    {
        self.pieces.iter().any(|p| p.active)
    }

    /*
    pub fn complete_castling(&mut self, by: Color, castling: Castling)
    {
        let (ini_rook_sq, end_rook_sq) = match (by, castling) {
            (Color::White, Castling::Short) => (Sq(7, 0), Sq(5, 0)), // h1 → f1
            (Color::White, Castling::Long) => (Sq(0, 0), Sq(3, 0)),  // a1 → d1

            (Color::Black, Castling::Short) => (Sq(7, 7), Sq(5, 7)), // h8 → f8
            (Color::Black, Castling::Long) => (Sq(0, 7), Sq(3, 7)),  // a8 → d8
        };

        let (rook_id, r) = self.get_mut_piece(ini_rook_sq.0, ini_rook_sq.1).unwrap();
        r.kind = r.kind.set_moved(true);

        let _ = self.move_piece(rook_id, end_rook_sq);
    }
    */

    pub fn has_piece(&self, origin: usize, age: usize) -> bool
    {
        self.pieces
            .iter()
            .any(|p| p.origin == origin && p.age == age)
    }
}
