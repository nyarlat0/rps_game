use leptos::task;
use std::sync::Arc;

use chrono::{DateTime, Datelike, Local, Timelike, Utc};
use codee::string::FromToStringCodec;
use leptos::{
    html::{Div, Textarea},
    prelude::*,
    reactive::spawn_local,
};
use leptos_fluent::{tr, I18n};
use leptos_use::{storage::use_local_storage, use_scroll, UseScrollReturn};
use shared::{forum::UserForumPost, ws_messages::ServerMsg};

use crate::{
    api::{
        create_post, delete_post, dislike_post, fetch_posts, fetch_posts_by, like_post,
        undo_reaction,
    },
    hooks::{MyToaster, NavBarCtx, SettingsCtx, StateCtx, UserResCtx, WebsocketContext},
};

#[derive(Clone)]
struct UsernameCtx(Arc<String>);

fn has_mention(s: &str, username: &str) -> bool
{
    s.split('@')
     .skip(1) // parts after each '@'
     .any(|tail| tail.starts_with(username))
}

fn is_word_char(ch: char) -> bool
{
    ch.is_alphanumeric() || ch == '_'
}

fn render_mentions(text: &str) -> impl IntoView + use<>
{
    let mut out = Vec::new();
    let mut last_byte = 0;

    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut i = 0;

    while i < chars.len() {
        let (byte_idx, ch) = chars[i];

        if ch == '@' {
            let mut j = i + 1;

            if j < chars.len() && is_word_char(chars[j].1) {
                while j < chars.len() && is_word_char(chars[j].1) {
                    j += 1;
                }

                let mention_start = byte_idx;
                let mention_end = if j < chars.len() {
                    chars[j].0
                } else {
                    text.len()
                };

                if last_byte < mention_start {
                    out.push(text[last_byte..mention_start].to_string().into_any());
                }

                out.push(view! {
                             <span class="mention-name">
                                 { &text[mention_start..mention_end] }
                             </span>
                         }.into_any());

                last_byte = mention_end;
                i = j;
                continue;
            }
        }

        i += 1;
    }

    if last_byte < text.len() {
        out.push(text[last_byte..].to_string().into_any());
    }

    view! { <> {out} </> }
}

#[component]
pub fn Forum() -> impl IntoView
{
    let navctx = expect_context::<NavBarCtx>();
    let (visible_forum, _) = navctx.visible_forum;
    let (authed, _) = expect_context::<StateCtx>().authed;
    let UserResCtx(userinfo_res) = expect_context::<UserResCtx>();

    view! {
        <Show
            when=move || authed.get() && userinfo_res.get().flatten().is_some()
            fallback=move || view! {
                <div
                    class="card stack forum"
                    class:active=move || visible_forum.get()
                >
                    { move || tr!("forum-login-prompt") }
                </div>
            }
        >
            <ForumAuth />
        </Show>
    }
}

#[component]
fn ForumAuth() -> impl IntoView
{
    let navctx = expect_context::<NavBarCtx>();
    let (visible_forum, _) = navctx.visible_forum;

    let ws = expect_context::<WebsocketContext>();

    let UserResCtx(user_res) = expect_context::<UserResCtx>();
    let user_info = user_res.get_untracked().unwrap().unwrap();

    let username = Arc::new(user_info.username.clone());
    provide_context(UsernameCtx(username));

    let (last_seen, set_last_seen, _) =
        use_local_storage::<i64, FromToStringCodec>("last_seen_post");

    let (new_posts, set_new_posts) = navctx.new_posts;

    let toaster = MyToaster::new();

    let forum_res = LocalResource::new(move || fetch_posts());

    let (message, set_message) = signal(String::new());

    let on_submit = {
        let toaster = toaster.clone();
        move || {
            let msg = message.get().trim().to_string();
            if !msg.is_empty() {
                let toaster = toaster.clone();

                spawn_local(async move {
                    match create_post(msg).await {
                        Ok(_) => forum_res.refetch(),
                        Err(err) => {
                            let prefix = tr!("forum-send-error");
                            let toast = format!("{prefix} ({err:?})");
                            toaster.error(&toast);
                        }
                    }
                });
                set_message.set("".to_string());
            }
        }
    };

    let forum_elem = NodeRef::<Div>::new();
    let textarea_elem = NodeRef::<Textarea>::new();

    let UseScrollReturn { y, set_y, .. } = use_scroll(forum_elem);
    let (scrolled_once, set_scrolled_once) = signal(false);

    let (near_top, set_near_top) = signal(false);
    let (near_bottom, set_near_bottom) = signal(true);

    let (posts_sig, set_posts_sig) = signal(Vec::<UserForumPost>::new());

    // fill fetched posts in the signal and scroll to the bottom
    Effect::new(move |_| {
        if let Some(Some(posts)) = forum_res.get() {
            set_posts_sig.set(posts);
        }
    });

    // scroll to buttom when posts update
    Effect::new({
        let set_y = set_y.clone();

        move |_| {
            let has_posts = posts_sig.with(|v| !v.is_empty());
            let already = scrolled_once.get();

            if has_posts && !already {
                if let Some(el) = forum_elem.get() {
                    let bottom = (el.scroll_height() - el.client_height()) as f64;
                    set_y(bottom);
                    set_scrolled_once.set(true);
                }
            }

            let y_coord = y.get_untracked();
            if already {
                if let Some(el) = forum_elem.get() {
                    let bottom = (el.scroll_height() - el.client_height()) as f64;

                    if (bottom - y_coord) <= (5.0 * (bottom / 26.0)) {
                        set_y(bottom);
                    }
                }
            }
        }
    });

    // fetch more on scroll up
    let (loading, set_loading) = signal(false);
    Effect::new({
        let toaster = toaster.clone();
        let set_y = set_y.clone();

        move |_| {
            if loading.get_untracked() || !scrolled_once.get() {
                return;
            };
            let near = near_top.get();

            if let Some(UserForumPost { post, .. }) =
                posts_sig.with_untracked(|v| v.first().cloned())
            {
                if visible_forum.get_untracked() && near && post.id != 1 {
                    let post_id = post.id;
                    let toaster = toaster.clone();
                    let set_y = set_y.clone();
                    let y_coord = y.get_untracked();

                    let h_before = forum_elem.get_untracked().unwrap().scroll_height() as f64;

                    set_loading.set(true);
                    spawn_local(async move {
                        let end_id = post_id - 1;
                        let start_id = (end_id - 25).max(1);

                        match fetch_posts_by(start_id, end_id).await {
                            Some(prev_posts) => {
                                set_posts_sig.update(move |posts| {
                                                 posts.splice(0..0, prev_posts);
                                             });

                                task::tick().await;

                                if let Some(el) = forum_elem.get_untracked() {
                                    let h_after = el.scroll_height() as f64;
                                    set_y(y_coord + h_after - h_before);

                                    task::tick().await;

                                    set_loading.set(false);
                                }
                            }
                            None => {
                                let msg = tr!("forum-load-old-error");
                                toaster.error(&msg);
                            }
                        }
                    });
                }
            }
        }
    });

    // listen to new posts
    Effect::new(move |_| {
        if let Some(msg) = ws.message.get() {
            match msg {
                ServerMsg::NewPostMsg(post) => {
                    set_posts_sig.update(move |posts| {
                                     posts.push(UserForumPost { post,
                                                                liked: false,
                                                                disliked: false })
                                 });
                }
                _ => {}
            }
        };
    });

    // update last seen counter
    Effect::new(move |_| {
        let opt_id = posts_sig.with(|v| v.last().map(|up| up.post.id));
        let y_coord = y.get();

        if visible_forum.get() {
            if let Some(last_id) = opt_id {
                if let Some(el) = forum_elem.get_untracked() {
                    let text_el = textarea_elem.get().unwrap();
                    let bottom = (el.scroll_height()
                                  - el.client_height()
                                  - text_el.client_height()
                                  - text_el.scroll_height())
                                 as f64;
                    if (bottom - y_coord) <= 40.0 {
                        set_last_seen.update(|ls| *ls = (*ls).max(last_id));
                    }
                }
            }
        }
    });

    // set new posts marker
    Effect::new(move |_| {
        let ls = last_seen.get();
        let opt_id = posts_sig.with(|v| v.last().map(|up| up.post.id));

        if let Some(last_id) = opt_id {
            if last_id > ls {
                set_new_posts.set(true);
            } else if new_posts.get_untracked() {
                set_new_posts.set(false);
            }
        }
    });

    // track near bottom position
    Effect::new(move || {
        let y_coord = y.get();
        if let Some(el) = forum_elem.get_untracked() {
            let bottom = (el.scroll_height() - el.client_height()) as f64;
            let thresh = 2.0 * (bottom / 26.0);

            let is_near = (bottom - y_coord) <= thresh;

            if is_near != near_bottom.get_untracked() {
                set_near_bottom.set(is_near);
            }
        }
    });

    // track near top position
    Effect::new({
        move |_| {
            let y_px = y.get(); // tracked on scroll
            let is_near = y_px <= 64.0;
            if near_top.get_untracked() != is_near {
                set_near_top.set(is_near);
            }
        }
    });

    let (is_admin, _) = expect_context::<SettingsCtx>().admin_control;

    view! {
        <button
            class="forum-reload-btn icon-btn navbar-icon"
            class:active=move || visible_forum.get()
            on:click=move |_| forum_res.refetch()
        >
            <svg class="icon forum-reaction-icon" aria-hidden="true">
                <use href="/icons.svg#rotate-ccw"></use>
            </svg>
        </button>
        <button
            class="forum-scroll-btn icon-btn navbar-icon"
            class:active=move || visible_forum.get()
            style=move || if near_bottom.get() {
                "display: none;"
            } else {
                ""
            }
            on:click=move |_| {
                if let Some(el) = forum_elem.get() {
                    let bottom = (el.scroll_height() - el.client_height()) as f64;
                    set_y(bottom);
                    set_near_bottom.set(true);
                }
            }
        >
            <svg class="icon forum-scroll-icon" aria-hidden="true">
                <use href="/icons.svg#arrow-down"></use>
            </svg>
        </button>
        <div
            class="card stack forum"
            class:active=move || visible_forum.get()
            node_ref=forum_elem
        >
            <h3>{ move || tr!("forum-title") }</h3>
            <div
                class:loading-spinner=move || loading.get()
                style=move || if loading.get() {
                    ""
                } else {
                    "display: none;"
                }
            ></div>
            <hr />
            <For
                each=move || posts_sig.get()
                key=|upost| (upost.post.id, upost.post.likes, upost.post.dislikes)
                children=move |upost| {
                    let toaster = toaster.clone();
                        view!{
                            <PostRow
                                upost
                                is_admin
                                on_author=Callback::new(move |s: String| {

                                    set_message.update(|mes| {
                                        mes.push_str(&format!("@{}, ", s))
                                    });

                                    if let Some(el) = textarea_elem.get() {
                                        el.focus().ok();
                                    }
                                })
                                on_error=Callback::new(move |s: String| {
                                    let prefix = tr!("forum-action-error");
                                    let msg = format!("{prefix} ({s})");
                                    toaster.error(&msg);
                                })
                                on_refetch=Callback::new(move |_| forum_res.refetch())
                            />
                        }
                }
            />
            <textarea
                name="message"
                placeholder=move || tr!("forum-placeholder")
                node_ref=textarea_elem
                prop:value=move || message.get()
                on:input=move |ev| set_message.set(event_target_value(&ev))
                on:keydown={
                    let on_submit = on_submit.clone();
                    move |ev: web_sys::KeyboardEvent| {
                        if ev.key() == "Enter" && !ev.shift_key() {
                            ev.prevent_default();
                            on_submit();
                        }
                    }
                }
                />
            <div class="cluster" style="--cluster-justify: space-between;">
                <button
                class="tetriary destructive"
                on:click=move |_| set_message.set("".to_string())
                >
                    { move || tr!("forum-clear") }
                </button>
                <button
                style="width: 50%;"
                on:click=move |_| on_submit()
                >
                    { move || tr!("forum-send") }
                </button>
            </div>
        </div>
    }.into_any()
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Reaction
{
    None,
    Like,
    Dislike,
}

#[component]
fn PostRow(upost: UserForumPost,
           is_admin: ReadSignal<bool>,
           on_author: Callback<String>,
           on_error: Callback<String>,
           on_refetch: Callback<()>)
           -> impl IntoView
{
    let i18n = expect_context::<I18n>();

    let post = upost.post;
    let UsernameCtx(username) = expect_context::<UsernameCtx>();

    // derive initial reaction
    let init = if upost.liked {
        Reaction::Like
    } else if upost.disliked {
        Reaction::Dislike
    } else {
        Reaction::None
    };

    let (reaction, set_reaction) = signal(init);
    let (likes, set_likes) = signal(post.likes);
    let (dislikes, set_dislikes) = signal(post.dislikes);

    // optimistic toggle helper
    let toggle = move |target: Reaction| {
        let prev = reaction.get();
        let next = if prev == target {
            Reaction::None
        } else {
            target
        };

        // optimistic counters
        match (prev, next) {
            (Reaction::Like, Reaction::None) => set_likes.update(|n| *n -= 1),
            (Reaction::Dislike, Reaction::None) => set_dislikes.update(|n| *n -= 1),
            (Reaction::None, Reaction::Like) => set_likes.update(|n| *n += 1),
            (Reaction::None, Reaction::Dislike) => set_dislikes.update(|n| *n += 1),
            (Reaction::Dislike, Reaction::Like) => {
                set_dislikes.update(|n| *n -= 1);
                set_likes.update(|n| *n += 1);
            }
            (Reaction::Like, Reaction::Dislike) => {
                set_likes.update(|n| *n -= 1);
                set_dislikes.update(|n| *n += 1);
            }
            _ => {}
        }
        set_reaction.set(next);

        let pid = post.id;
        let on_error = on_error.clone();

        spawn_local(async move {
            let res = match next {
                Reaction::Like => like_post(pid).await,
                Reaction::Dislike => dislike_post(pid).await,
                Reaction::None => undo_reaction(pid).await,
            };
            if let Err(e) = res {
                on_error.run(format!("{e:?}"));
            }
        });
    };

    let delete_btn = move || {
        let pid = post.id;
        let on_error = on_error.clone();

        spawn_local(async move {
            let res = delete_post(pid).await;
            if let Err(e) = res {
                on_error.run(format!("{e:?}"));
            } else {
                on_refetch.run(());
            }
        });
    };

    let like_active = move || reaction.get() == Reaction::Like;
    let dislike_active = move || reaction.get() == Reaction::Dislike;

    view! {
        <div class="cluster" style="--cluster-justify: space-between;">
            <div class="cluster">
                <p style="font-weight:700;">{post.id}</p>
                <button
                class="forum-author-btn"
                on:click=move |_| {on_author.run(post.author.clone())}
                >
                    {post.author.clone()}
                </button>
            </div>
            <p style="color:var(--muted);">
                {move || display_time(post.created_at, &i18n.language.get().id.to_string())}
            </p>
        </div>

        <div
        class="stack post-contents"
        class:mention=has_mention(&post.contents, &username)
        style="--stack-gap: var(--s-2);"
        >
            <p>{render_mentions(&post.contents)}</p>

            <div class="cluster" style="--cluster-justify: space-between; --cluster-align: baseline;">
            <div class="cluster">
                <button
                    class="forum-reaction-btn icon-btn navbar-icon"
                    style=move || if like_active() {
                        "--hover-color:var(--success); color:var(--success); align-items:baseline;"
                    } else {
                        "--hover-color:var(--success); align-items:baseline;"
                    }
                    on:click=move |_| toggle(Reaction::Like)
                >
                    <svg class="icon forum-reaction-icon" aria-hidden="true">
                        <use href="/icons.svg#thumbs-up"></use>
                    </svg>
                    {move || likes.get()}
                </button>

                <button
                    class="forum-reaction-btn icon-btn navbar-icon"
                    style=move || if dislike_active() {
                        "--hover-color:var(--error); color:var(--error); align-items:baseline;"
                    } else {
                        "--hover-color:var(--error); align-items:baseline;"
                    }
                    on:click=move |_| toggle(Reaction::Dislike)
                >
                    <svg class="icon forum-reaction-icon" aria-hidden="true">
                        <use href="/icons.svg#thumbs-down"></use>
                    </svg>
                    {move || dislikes.get()}
                </button>
            </div>
            <button
            class="forum-reaction-btn icon-btn navbar-icon"
            style=move || if is_admin.get() {
                "--hover-color:var(--error);"
            } else {
                "display: none;"
            }
            on:click=move |_| {delete_btn()}
            >
                <svg class="icon forum-reaction-icon" aria-hidden="true">
                        <use href="/icons.svg#trash-2"></use>
                </svg>
            </button>

            </div>
        </div>
        <hr/>
    }
}

fn display_time(dt_utc: DateTime<Utc>, lang_code: &str) -> String
{
    let dt = dt_utc.with_timezone(&Local);
    let now = Local::now();

    const MONTHS_EN: [&str; 12] =
        ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    const MONTHS_RU: [&str; 12] =
        ["янв", "фев", "мар", "апр", "май", "июн", "июл", "авг", "сен", "окт", "ноя", "дек"];

    let months = if lang_code.starts_with("ru") {
        &MONTHS_RU
    } else {
        &MONTHS_EN
    };

    let h = dt.hour();
    let m = dt.minute();

    if dt.date_naive() == now.date_naive() {
        format!("{:02}:{:02}", h, m)
    } else if dt.year() == now.year() {
        let month = months[(dt.month() - 1) as usize];
        format!("{} {} {:02}:{:02}", dt.day(), month, h, m)
    } else {
        format!("{:02}.{:02}.{:02} {:02}:{:02}",
                dt.day(),
                dt.month(),
                dt.year() % 100,
                h,
                m)
    }
}
