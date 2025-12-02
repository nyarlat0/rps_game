use leptos::prelude::*;
use leptos_use::{core::ConnectionReadyState, use_timeout_fn, UseTimeoutFnReturn};
use shared::{
    auth::UserInfo,
    game::{GameError, GameResult},
    rps_game::{RpsGameReq, RpsGameState, RpsMove},
    ws_messages::{ClientMsg, ServerMsg},
};

use crate::hooks::{MyToaster, WebsocketContext};

fn mv_into_view(mv: RpsMove, size: usize) -> AnyView
{
    match mv {
        RpsMove::Rock => view! {
                             <svg class="icon"
                             style=format!("inline-size: {0}cap; block-size: {0}cap;", size)
                             aria-hidden="true">
                                 <use href="/icons.svg#hand-rock"></use>
                             </svg>
                         }.into_any(),
        RpsMove::Paper => view! {
                              <svg class="icon"
                              style=format!("inline-size: {0}cap; block-size: {0}cap;", size)
                              aria-hidden="true">
                                  <use href="/icons.svg#hand-paper"></use>
                              </svg>
                          }.into_any(),
        RpsMove::Scissors => view! {
                                 <svg class="icon"
                                 style=format!("inline-size: {0}cap; block-size: {0}cap;", size)
                                 aria-hidden="true">
                                     <use href="/icons.svg#hand-scissors"></use>
                                 </svg>
                             }.into_any(),
    }
}

#[component]
pub fn RpsGame() -> impl IntoView
{
    let ws = expect_context::<WebsocketContext>();
    let user_info = expect_context::<UserInfo>();

    let (curr_game, set_curr_game) = signal::<Option<RpsGameState>>(None);
    let (curr_mv, set_curr_mv) = signal::<Option<RpsMove>>(None);
    let (can_leave, set_can_leave) = signal(false);

    let toaster = MyToaster::new();

    let UseTimeoutFnReturn { start: timer_start, .. } = {
        let user_info = user_info.clone();
        use_timeout_fn(move |()| {
                           if let Some(RpsGameState::Game { players, submitted }) = curr_game.get()
                           {
                               let opp_sub = if players[0] == user_info.username {
                                   submitted[1]
                               } else {
                                   submitted[0]
                               };

                               if !opp_sub {
                                   set_can_leave.set(true);
                               }
                           }
                       },
                       20_000.0)
    };

    Effect::new({
        let ws = ws.clone();

        move |_| {
            if let Some(msg) = ws.message.get() {
                if let ServerMsg::RpsGameMsg(rps_state) = msg {
                    if matches!(rps_state, RpsGameState::Game { .. }) {
                        set_can_leave.set(false);
                        timer_start(()); // <<—— correct place
                    }
                    set_curr_game.set(Some(rps_state));
                } else if let ServerMsg::GameErrorMsg(GameError::Disconnected) = msg {
                    set_curr_game.set(None);
                    set_curr_mv.set(None);
                    toaster.error("Opponent disconnected!");
                    ws.send(ClientMsg::RpsGameMsg(RpsGameReq::Start));
                }
            };
        }
    });

    Effect::new({
        let ws = ws.clone();
        move |_| {
            if ws.state.get() == ConnectionReadyState::Open {
                ws.send(ClientMsg::RpsGameMsg(RpsGameReq::Start));
            }
        }
    });

    let leave_btn = {
        let ws = ws.clone();
        move |_| {
            if ws.state.get() == ConnectionReadyState::Open {
                ws.send(ClientMsg::RpsGameMsg(RpsGameReq::Leave));
                ws.send(ClientMsg::RpsGameMsg(RpsGameReq::Start));
                set_curr_game.set(None);
                set_curr_mv.set(None);
            }
        }
    };

    let next_btn = {
        let ws = ws.clone();
        move |_| {
            if ws.state.get() == ConnectionReadyState::Open {
                ws.send(ClientMsg::RpsGameMsg(RpsGameReq::Start));
                set_curr_game.set(None);
                set_curr_mv.set(None);
            }
        }
    };

    view! {
        <div class="stack fill-page card">

        <div class="cluster" style="--cluster-justify: center; --cluster-gap: 0;">
            <svg class="icon"
            style="inline-size: 4cap; block-size: 4cap;"
            aria-hidden="true">
                <use href="/icons.svg#hand-rock"></use>
            </svg>
            <h1>"–"</h1>
            <svg class="icon"
            style="inline-size: 4cap; block-size: 4cap;"
            aria-hidden="true">
                <use href="/icons.svg#hand-paper"></use>
            </svg>
            <h1>"–"</h1>
            <svg class="icon"
            style="inline-size: 4cap; block-size: 4cap;"
            aria-hidden="true">
                <use href="/icons.svg#hand-scissors"></use>
            </svg>
        </div>

        { move || {
            match curr_game.get() {
                None => {
                    set_can_leave.set(false);
                    view!{
                        <p>"Waiting for opponent.."</p>
                        <div class="loading-spinner" style="margin-top: auto; margin-bottom: auto;"></div>
                    }.into_any()
                },

                Some(RpsGameState::Game { players, submitted }) => {
                    let (opp_name, opp_sub, player_sub) = if players[0] == user_info.username {
                        (players[1].clone(), submitted[1], submitted[0])
                    } else {
                        (players[0].clone(), submitted[0], submitted[1])
                    };

                    let submit = {
                        let ws = ws.clone();
                        move |mv: RpsMove| {
                            ws.send(ClientMsg::RpsGameMsg(RpsGameReq::Submit(mv)));
                            set_curr_mv.set(Some(mv));
                        }
                    };

                    view!{
                        <h3>"Playing against: "<span class="mention-name">{opp_name}</span>
                        <span style="color: var(--muted);">
                        {move || {
                            if opp_sub {
                                " (moved)"
                            } else {
                                ""
                            }
                        }}
                        </span></h3>
                        <div
                        class="cluster"
                        class:el-hide=player_sub
                        style="--cluster-justify: center; margin-top: auto; margin-bottom: auto;"
                        >
                            <button
                            class="icon-btn"
                            on:click={
                                let submit = submit.clone();
                                move |_| {submit(RpsMove::Rock)}
                            }
                            >
                                <svg class="icon"
                                style="inline-size: 4cap; block-size: 4cap;"
                                aria-hidden="true">
                                    <use href="/icons.svg#hand-rock"></use>
                                </svg>
                            </button>
                            <button
                            class="icon-btn"
                            on:click={
                                let submit = submit.clone();
                                move |_| {submit(RpsMove::Paper)}
                            }
                            >
                                <svg class="icon"
                                style="inline-size: 4cap; block-size: 4cap;"
                                aria-hidden="true">
                                    <use href="/icons.svg#hand-paper"></use>
                                </svg>
                            </button>
                            <button
                            class="icon-btn"
                            on:click={
                                let submit = submit.clone();
                                move |_| {submit(RpsMove::Scissors)}
                            }
                            >
                                <svg class="icon"
                                style="inline-size: 4cap; block-size: 4cap;"
                                aria-hidden="true">
                                    <use href="/icons.svg#hand-scissors"></use>
                                </svg>
                            </button>
                        </div>
                        <p class:el-hide=!player_sub>
                            "You played: "{mv_into_view(curr_mv.get().unwrap_or(RpsMove::Rock), 3)}
                        </p>
                    }.into_any()
                }
                Some(RpsGameState::Finished(mut info)) => {
                    set_can_leave.set(false);

                    let (opp_name, opp_move, player_move, res) = if info.players[0] == user_info.username {
                        (info.players[1].clone(), info.moves[1], info.moves[0], info.resolve())
                    } else {
                        info.reverse();
                        (info.players[1].clone(), info.moves[1], info.moves[0], info.resolve())
                    };

                    view! {
                        <h3 style=match res {
                            GameResult::Win => "color: var(--success);",
                            GameResult::Defeat => "color: var(--error);",
                            GameResult::Draw => "",
                        }
                        >
                            "Game finished! "{res.to_string()}
                        </h3>
                        <p><span class="mention-name">{opp_name}</span>" played:  "{mv_into_view(opp_move, 3)}</p>
                        <p>"You played: "{mv_into_view(player_move, 3)}</p>
                    }.into_any()
                }
            }
        }}
            <div class="stack" style="margin-top: auto; --stack-gap: var(--s0);">
            <button
            class:el-hide=move || !curr_game.get().is_some_and(|g| matches!(g, RpsGameState::Finished{..}))
            on:click=next_btn>
                "Next game!"
            </button>
            <button
            class="secondary destructive"
            class:el-hide=move || {!can_leave.get()}
            on:click=leave_btn>
                "Leave"
            </button>
            <div class="cluster" style="--cluster-justify: center;">
            <a href = "/games" class="button secondary" style="width: 50%;">
                "Other games"
            </a>
            <a href = "/" class="button secondary" style="width: calc(50% - 1rem);">
                "Home"
            </a>
            </div>
            </div>

        </div>
    }
}
