use serde::{Deserialize, Serialize};
use std::{cmp::min, collections::HashMap};

use crate::*;

type WorldLine = Vec<State>;

#[derive(Clone, Serialize, Deserialize)]
pub struct Engine
{
    pub wlines: Vec<WorldLine>,
    pub move_history: HashMap<usize, HashMap<usize, Move>>, // t -> fut_seen -> Move
    pub preboards: HashMap<usize, PreBoard>,
    pub boards: HashMap<usize, Board>,
    pub turn: Color,
    pub action_lines: (Option<usize>, Option<usize>),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct State
{
    pub t: usize,
    pub x: usize,
    pub y: usize,
    pub alive: bool,
    pub informed: Vec<(usize, usize)>, // (wid, pid)
    pub fut_seen: usize,
    pub p: Piece,
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
    pub origin: usize,
    pub age: usize,
    pub t: usize,
    pub fut_seen: usize,
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

impl BoardConflict {}

impl Engine
{
    /// Returns None if figure is dead
    pub fn get_piv(&self, wid: usize, pid: usize) -> Option<PieceView>
    {
        let line = self.wlines.get(wid)?;
        let state = line.get(pid)?.clone();

        if !state.alive {
            return None;
        }

        let Piece { kind,
                    color,
                    inverted,
                    active, } = state.p;

        let sq = Sq(state.x, state.y);
        let (origin, age, t, fut_seen) = (wid, pid, state.t, state.fut_seen);

        Some(PieceView { origin,
                         age,
                         t,
                         fut_seen,
                         sq,
                         kind,
                         color,
                         inverted,
                         active })
    }

    pub fn make_preboard(&self, t: usize) -> Option<PreBoard>
    {
        let mut board = PreBoard::new(t);

        for (origin, wline) in self.wlines.iter().enumerate() {
            for (age, state) in wline.iter().enumerate() {
                if state.t == t && state.alive {
                    let Piece { kind,
                                color,
                                inverted,
                                active, } = state.p;
                    let sq = Sq(state.x, state.y);
                    board.space[state.x][state.y].push(PieceView { origin,
                                                                   age,
                                                                   t,
                                                                   fut_seen: state.fut_seen,
                                                                   sq,
                                                                   kind,
                                                                   color,
                                                                   inverted,
                                                                   active });
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

    pub fn possible_moves(&self, wid: usize, pid: usize) -> Result<Vec<Move>, EngineErr>
    {
        let piv = self.get_piv(wid, pid).ok_or(EngineErr::WrongPiece)?;
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

    pub fn kill_piece(&mut self, wid: usize, pid: usize) -> Result<(), EngineErr>
    {
        let wl = self.wlines.get_mut(wid).ok_or(EngineErr::WrongPiece)?;

        let mut times = Vec::new();

        for st in &mut wl[pid..] {
            st.alive = false;
            times.push(st.t);
        }

        self.update_preboards(&times)?;
        Ok(())
    }

    pub fn undo_last_mv(&mut self, piv: PieceView) -> Result<(), EngineErr>
    {
        let wl = self.wlines.get(piv.origin).ok_or(EngineErr::WrongPiece)?;

        for st in wl[..piv.age].iter().rev() {
            if (st.x, st.y) != (piv.sq.0, piv.sq.1) {
                let mv = self.move_history
                             .get(&st.t)
                             .ok_or(EngineErr::WrongMove)?
                             .get(&st.fut_seen)
                             .ok_or(EngineErr::WrongMove)?;

                self.undo_mv(*mv)?;
                break;
            }
        }

        Ok(())
    }

    pub fn undo_mv(&mut self, mv: Move) -> Result<(), EngineErr>
    {
        let brd_moves = self.move_history
                            .get_mut(&mv.t)
                            .ok_or(EngineErr::WrongMove)?;
        brd_moves.remove(&mv.fut_seen).ok_or(EngineErr::WrongMove)?;

        let bf = self.get_piv(mv.origin, mv.age)
                     .ok_or(EngineErr::WrongPiece)?;

        let wl = self.wlines
                     .get_mut(mv.origin)
                     .ok_or(EngineErr::WrongPiece)?;

        let mut times = Vec::new();

        for st in wl[(mv.age + 1)..].iter_mut() {
            if (st.x, st.y) == (mv.sq.0, mv.sq.1) {
                st.x = bf.sq.0;
                st.y = bf.sq.1;
                times.push(st.t);
            } else {
                break;
            }
        }

        self.update_preboards(&times)?;

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

    pub fn update_preboards(&mut self, times: &Vec<usize>) -> Result<(), EngineErr>
    {
        for &t in times {
            let pb = self.make_preboard(t).ok_or(EngineErr::AbsentBoard)?;
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
