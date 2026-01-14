use leptos::prelude::*;
use leptos_fluent::tr;
use leptos_use::{core::ConnectionReadyState, use_timeout_fn, UseTimeoutFnReturn};

use nolan_chess::{Board, Color, Engine, Move as ChessMove, Piece, PieceKind, Sq};
use shared::{
    auth::UserInfo,
    chess::{ChessGameInfo, ChessGameReq, ChessGameState},
    game::{GameError, GameResult},
    ws_messages::{ClientMsg, ServerMsg},
};

use crate::hooks::{MyToaster, WebsocketContext};

#[derive(Clone, Copy)]
struct ModeCtx((ReadSignal<Option<bool>>, WriteSignal<Option<bool>>));

#[component]
pub fn NolanChess() -> impl IntoView
{
    let (local, set_local) = signal(None::<bool>);

    provide_context(ModeCtx((local, set_local)));

    MultiPlay
}

#[component]
pub fn ChooseMode() -> impl IntoView
{
    let ModeCtx((_local, set_local)) = expect_context::<ModeCtx>();

    view! {
        <div class="card stack fill-page">
            <h1>"Nolan Chess"</h1>

            <h2>"Choose your mode:"</h2>

            /*
            <button
            on:click=move |_| {set_local.set(Some(true));}
            >
                "Local"
            </button>
            */

            <button
            on:click=move |_| {set_local.set(Some(false));}
            >
                "Multiplayer"
            </button>

            <div class="cluster" style="--cluster-justify: center; margin-top: auto;">
            <a href = "/games" class="button secondary" style="width: 50%;">
                { move || tr!("rps-other-games") }
            </a>
            <a href = "/" class="button secondary" style="width: calc(50% - 1rem);">
                { move || tr!("rps-home") }
            </a>
            </div>
        </div>
    }
}

#[component]
pub fn LocalPlay() -> impl IntoView
{
    view! {
        <div class="card stack fill-page">
        </div>
    }
}

#[component]
pub fn MultiPlay() -> impl IntoView
{
    let ws = expect_context::<WebsocketContext>();
    let user_info = expect_context::<UserInfo>();

    let (curr_game, set_curr_game) = signal::<Option<ChessGameState>>(None);
    let (curr_mv, set_curr_mv) = signal::<Option<ChessMove>>(None);
    let (can_leave, set_can_leave) = signal(false);

    let (my_turn, set_my_turn) = signal(false);

    let toaster = MyToaster::new();
    let (engine, set_engine) = signal(Engine::new());

    let UseTimeoutFnReturn { start: timer_start, .. } = {
        let user_info = user_info.clone();
        use_timeout_fn(move |()| {
                           if let Some(ChessGameState::Game { players,
                                                              last_move,
                                                              turn, }) = curr_game.get()
                           {
                               set_can_leave.set(true);
                           }
                       },
                       20_000.0)
    };

    Effect::new({
        let ws = ws.clone();
        let toaster = toaster.clone();

        move |_| {
            if let Some(msg) = ws.message.get() {
                if let ServerMsg::ChessGameMsg(ch_state) = msg {
                    if matches!(ch_state, ChessGameState::Game { .. }) {
                        set_can_leave.set(false);
                        timer_start(());
                    }
                    set_curr_game.set(Some(ch_state));
                } else if let ServerMsg::GameErrorMsg(GameError::Disconnected) = msg {
                    set_curr_game.set(None);
                    set_curr_mv.set(None);
                    set_engine.set(Engine::new());

                    let msg = tr!("rps-opponent-disconnected");
                    toaster.error(&msg);
                    ws.send(ClientMsg::ChessGameMsg(ChessGameReq::Start));
                }
            };
        }
    });

    Effect::new({
        let ws = ws.clone();
        move |_| {
            if ws.state.get() == ConnectionReadyState::Open {
                ws.send(ClientMsg::ChessGameMsg(ChessGameReq::Start));
            }
        }
    });

    let leave_btn = {
        let ws = ws.clone();
        move |_| {
            if ws.state.get() == ConnectionReadyState::Open {
                ws.send(ClientMsg::ChessGameMsg(ChessGameReq::Leave));
                ws.send(ClientMsg::ChessGameMsg(ChessGameReq::Start));
                set_curr_game.set(None);
                set_curr_mv.set(None);
                set_engine.set(Engine::new());
            }
        }
    };

    let next_btn = {
        let ws = ws.clone();
        move |_| {
            if ws.state.get() == ConnectionReadyState::Open {
                ws.send(ClientMsg::ChessGameMsg(ChessGameReq::Start));
                set_curr_game.set(None);
                set_curr_mv.set(None);
                set_engine.set(Engine::new());
            }
        }
    };

    view! {
        {move || {
            let toaster = toaster.clone();
            match curr_game.get() {
                None => {
                    set_can_leave.set(false);
                    view!{
                        <p>{ tr!("rps-waiting") }</p>
                        <div class="loading-spinner" style="margin-top: auto; margin-bottom: auto;"></div>
                    }.into_any()
                },

                Some(ChessGameState::Game { players, last_move, turn }) => {
                    let (opp_name, my_side, opp_side) = if players[0] == user_info.username {
                        (players[1].clone(), Color::White, Color::Black)
                    } else {
                        (players[0].clone(), Color::Black, Color::White)
                    };

                    view! {
                        <svg class="icon"
        style=format!("inline-size: {0}cap; block-size: {0}cap;", 1.5)
        aria-hidden="true">
            <use href="/icons.svg#chess-pawn"></use>
        </svg>
                        <For
                            each=move || engine.get().board_history
                            key=|board| board.age
                            children=move |board| {
                                let toaster = toaster.clone();
                                view!{
                                    <BoardDisplay
                                        board
                                        engine
                                        on_error=Callback::new(move |s: String| {
                                            let prefix = tr!("forum-action-error");
                                            let msg = format!("{prefix} ({s})");
                                            toaster.error(&msg);
                                        })
                                    />
                                }
                            }
                        />
                        <p>
                            "Playing against "{opp_name}
                        </p>

                    }.into_any()
                }
                Some(ChessGameState::Finished( ChessGameInfo{players, engine} )) => {
                    view! {}.into_any()
                }
            }
        }}
        <div class="stack" style="margin-top: auto; --stack-gap: var(--s0);">
            <button
            class:el-hide=move || !curr_game.get().is_some_and(|g| matches!(g, ChessGameState::Finished{..}))
            on:click=next_btn>
                { move || tr!("rps-next-game") }
            </button>
            <button
            class="secondary destructive"
            class:el-hide=move || {!can_leave.get()}
            on:click=leave_btn>
                { move || tr!("rps-leave") }
            </button>
            <div class="cluster" style="--cluster-justify: center;">
            <a href = "/games" class="button secondary" style="width: 50%;">
                { move || tr!("rps-other-games") }
            </a>
            <a href = "/" class="button secondary" style="width: calc(50% - 1rem);">
                { move || tr!("rps-home") }
            </a>
            </div>
            </div>
    }
}

fn all_squares_white_view() -> Vec<Sq>
{
    (0..8).rev() // y: 7 → 0 (top → bottom)
          .flat_map(|y| (0..8).map(move |x| Sq(x, y)))
          .collect()
}

#[component]
fn BoardDisplay(board: Board,
                engine: ReadSignal<Engine>,
                on_error: Callback<String>)
                -> impl IntoView
{
    let squares = Memo::new(|_| all_squares_white_view());
    view! {
        <div class="chess-board">
            <For
            each=move || squares.get()
            key=|sq| (sq.0, sq.1)
            children=move |sq| {
                let board = board.clone();

                view! {
                    <div
                        class="square"
                        class:dark=((sq.0 + sq.1) % 2 == 1)
                        class:light=((sq.0 + sq.1) % 2 == 0)
                    >
                        {
                            match board.occ[sq.0 as usize][sq.1 as usize] {
                                None => None,
                                Some(pid) => {
                                    board.pieces.get(&pid).map(|p| {
                                        view! { <PieceDisplay piece = *p/> }
                                    })
                                }
                            }
                        }
                    </div>
                }
            }
            />
        </div>
    }
}

#[component]
fn PieceDisplay(piece: Piece) -> impl IntoView
{
    let svg_path = match piece.kind {
        PieceKind::P { .. } => "/icons.svg#chess-pawn",
        PieceKind::N => "/icons.svg#chess-knight",
        PieceKind::R { .. } => "/icons.svg#chess-rook",
        PieceKind::B => "/icons.svg#chess-bishop",
        PieceKind::Q => "/icons.svg#chess-king",
        PieceKind::K { .. } => "/icons.svg#chess-king",
    };

    let size = 1.5;

    view! {
        <svg class="icon"
        style=format!("inline-size: {0}cap; block-size: {0}cap;", size)
        aria-hidden="true">
            <use href="/icons.svg#chess-pawn"></use>
        </svg>
    }
}
