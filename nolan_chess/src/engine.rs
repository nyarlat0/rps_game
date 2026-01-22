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

#[derive(Clone)]
pub enum EngineErr
{
    WrongPiece,
    AbsentBoard,
    WrongMove,
    InfLoop,
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

    pub fn make_preboard(&self, t: usize) -> Option<PreBoard>
    {
        let mut board = PreBoard::new(t);

        for (origin, wline) in self.wlines.iter().enumerate() {
            for (age, state) in wline.iter().enumerate() {
                if state.t == t && state.alive {
                    board.space[state.x][state.y].push(PieceView::from_state(state, origin, age));
                }
            }
        }

        if board.is_empty() {
            None
        } else {
            Some(board)
        }
    }

    pub fn make_board(&self, t: usize) -> Result<Board, BoardErr>
    {
        let mut pieces = Vec::new();
        let mut occ = [[None; 8]; 8];

        let preboard = self.make_preboard(t).ok_or(BoardErr::InvalidTime)?;

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
            };
        }

        self.boards = boards;
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

        self.make_preboards(&times)?;
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

        let mut times = Vec::new();

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

                // Update clear state border and save time for preboard updates later
                del_age = age;
                times.push(st.t);

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

                times.push(st.t);
            } else {
                break;
            }
        }

        self.make_preboards(&times)?;

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

    pub fn mass_ressurection(&mut self) -> Result<Option<()>, EngineErr>
    {
        let mut res = None::<()>;

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
    /// and creates preboards for specified times
    pub fn make_preboards(&mut self, times: &Vec<usize>) -> Result<(), EngineErr>
    {
        let mut pbs = HashMap::new();
        for &t in times {
            pbs.insert(t, PreBoard::new(t));
        }

        for (origin, wline) in self.wlines.iter().enumerate() {
            for (age, state) in wline.iter().enumerate() {
                if times.contains(&state.t) && state.alive {
                    let pb = pbs.get_mut(&state.t).ok_or(EngineErr::AbsentBoard)?;
                    pb.space[state.x][state.y].push(PieceView::from_state(state, origin, age));
                }
            }
        }

        for (t, pb) in pbs {
            self.preboards.insert(t, pb);
        }

        Ok(())
    }

    pub fn update_preboard(&mut self, t: usize) -> Result<(), EngineErr>
    {
        let pb = self.make_preboard(t).ok_or(EngineErr::AbsentBoard)?;
        self.preboards.insert(t, pb);

        Ok(())
    }

    pub fn update_infos(&mut self, times: &Vec<usize>) -> Result<(), EngineErr>
    {
        for &t in times {
            self.update_info(t)?;
        }

        Ok(())
    }

    pub fn update_info(&mut self, t: usize) -> Result<(), EngineErr>
    {
        let pb = self.preboards.get(&t).ok_or(EngineErr::AbsentBoard)?;

        todo!()
    }
}
