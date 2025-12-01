use async_trait::async_trait;
use shared::game::{GameError, GameResult};
use sqlx::PgPool;

use crate::domain::{
    game_model::{FinishedGame, GameRecorder},
    rps_model::{FinishedRpsGame, RpsGame},
};

pub struct PsqlGameRecorder
{
    pub db: PgPool,
}

#[async_trait]
impl GameRecorder<RpsGame> for PsqlGameRecorder
{
    async fn record(&self, game: FinishedRpsGame) -> Result<(), GameError>
    {
        sqlx::query!(
                     r#"
            INSERT INTO rps_games
                (player1, player2, move1, move2, created_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
                     game.players_id[0],
                     game.players_id[1],
                     game.moves[0].to_string(),
                     game.moves[1].to_string(),
                     game.created_at,
        ).execute(&self.db)
         .await
         .map_err(|_e| GameError::DbError)?;

        match game.resolve() {
            GameResult::Win => {
                sqlx::query!(
                             r#"
            INSERT INTO rps_results (user_id, win_counter)
            VALUES ($1, 1)
            ON CONFLICT (user_id)
            DO UPDATE SET win_counter = rps_results.win_counter + 1
            "#,
                             game.players_id[0]
                ).execute(&self.db)
                 .await
                 .map_err(|_e| GameError::DbError)?;

                sqlx::query!(
                             r#"
            INSERT INTO rps_results (user_id, lose_counter)
            VALUES ($1, 1)
            ON CONFLICT (user_id)
            DO UPDATE SET lose_counter = rps_results.lose_counter + 1
            "#,
                             game.players_id[1]
                ).execute(&self.db)
                 .await
                 .map_err(|_e| GameError::DbError)?;
            }

            GameResult::Draw => {
                sqlx::query!(
                             r#"
            INSERT INTO rps_results (user_id, draw_counter)
            VALUES ($1, 1)
            ON CONFLICT (user_id)
            DO UPDATE SET draw_counter = rps_results.draw_counter + 1
            "#,
                             game.players_id[0]
                ).execute(&self.db)
                 .await
                 .map_err(|_e| GameError::DbError)?;

                sqlx::query!(
                             r#"
            INSERT INTO rps_results (user_id, draw_counter)
            VALUES ($1, 1)
            ON CONFLICT (user_id)
            DO UPDATE SET draw_counter = rps_results.draw_counter + 1
            "#,
                             game.players_id[1]
                ).execute(&self.db)
                 .await
                 .map_err(|_e| GameError::DbError)?;
            }
            GameResult::Defeat => {
                sqlx::query!(
                             r#"
            INSERT INTO rps_results (user_id, win_counter)
            VALUES ($1, 1)
            ON CONFLICT (user_id)
            DO UPDATE SET win_counter = rps_results.win_counter + 1
            "#,
                             game.players_id[1]
                ).execute(&self.db)
                 .await
                 .map_err(|_e| GameError::DbError)?;

                sqlx::query!(
                             r#"
            INSERT INTO rps_results (user_id, lose_counter)
            VALUES ($1, 1)
            ON CONFLICT (user_id)
            DO UPDATE SET lose_counter = rps_results.lose_counter + 1
            "#,
                             game.players_id[0]
                ).execute(&self.db)
                 .await
                 .map_err(|_e| GameError::DbError)?;
            }
        }

        Ok(())
    }
}
