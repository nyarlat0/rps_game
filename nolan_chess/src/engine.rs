use serde::{Deserialize, Serialize};
use std::{cmp::min, collections::HashMap};

use crate::*;

#[derive(Clone)]
pub struct Engine
{
    pub wlines: Vec<Vec<State>>, // wid -> age -> state
    pub move_history: HashMap<MoveId, Move>,
    pub preboards: HashMap<usize, PreBoard>,
    pub boards: HashMap<usize, Board>,
    pub pbuf: HashMap<PieceId, (usize, usize)>, // pieceid -> (wid, age)
    pub turn: Color,
    pub action_lines: (Option<usize>, Option<usize>),
}

#[derive(Clone)]
pub struct State
{
    pub t: usize,
    pub x: usize,
    pub y: usize,
    pub fut_seen: usize,
    pub loop_turn: usize,
    pub alive: bool,
    pub active: bool,
    pub kind: PieceKind,
    pub color: Color,
    pub inverted: bool,
    pub informed: Vec<PieceId>,
}

/// Long-term identification of a move
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MoveId
{
    t: usize,
    fut_seen: usize,
    loop_turn: usize,
}

/// Long-term identification of a piece
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PieceId
{
    pub wid: usize,
    pub fut_seen: usize,
    pub loop_turn: usize,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Castling
{
    Long,
    Short,
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Move
{
    pub pid: PieceId,
    pub t: usize,
    pub sq: Sq,
    pub to_past: bool,
    pub castling: Option<Castling>,
    pub en_passant: Option<Sq>,
    pub promotion: Option<PieceKind>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ValidRes
{
    HistChanged,
    Unchanged,
}

#[derive(Clone)]
pub enum EngineErr
{
    WrongPiece,
    AbsentBoard,
    WrongMove,
    InfLoop,
    NoMoves,
    BoardErr(BoardErr),
}

#[derive(Clone)]
pub enum BoardErr
{
    InvalidTime,
    InvalidBoard(BoardConflict),
}

#[derive(Clone)]
pub struct BoardConflict
{
    pivs: Vec<PieceView>,
}

#[derive(Clone, Copy)]
pub struct PieceCollision
{
    bf_collider: PieceView,
    collider: PieceView,
    collided: PieceView,
}

impl PieceId
{
    pub fn from_state(st: &State, wid: usize) -> Self
    {
        Self { wid,
               fut_seen: st.fut_seen,
               loop_turn: st.loop_turn }
    }
    pub fn from_piv(piv: PieceView) -> Self
    {
        Self { wid: piv.origin,
               fut_seen: piv.fut_seen,
               loop_turn: piv.loop_turn }
    }
}

impl MoveId
{
    pub fn from_piv(piv: PieceView) -> Self
    {
        Self { t: piv.t,
               fut_seen: piv.fut_seen,
               loop_turn: piv.loop_turn }
    }

    pub fn from_state(st: &State) -> Self
    {
        Self { t: st.t,
               fut_seen: st.fut_seen,
               loop_turn: st.loop_turn }
    }
}

impl Default for Engine
{
    fn default() -> Self
    {
        // Board coordinate system:
        // (x, y) with x=0..7 left->right, y=0..7 bottom->top.
        // White starts at y=0/1, Black at y=7/6.

        let mut wlines: Vec<Vec<State>> = Vec::new();
        let pbuf: HashMap<PieceId, (usize, usize)> = HashMap::new();

        // Helper: create initial state at t=0, fut_seen=0, loop_turn=0.
        let mut spawn = |kind: PieceKind, color: Color, x: usize, y: usize| {
            let st = State { t: 0,
                             x,
                             y,
                             fut_seen: 0,
                             loop_turn: 0,
                             alive: true,
                             active: true,
                             kind,
                             color,
                             inverted: false,
                             informed: Vec::new() };

            // one worldline per piece, age 0 only
            wlines.push(vec![st]);
        };

        // --- WHITE ---
        let w = Color::White;

        // rank 0 (y=0)
        spawn(PieceKind::R { moved: false }, w, 0, 0);
        spawn(PieceKind::N, w, 1, 0);
        spawn(PieceKind::B, w, 2, 0);
        spawn(PieceKind::Q, w, 3, 0);
        spawn(PieceKind::K { moved: false }, w, 4, 0);
        spawn(PieceKind::B, w, 5, 0);
        spawn(PieceKind::N, w, 6, 0);
        spawn(PieceKind::R { moved: false }, w, 7, 0);

        // rank 1 (y=1)
        for x in 0..8 {
            spawn(PieceKind::P { moved: false }, w, x, 1);
        }

        // --- BLACK ---
        let b = Color::Black;

        // rank 7 (y=7)
        spawn(PieceKind::R { moved: false }, b, 0, 7);
        spawn(PieceKind::N, b, 1, 7);
        spawn(PieceKind::B, b, 2, 7);
        spawn(PieceKind::Q, b, 3, 7);
        spawn(PieceKind::K { moved: false }, b, 4, 7);
        spawn(PieceKind::B, b, 5, 7);
        spawn(PieceKind::N, b, 6, 7);
        spawn(PieceKind::R { moved: false }, b, 7, 7);

        // rank 6 (y=6)
        for x in 0..8 {
            spawn(PieceKind::P { moved: false }, b, x, 6);
        }

        // If you truly want everything else "inferred", keep them empty here.
        // (Your later build step can materialize Board/PreBoard from wlines[piece][age].)
        Self { wlines,
               move_history: HashMap::new(),
               preboards: HashMap::new(),
               boards: HashMap::new(),
               pbuf,
               turn: Color::White,
               action_lines: (Some(0), None) }
    }
}

impl Engine
{
    /// Returns None if figure is dead
    pub fn get_piv(&self, wid: usize, age: usize) -> Option<PieceView>
    {
        let line = self.wlines.get(wid)?;
        let state = line.get(age)?.clone();

        if !state.alive {
            return None;
        }

        Some(PieceView::from_state(&state, wid, age))
    }

    /// Returns None only if state not found.
    /// Looses info about live/death!!!
    pub fn get_piv_necro(&self, wid: usize, age: usize) -> Option<PieceView>
    {
        let line = self.wlines.get(wid)?;
        let state = line.get(age)?.clone();

        Some(PieceView::from_state(&state, wid, age))
    }

    /// Returns None if figure is dead
    pub fn get_piv_from_mv(&self, mv: Move) -> Option<PieceView>
    {
        let (wid, age) = *self.pbuf.get(&mv.pid)?;
        self.get_piv(wid, age)
    }

    pub fn make_board(&self, t: usize) -> Result<Board, BoardErr>
    {
        let mut pieces = Vec::new();
        let mut occ = [[None; 8]; 8];

        let preboard = self.preboards.get(&t).ok_or(BoardErr::InvalidTime)?;

        for (y, row) in preboard.space.iter().enumerate() {
            for (x, prep) in row.iter().enumerate() {
                if prep.len() > 1 {
                    return Err(BoardErr::InvalidBoard(BoardConflict { pivs: prep.clone() }));
                } else {
                    if let Some(p) = prep.get(0) {
                        let idx = pieces.len();
                        pieces.push(*p);
                        occ[x][y] = Some(idx);
                    }
                }
            }
        }

        Ok(Board { pieces,
                   occ,
                   age: t })
    }

    pub fn possible_moves(&self, wid: usize, age: usize) -> Result<Vec<Move>, EngineErr>
    {
        let piv = self.get_piv(wid, age).ok_or(EngineErr::WrongPiece)?;
        let pres_board = self.make_board(piv.t)
                             .map_err(|err| EngineErr::BoardErr(err))?;

        let fut_board = match self.make_board(piv.t + 1) {
            Ok(b) => b,
            Err(err) => match err {
                BoardErr::InvalidTime => pres_board,
                BoardErr::InvalidBoard(bc) => {
                    todo!()
                }
            },
        };

        todo!()
    }

    pub fn build_boards(&mut self) -> Result<(), EngineErr>
    {
        let mut boards = HashMap::new();
        let mut t = 0;
        let mut pb_buf = vec![self.preboards.clone()];

        loop {
            let Some(pb) = self.preboards.get(&t) else {
                break;
            };

            let last_t = match self.turn {
                Color::White => self.action_lines.0.ok_or(EngineErr::NoMoves)?,
                Color::Black => self.action_lines.1.ok_or(EngineErr::NoMoves)?,
            };

            if t > last_t {
                break;
            }

            if pb.is_valid() {
                //
                // If board is valid then push it and go forward
                //
                let brd = pb.try_into_board().ok_or(EngineErr::AbsentBoard)?;
                boards.insert(t, brd);
                t += 1;
            } else {
                //
                // Else fix board
                //
                self.fix_board(t)?;
                if pb_buf.contains(&self.preboards) {
                    // If state of engine repeats then throw infinite loop err
                    return Err(EngineErr::InfLoop);
                } else {
                    // Else save new state and start boards calculation anew
                    pb_buf.push(self.preboards.clone());
                    t = 0;
                }
            }
        }

        self.boards = boards;
        Ok(())
    }

    pub fn validate_moves(&mut self) -> Result<ValidRes, EngineErr>
    {
        todo!()
    }

    pub fn global_validation(&mut self) -> Result<(), EngineErr>
    {
        let mut pb_buf = vec![];

        loop {
            // Stage 0 -- compare and safe engine state

            if pb_buf.contains(&self.preboards) {
                // If state of engine repeats then throw infinite loop err
                return Err(EngineErr::InfLoop);
            } else {
                // Else save new state
                pb_buf.push(self.preboards.clone());
            }

            // Stage 1 -- build boards

            self.build_boards()?;

            match self.validate_moves()? {
                ValidRes::HistChanged => continue,
                ValidRes::Unchanged => (),
            }

            match self.mass_ressurection()? {
                ValidRes::HistChanged => continue,
                ValidRes::Unchanged => (),
            }

            self.update_infos()?;
            break;
        }

        Ok(())
    }

    pub fn fix_board(&mut self, t: usize) -> Result<(), EngineErr>
    {
        while let Err(BoardErr::InvalidBoard(bc)) = self.make_board(t) {
            self.fix_conflict(bc)?;
        }

        Ok(())
    }

    /// Conflict is any intersection of n pieces.
    /// Is fixed by separating pieces to pairs,
    /// tracking each pair collision (root of conflict) and fixing it.
    pub fn fix_conflict(&mut self, mut bc: BoardConflict) -> Result<(), EngineErr>
    {
        // Sort by future seen for casuality
        bc.pivs.sort_by(|a, b| a.fut_seen.cmp(&b.fut_seen));

        // Get pairs of conflicting pieces
        let mut iter = bc.pivs.into_iter();
        let mut prev_survived = None::<PieceView>;

        while let Some(piv) = iter.next() {
            let other_piv = {
                if let Some(surv) = prev_survived {
                    surv
                } else if let Some(next_piv) = iter.next() {
                    next_piv
                } else {
                    break;
                }
            };

            let col = self.track_collision(piv, other_piv)
                          .ok_or(EngineErr::WrongPiece)?;

            prev_survived = Some(self.fix_collision(col, piv.t)?);
        }

        Ok(())
    }

    /// Collision is the root of intersection of figures (aka conflict).
    /// Collision time is the first conflict time.
    /// Method undoes move or kills piece, returns the piece that stays unchanged.
    /// Gets conflict time as input to return correct survivor in case of mutual destruction.
    pub fn fix_collision(&mut self,
                         col: PieceCollision,
                         conf_t: usize)
                         -> Result<PieceView, EngineErr>
    {
        let PieceCollision { bf_collider,
                             collider,
                             collided, } = col;
        let surv: PieceView;

        // Pieces of the same side
        if collider.color == collided.color {
            // In oncoming collision
            // collider = ordinary,
            // collided = inverted
            if collider.inverted != collided.inverted {
                // Inverted kills ordinary on board of its color
                if Color::from_board(collider.t) == collider.color {
                    self.kill_piece(collider.origin, collider.age)?;
                    surv = collided;
                } else {
                    // Else undo inverted move
                    self.undo_last_mv(collided)?;
                    surv = collider;
                }
            } else {
                // In co-directional undo collider move
                self.undo_last_mv(collider)?;
                surv = collided;
            }
        // Enemy pieces
        } else {
            // In oncoming collision
            // collider = ordinary,
            // collided = inverted
            if collider.inverted != collided.inverted {
                // If ordinary moved and knew future before collision
                // then it kills inverted in collision board,
                // but is killed after collision board
                if bf_collider.sq != collider.sq && bf_collider.knows_future() {
                    self.kill_piece(collided.origin, collided.age)?;
                    self.kill_piece(collider.origin, collider.age + 1)?;

                    if conf_t == collider.t {
                        surv = collider;
                    } else {
                        surv = collided;
                    }
                } else {
                    // Else just kill ordinary
                    self.kill_piece(collider.origin, collider.age)?;
                    surv = collided;
                }
            } else {
                // Else collided is killed
                self.kill_piece(collided.origin, collided.age)?;
                surv = collider;
            }
        }

        Ok(surv)
    }

    /// Collision is the root of intersection of figures (aka conflict).
    /// Collision time is the first conflict time.
    /// If oncoming (inverted vs ordinary) collision,
    /// always return ordinary as collider and inv as collided.
    pub fn track_collision(&self, piv: PieceView, other_piv: PieceView) -> Option<PieceCollision>
    {
        // First checked for sq change is piv with
        // most future seen in case both figures move
        // at the same time
        let (piv, other_piv) = if piv.fut_seen >= other_piv.fut_seen {
            (piv, other_piv)
        } else {
            (other_piv, piv)
        };

        // Oncoming collision can have some exotic situations
        let oncoming = piv.inverted != other_piv.inverted;

        if !oncoming {
            let mut prev_piv = piv;
            let mut prev_other_piv = other_piv;

            for d in 1..=min(piv.age, other_piv.age) {
                let piv_bf = self.get_piv(piv.origin, piv.age.saturating_sub(d))?;
                let other_piv_bf = self.get_piv(other_piv.origin, other_piv.age.saturating_sub(d))?;

                if piv_bf.sq != piv.sq {
                    let bf_collider = piv_bf;
                    let collider = prev_piv;
                    let collided = prev_other_piv;
                    return Some(PieceCollision { bf_collider,
                                                 collider,
                                                 collided });
                }

                if other_piv_bf.sq != other_piv.sq {
                    let bf_collider = other_piv_bf;
                    let collider = prev_other_piv;
                    let collided = prev_piv;
                    return Some(PieceCollision { bf_collider,
                                                 collider,
                                                 collided });
                }

                prev_piv = piv_bf;
                prev_other_piv = other_piv_bf;
            }
        } else {
            let (inv, ord) = if piv.inverted {
                (piv, other_piv)
            } else {
                (other_piv, piv)
            };

            let mut prev_inv = inv;
            let mut prev_ord = ord;

            for d in 1..=ord.age {
                let ord_bf = self.get_piv(piv.origin, piv.age.saturating_sub(d))?;
                let mb_inv_aft = self.get_piv(other_piv.origin, other_piv.age + d);

                if let Some(inv_aft) = mb_inv_aft {
                    if ord_bf.sq != ord.sq {
                        let bf_collider = ord_bf;
                        let collider = prev_ord;
                        let collided = prev_inv;
                        return Some(PieceCollision { bf_collider,
                                                     collider,
                                                     collided });
                    }

                    if inv_aft.sq != inv.sq {
                        let bf_collider = ord_bf;
                        let collider = prev_ord;
                        let collided = prev_inv;
                        return Some(PieceCollision { bf_collider,
                                                     collider,
                                                     collided });
                    }

                    prev_ord = ord_bf;
                    prev_inv = inv_aft;
                } else {
                    // If inverted isnt present its just before collision
                    let bf_collider = ord_bf;
                    let collider = prev_ord;
                    let collided = prev_inv;
                    return Some(PieceCollision { bf_collider,
                                                 collider,
                                                 collided });
                }
            }
        }

        None
    }

    pub fn kill_piece(&mut self, wid: usize, age: usize) -> Result<(), EngineErr>
    {
        let wl = self.wlines.get_mut(wid).ok_or(EngineErr::WrongPiece)?;

        let mut times = Vec::new();

        for st in &mut wl[age..] {
            st.alive = false;
            times.push(st.t);
        }

        self.update_bufs();
        Ok(())
    }

    pub fn undo_last_mv(&mut self, piv: PieceView) -> Result<(), EngineErr>
    {
        let wl = self.wlines.get(piv.origin).ok_or(EngineErr::WrongPiece)?;

        for st in wl[..piv.age].iter().rev() {
            if (st.x, st.y) != (piv.sq.0, piv.sq.1) {
                self.undo_mv(MoveId::from_state(st))?;
                break;
            }
        }

        Ok(())
    }

    /// Clears move from move history,
    /// undoes position change of the piece
    /// and makes it active again
    pub fn undo_mv(&mut self, mvid: MoveId) -> Result<(), EngineErr>
    {
        // Remove old move
        let mv = self.move_history
                     .remove(&mvid)
                     .ok_or(EngineErr::WrongMove)?;

        // Piv before move, undid states will be set to it
        let bf_piv = self.get_piv_from_mv(mv).ok_or(EngineErr::WrongPiece)?;

        // Piv just after undid move,
        // is used to know what posion to undo
        let mut ud_piv = self.get_piv_necro(bf_piv.origin, bf_piv.age + 1)
                             .ok_or(EngineErr::WrongPiece)?;

        let wl = self.wlines
                     .get_mut(bf_piv.origin)
                     .ok_or(EngineErr::WrongPiece)?;

        // Make piece active again
        {
            let mv_st = wl.get_mut(bf_piv.age).ok_or(EngineErr::WrongPiece)?;
            mv_st.active = true;
        }

        // If move was to past then clear the whole backward loop
        // and its moves
        if mv.to_past {
            let mut del_age = bf_piv.age + 1;
            let mut prev_piv = bf_piv;

            for (age, st) in wl.iter_mut().enumerate().skip(bf_piv.age + 1) {
                // As soon as state time is past action point
                // backward loop is finished
                if st.t > mv.t {
                    break;
                }

                // If it moved again then clear move and update undid piv
                if (st.x, st.y) != (ud_piv.sq.0, ud_piv.sq.1) {
                    self.move_history
                        .remove(&MoveId::from_piv(prev_piv))
                        .ok_or(EngineErr::WrongMove)?;

                    ud_piv = PieceView::from_state(st, bf_piv.origin, age);
                }

                // Update clear state border
                del_age = age;

                prev_piv = PieceView::from_state(st, bf_piv.origin, age);
            }

            // Clear backward loop states from worldline
            wl.drain((bf_piv.age + 1)..=del_age);
        }

        for st in wl.iter_mut().skip(bf_piv.age + 1) {
            if (st.x, st.y) == (ud_piv.sq.0, ud_piv.sq.1) {
                st.x = bf_piv.sq.0;
                st.y = bf_piv.sq.1;
                st.kind = bf_piv.kind;
            } else {
                break;
            }
        }

        self.update_bufs();

        if let Some(cast) = mv.castling {
            let (old_sq, new_sq) = match cast {
                Castling::Long => match bf_piv.color {
                    Color::White => (Sq(0, 0), Sq(3, 0)),
                    Color::Black => (Sq(0, 7), Sq(3, 7)),
                },
                Castling::Short => match bf_piv.color {
                    Color::White => (Sq(7, 0), Sq(5, 0)),
                    Color::Black => (Sq(7, 7), Sq(5, 7)),
                },
            };

            for (wid, wl) in self.wlines.iter().enumerate() {}
        }

        Ok(())
    }

    fn rm_mv(&mut self, mvid: MoveId) -> Result<(), EngineErr>
    {
        // Remove old move
        self.move_history
            .remove(&mvid)
            .ok_or(EngineErr::WrongMove)?;

        Ok(())
    }

    pub fn mass_ressurection(&mut self) -> Result<ValidRes, EngineErr>
    {
        let mut res = ValidRes::Unchanged;

        for wl in self.wlines.iter_mut() {
            for st in wl.iter_mut() {
                if !st.alive {
                    let pb = self.preboards.get(&st.t).ok_or(EngineErr::AbsentBoard)?;
                }
            }
        }

        todo!()
    }

    /// Cycles through all world lines
    /// builds preboards and fills piece buffer,
    /// updates action lines
    pub fn update_bufs(&mut self)
    {
        let mut pbs = HashMap::<usize, PreBoard>::new();
        let mut pbuf = HashMap::<PieceId, (usize, usize)>::new();
        let mut actl = (None::<usize>, None::<usize>);

        for (origin, wline) in self.wlines.iter().enumerate() {
            for (age, state) in wline.iter().enumerate() {
                // push to preboards only alive pieces
                if state.alive {
                    // if state is active update the action line
                    if state.active {
                        match state.color {
                            Color::White => {
                                actl.0 = Some(actl.0.map_or(state.t, |l| l.min(state.t)))
                            }
                            Color::Black => {
                                actl.1 = Some(actl.1.map_or(state.t, |l| l.min(state.t)))
                            }
                        }
                    }

                    if let Some(pb) = pbs.get_mut(&state.t) {
                        pb.space[state.x][state.y].push(PieceView::from_state(state, origin, age));
                    } else {
                        let mut pb = PreBoard::new(state.t);
                        pb.space[state.x][state.y].push(PieceView::from_state(state, origin, age));
                        pbs.insert(state.t, pb);
                    }
                }
                // push to pbuf indifferently
                pbuf.insert(PieceId::from_state(state, origin), (origin, age));
            }
        }

        self.preboards = pbs;
        self.pbuf = pbuf;
    }

    pub fn update_infos(&mut self) -> Result<(), EngineErr>
    {
        todo!()
    }

    pub fn update_info(&mut self, t: usize) -> Result<(), EngineErr>
    {
        let pb = self.preboards.get(&t).ok_or(EngineErr::AbsentBoard)?;

        todo!()
    }
}
