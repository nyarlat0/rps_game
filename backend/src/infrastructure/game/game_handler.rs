use actix::prelude::*;
use actix_web::web;
use shared::{
    rps_game::{RpsGameInfo, RpsGameState, RpsMove},
    ws_messages::ServerMsg,
};
use uuid::Uuid;

use crate::{
    application::auth_handler::AuthHandler,
    domain::{
        game_model::GameRepository,
        rps_model::FinishedRpsGame,
        rps_model::RpsGame,
        users_actor::{self, UsersActor},
    },
    infrastructure::game::{
        games_actor::{self, GamesActor},
        players_actor::{self, PlayersQueueActor},
    },
};

#[derive(Clone)]
pub struct RpsGameHandler
{
    pub player_qu: Addr<PlayersQueueActor>,
    pub games_actor: Addr<GamesActor>,
    pub users_actor: Addr<UsersActor>,
    pub auth_handler: web::Data<AuthHandler>,
}

impl RpsGameHandler
{
    pub async fn start(&self, user_id: Uuid)
    {
        if !self.games_actor
                .send(games_actor::Contains { user_id })
                .await
                .unwrap()
        {
            if let Some(opp_id) = self.player_qu
                                      .send(players_actor::Join { user_id })
                                      .await
                                      .unwrap()
            {
                let game = RpsGame::new(user_id, opp_id);

                let game_id = self.games_actor
                                  .send(games_actor::Create { game })
                                  .await
                                  .unwrap() as u32;
                let username = self.auth_handler
                                   .auth_service
                                   .get_userinfo(user_id)
                                   .await
                                   .unwrap()
                                   .username;

                let opponent = self.auth_handler
                                   .auth_service
                                   .get_userinfo(opp_id)
                                   .await
                                   .unwrap()
                                   .username;
                let user_msg = ServerMsg::RpsGameMsg(RpsGameState::Matched { game_id, opponent });
                let opp_msg = ServerMsg::RpsGameMsg(RpsGameState::Matched { game_id,
                                                                            opponent: username });

                self.users_actor.do_send(users_actor::SendToUser { user_id,
                                                                   msg: user_msg });
                self.users_actor
                    .do_send(users_actor::SendToUser { user_id: opp_id,
                                                       msg: opp_msg });
            } else {
                let msg = ServerMsg::RpsGameMsg(RpsGameState::Waiting);
                self.users_actor
                    .do_send(users_actor::SendToUser { user_id, msg });
            }
        }
    }

    pub async fn submit(&self, user_id: Uuid, game_id: usize, mv: RpsMove)
    {
        if let Some(fin_game) = self.games_actor
                                    .send(games_actor::Submit { user_id,
                                                                game_id,
                                                                mv })
                                    .await
                                    .unwrap()
        {
            let username = self.auth_handler.auth_service.get_userinfo(user_id);

            let opp_id = if fin_game.players_id[0] == user_id {
                fin_game.players_id[1]
            } else {
                fin_game.players_id[0]
            };
            let opponent = self.auth_handler
                               .auth_service
                               .get_userinfo(opp_id)
                               .await
                               .unwrap()
                               .username;
            let game_info = RpsGameInfo { players: [username, opponent],
                                          moves: fin_game.moves };
            let msg = ServerMsg::RpsGameMsg(RpsGameState::Finished(game_info));

            self.users_actor
                .do_send(users_actor::SendToUser { user_id, msg });
            self.users_actor
                .do_send(users_actor::SendToUser { user_id: opp_id,
                                                   msg });
        }
    }
}
