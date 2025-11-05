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
        create_post, dislike_post, fetch_posts, like_post,
        undo_reaction,
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
        move |_| {
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
    let UseScrollReturn { set_y, .. } =
        use_scroll(forum_elem);

    view! {
        <div
            class="card stack forum"
            class:active=move || visible_forum.get()
            node_ref=forum_elem
        >
            <h3>"Forum"</h3>
            {
                move || match forum_res.get() {
                    Some(Some(posts)) => {
                        let on_submit = on_submit.clone();
                        let toaster = toaster.clone();

                        let (posts_sig, set_posts_sig) = signal(posts);

                        // scroll to buttom when posts update
                        Effect::new({
                            let set_y = set_y.clone();

                            move |_| {
                                let _ = posts_sig.get();

                                if let Some(el) = forum_elem.get() {
                                    let bottom = el.scroll_height() as f64;
                                    set_y(bottom);
                                }
                            }
                        });

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
                            <hr />
                            <For
                                each=move || posts_sig.get()
                                key=|upost| upost.post.id
                                children=move |upost| {
                                    let (like, set_like) = signal(upost.liked);
                                    let (dislike, set_dislike) = signal(upost.disliked);

                                    let post = upost.post;
                                    let (like_counter, set_like_counter) = signal(post.likes);
                                    let (dislike_counter, set_dislike_counter) = signal(post.dislikes);

                                    let post_id = post.id;

                                    let (first_run, set_first_run) = signal(true);

                                    Effect::new({
                                        let toaster = toaster.clone();

                                        move || {
                                            let toaster = toaster.clone();

                                            if first_run.get(){
                                                set_first_run.set(false);
                                                return;
                                            }

                                            if like.get(){
                                                spawn_local(async move {
                                                    match like_post(post_id).await {
                                                        Ok(_) => {},
                                                        Err(err) => {
                                                            toaster.error(&format!("{:?}", err))
                                                        }
                                                    }
                                                });
                                            } else if dislike.get(){
                                                spawn_local(async move {
                                                    match dislike_post(post_id).await {
                                                        Ok(_) => {},
                                                        Err(err) => {
                                                            toaster.error(&format!("{:?}", err))
                                                        }
                                                    }
                                                });
                                            };
                                        }
                                    });

                                    view! {
                                        <div
                                            class="cluster"
                                            style="--cluster-justify: space-between;"
                                        >
                                        <div class="cluster">
                                            <p style="font-weight: 700;">{post.id}</p>
                                            <p style="font-weight: 700; color: var(--success);">
                                                {post.author}
                                            </p>
                                        </div>
                                        <p style="color: var(--muted);">{post.created_at
                                            .with_timezone(&Local)
                                            .format("%d.%m.%Y %H:%M").to_string()
                                        }</p>
                                        </div>
                                        <div
                                        class="stack post-contents"
                                        style="--stack-gap: var(--s-2);"
                                        >
                                        <p
                                        class="post-contents"
                                        >
                                            {post.contents}
                                        </p>
                                        <div
                                        class="cluster"
                                        >
                                            <button
                                                class="icon-btn"
                                                title="Toggle like"
                                                aria-label="Toggle like"
                                                style=move || {
                                                    if like.get() {
                                                        "--hover-color:var(--success);
                                                        align-items: baseline;
                                                        color: var(--success);"
                                                    } else {
                                                        "--hover-color:var(--success);
                                                        align-items: baseline;"
                                                    }
                                                }
                                                on:click={
                                                    let toaster = toaster.clone();
                                                    move |_| {
                                                        set_like.update(|value| *value = !*value);

                                                        if like.get() {
                                                            set_like_counter.update(|n| *n += 1);
                                                        } else {
                                                            set_like_counter.update(|n| *n -= 1);
                                                        }

                                                        if dislike.get() && like.get() {
                                                            set_dislike.set(false);
                                                            set_dislike_counter.update(|n| *n -= 1);
                                                        };

                                                        if !dislike.get() && !like.get() {
                                                            let toaster = toaster.clone();
                                                            spawn_local(async move {
                                                                match undo_reaction(post_id).await {
                                                                    Ok(_) => {},
                                                                    Err(err) => {
                                                                        toaster.error(&format!("{:?}", err))
                                                                    }
                                                                }
                                                            });
                                                        }
                                                    }
                                                }
                                            >
                                                <svg class="icon forum-reaction" alt="Toggle like">
                                                    <use href="icons.svg#thumbs-up"></use>
                                                </svg>
                                                {like_counter}
                                            </button>
                                            <button
                                                class="icon-btn"
                                                title="Toggle dislike"
                                                aria-label="Toggle dislike"
                                                style=move || {
                                                    if dislike.get() {
                                                        "--hover-color:var(--error);
                                                        align-items: baseline;
                                                        color: var(--error);"
                                                    } else {
                                                        "--hover-color:var(--error);
                                                        align-items: baseline;"
                                                    }
                                                }
                                                on:click={
                                                    let toaster = toaster.clone();
                                                    move |_| {
                                                        set_dislike.update(|value| *value = !*value);

                                                        if dislike.get() {
                                                            set_dislike_counter.update(|n| *n += 1);
                                                        } else {
                                                            set_dislike_counter.update(|n| *n -= 1);
                                                        }

                                                        if dislike.get() && like.get() {
                                                            set_like.set(false);
                                                            set_like_counter.update(|n| *n -= 1);
                                                        };

                                                        if !dislike.get() && !like.get() {
                                                            let toaster = toaster.clone();
                                                            spawn_local(async move {
                                                                match undo_reaction(post_id).await {
                                                                    Ok(_) => {},
                                                                    Err(err) => {
                                                                        toaster.error(&format!("{:?}", err))
                                                                    }
                                                                }
                                                            });
                                                        }
                                                    }
                                                }
                                            >
                                                <svg class="icon forum-reaction" alt="Toggle dislike">
                                                    <use href="icons.svg#thumbs-down"></use>
                                                </svg>
                                                {dislike_counter}
                                            </button>
                                        </div>
                                        </div>
                                        <hr />
                                    }
                                }
                            />
                            <textarea
                                name="message"
                                placeholder="Write your message..."
                                prop:value=move || message.get()
                                on:input=move |ev| set_message.set(event_target_value(&ev))
                                />
                            <div class="cluster" style="--cluster-justify: space-between;">
                                <button class="tetriary destructive">"Clear"</button>
                                <button on:click=on_submit style="width: 50%;">"Send"</button>
                            </div>
                        }.into_any()
                    }
        _ => view! {<div class="loading-spinner"></div>}.into_any(),
    }
        }
        </div>
    }.into_any()
}
