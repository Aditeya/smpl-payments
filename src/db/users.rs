use chrono::Utc;
use diesel::{ExpressionMethods, Insertable, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;

use super::{handle_duplicate_error, models::User, schema::users, Error, SmplDB};

#[derive(Insertable)]
#[diesel(table_name = users)]
struct NewUser<'a> {
    pub username: &'a str,
    pub email: &'a str,
    pub password: &'a str,
    pub status: bool,
}

impl SmplDB {
    pub async fn sign_up_user(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<User, Error> {
        let user = NewUser {
            username,
            email,
            password,
            status: true,
        };

        let mut conn = self.get_conn().await?;
        diesel::insert_into(users::table)
            .values(&user)
            .returning(User::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(handle_duplicate_error)
    }

    pub async fn get_user(&self, email: &str) -> Result<Option<User>, Error> {
        let mut conn = self.get_conn().await?;
        users::table
            .select(User::as_select())
            .filter(users::email.eq(email))
            .first(&mut conn)
            .await
            .optional()
            .map_err(handle_duplicate_error)
    }

    pub async fn get_user_by_id(&self, id: i32) -> Result<Option<User>, Error> {
        let mut conn = self.get_conn().await?;
        users::table
            .select(User::as_select())
            .filter(users::id.eq(id))
            .first(&mut conn)
            .await
            .optional()
            .map_err(handle_duplicate_error)
    }

    pub async fn update_username(&self, id: i32, username: &str) -> Result<Option<User>, Error> {
        let mut conn = self.get_conn().await?;
        let now = Utc::now();
        diesel::update(users::table)
            .filter(users::id.eq(id))
            .set((
                users::username.eq(username),
                users::updated_at.eq(Some(now)),
            ))
            .returning(User::as_returning())
            .get_result(&mut conn)
            .await
            .optional()
            .map_err(handle_duplicate_error)
    }
}
