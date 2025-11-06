use chrono::Local;
use leptos::{
    html::Div, prelude::*, reactive::spawn_local,
};
use leptos_use::{use_scroll, UseScrollReturn};
use shared::{
    forum::UserForumPost, ws_messages::ServerMsg,
};

use crate::{
    api::{
        create_post, dislike_post, fetch_posts,
        fetch_posts_by, like_post, undo_reaction,
    },
    hooks::{MyToaster, WebsocketContext},
};

#[component]
pub fn Forum(visible_forum: ReadSignal<bool>)
             -> impl IntoView
{
    let ws = {
        match use_context::<WebsocketContext>() {
            Some(ws) => ws,
            None => return view! {
                <div
                    class="card stack forum"
                    class:active=move || visible_forum.get()
                >
                    "Log in to see forum!"
                </div>
            }.into_any(),
        }
    };

    let toaster = MyToaster::new();

    let forum_res =
        LocalResource::new(move || fetch_posts());

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
                            toaster.error(&format!("{:?}",
                                                   err))
                        }
                    }
                });
                set_message.set("".to_string());
            }
        }
    };

    let forum_elem = NodeRef::<Div>::new();
    let UseScrollReturn { y, set_y, .. } =
        use_scroll(forum_elem);
    let (scrolled_once, set_scrolled_once) = signal(false);

    let (posts_sig, set_posts_sig) =
        signal(Vec::<UserForumPost>::new());

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
            let has_posts =
                posts_sig.with(|v| !v.is_empty());
            let already = scrolled_once.get();

            if has_posts && !already {
                if let Some(el) = forum_elem.get() {
                    let bottom = el.scroll_height() as f64;
                    set_y(bottom);
                    set_scrolled_once.set(true);
                }
            }

            let y_coord = y.get_untracked();
            if already {
                if let Some(el) = forum_elem.get() {
                    let bottom = (el.scroll_height()
                                  - el.client_height())
                                 as f64;

                    if (bottom - y_coord)
                       <= (5.0 * (bottom / 26.0))
                    {
                        set_y(bottom);
                    }
                }
            }
        }
    });

    // fetch more on scroll up
    Effect::new({
        let toaster = toaster.clone();
        move |_| {
            let y_coord = y.get();
            if let Some(UserForumPost { post, .. }) =
                posts_sig.with(|v| v.first().cloned())
            {
                if visible_forum.get()
                   && y_coord <= 64.0
                   && post.id != 1
                {
                    let post_id = post.id;
                    let toaster = toaster.clone();

                    spawn_local(async move {
                        let end_id = post_id - 1;
                        let start_id = (end_id - 25).max(1);

                        match fetch_posts_by(start_id, end_id).await {
                        Some(prev_posts) => {
                            set_posts_sig.update(move |posts| {
                                posts.splice(0..0, prev_posts);
                            });
                        }
                        None => {
                            toaster.error("Could not load old posts.")
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
                    set_posts_sig.update(move |posts| posts.push(
                            UserForumPost{
                                post, liked: false, disliked: false
                            }
                    ));
                }
                _ => {}
            }
        };
    });

    view! {
        <div
            class="card stack forum"
            class:active=move || visible_forum.get()
            node_ref=forum_elem
        >
            <h3>"Forum"</h3>
            <hr />
            <For
                each=move || posts_sig.get()
                key=|upost| (upost.post.id, upost.post.likes, upost.post.dislikes)
                children=move |upost| {
                    let toaster = toaster.clone();
                        view!{
                            <PostRow upost on_error=Callback::new(move |s: String| toaster.error(&s))/>
                        }
                }
            />
            <textarea
                name="message"
                placeholder="Write your message..."
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
                    "Clear"
                </button>
                <button
                style="width: 50%;"
                on:click=move |_| on_submit()
                >
                    "Send"
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
           on_error: Callback<String>)
           -> impl IntoView
{
    let post = upost.post;

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
            (Reaction::Like, Reaction::None) => {
                set_likes.update(|n| *n -= 1)
            }
            (Reaction::Dislike, Reaction::None) => {
                set_dislikes.update(|n| *n -= 1)
            }
            (Reaction::None, Reaction::Like) => {
                set_likes.update(|n| *n += 1)
            }
            (Reaction::None, Reaction::Dislike) => {
                set_dislikes.update(|n| *n += 1)
            }
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
                Reaction::Dislike => {
                    dislike_post(pid).await
                }
                Reaction::None => undo_reaction(pid).await,
            };
            if let Err(e) = res {
                on_error.run(format!("{e:?}"));
            }
        });
    };

    let like_active =
        move || reaction.get() == Reaction::Like;
    let dislike_active =
        move || reaction.get() == Reaction::Dislike;

    view! {
        <div class="cluster" style="--cluster-justify: space-between;">
            <div class="cluster">
                <p style="font-weight:700;">{post.id}</p>
                <p style="font-weight:700; color:var(--success);">{post.author.clone()}</p>
            </div>
            <p style="color:var(--muted);">
                {post.created_at.with_timezone(&Local).format("%d.%m.%Y %H:%M").to_string()}
            </p>
        </div>

        <div class="stack post-contents" style="--stack-gap: var(--s-2);">
            <p class="post-contents">{post.contents.clone()}</p>

            <div class="cluster">
                <button
                    class="icon-btn"
                    style=move || if like_active() {
                        "--hover-color:var(--success); color:var(--success); align-items:baseline;"
                    } else {
                        "--hover-color:var(--success); align-items:baseline;"
                    }
                    on:click=move |_| toggle(Reaction::Like)
                >
                    <svg class="icon forum-reaction" aria-hidden="true">
                        <use href="icons.svg#thumbs-up"></use>
                    </svg>
                    {move || likes.get()}
                </button>

                <button
                    class="icon-btn"
                    style=move || if dislike_active() {
                        "--hover-color:var(--error); color:var(--error); align-items:baseline;"
                    } else {
                        "--hover-color:var(--error); align-items:baseline;"
                    }
                    on:click=move |_| toggle(Reaction::Dislike)
                >
                    <svg class="icon forum-reaction" aria-hidden="true">
                        <use href="icons.svg#thumbs-down"></use>
                    </svg>
                    {move || dislikes.get()}
                </button>
            </div>
        </div>
        <hr/>
    }
}
