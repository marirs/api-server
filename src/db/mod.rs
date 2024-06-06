use crate::{
    error::Error,
    models::{NewUser, UpdateUser, User, UserInfo},
    secure::{
        hash_pass::{hash_password, verify_password},
        token::generate_api_string,
    },
};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Pool, Sqlite,
};
use std::{
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

/// Sqlite Backend struct
#[derive(Clone)]
pub struct SqliteBackend {
    /// # Database Connection Pool
    ///
    /// The `pool` module provides a connection pool for SQLite databases.
    pool: Pool<Sqlite>,
}

impl SqliteBackend {
    /// Creates a new connection to a SQLite database.
    ///
    /// # Arguments
    ///
    /// * `db_path` - A string slice containing the path to the database file.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the `Connection` object if successful, or an `Error` if an error occurred.
    pub async fn new_connection(db_path: &str) -> Result<Self, Error> {
        let conn = SqlitePoolOptions::new()
            .connect_with(
                SqliteConnectOptions::from_str(&("sqlite://".to_owned() + db_path))?
                    .create_if_missing(true),
            )
            .await?;

        Ok(Self { pool: conn })
    }

    #[inline]
    pub(crate) async fn check_and_create_table(&self) -> Result<(), Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                user_id TEXT PRIMARY KEY,
                password TEXT NOT NULL,
                api_key TEXT NOT NULL,
                email_id TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool.to_owned())
        .await?;

        Ok(())
    }

    /// Retrieves a user based on the provided API key.
    ///
    /// # Arguments
    ///
    /// - `api_key`: A string representing the API key to be used for the lookup.
    ///
    /// # Returns
    ///
    /// - If the user with the given API key is found, `Ok(UserInfo)` is returned.
    /// - If no user with the given API key is found, `Err(Error::NotFound)` is returned.
    /// - If an error occurs during the database operation, that error is returned as `Err(Error)`.
    pub async fn get_user_with_apikey(&self, api_key: &str) -> Result<UserInfo, Error> {
        sqlx::query_as::<_, UserInfo>(
            r#"
        SELECT * FROM users WHERE api_key = ?
        "#,
        )
        .bind(api_key.to_owned())
        .fetch_optional(&self.pool.to_owned())
        .await?
        .ok_or(Error::NotFound("User".to_string()))
    }

    /// Retrieve a user with the specified user ID from the database.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The ID of the user to retrieve.
    ///
    /// # Returns
    ///
    /// A `Result` containing:
    ///
    /// * `Ok(User)` - If a user with the specified ID is found in the database.
    /// * `Err(Error::NotFound)` - If no user with the specified ID is found in the database.
    pub async fn get_user_with_id(&self, user_id: &str) -> Result<UserInfo, Error> {
        sqlx::query_as::<_, UserInfo>(
            r#"
        SELECT * FROM users WHERE user_id = ?
        "#,
        )
        .bind(user_id.to_owned())
        .fetch_optional(&self.pool.to_owned())
        .await?
        .ok_or(Error::NotFound("User".to_string()))
    }

    /// Creates a new user with the provided information.
    ///
    /// # Arguments
    ///
    /// * `user` - A `NewUser` struct containing the details of the new user.
    ///
    /// # Returns
    ///
    /// An `Ok` result containing the newly created `User` if successful, or an `Err` containing
    /// an `Error` if there was a problem.
    pub async fn create_user(&self, user: NewUser, salt: &str) -> Result<User, Error> {
        let apikey = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos();

        let password = hash_password(&user.password, salt)?;
        if let Ok(verified) = verify_password(&user.password, &password.to_owned()) {
            if !verified {
                return Err(Error::PasswordHashError);
            }
        } else {
            return Err(Error::PasswordHashError);
        }

        let new_user = User {
            user_id: user.user_id,
            password,
            api_key: generate_api_string(&apikey.to_string()),
            email_id: user.email_id,
        };

        sqlx::query(
            r#"
            INSERT INTO users (user_id, password, api_key, email_id) VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(new_user.user_id.to_owned())
        .bind(new_user.password.to_owned())
        .bind(new_user.api_key.to_owned())
        .bind(new_user.email_id.to_owned())
        .execute(&self.pool.to_owned())
        .await?;

        Ok(new_user)
    }

    /// Updates a user's password or API key in the database.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The ID of the user to update.
    /// * `user` - An instance of `UpdateUser` struct that contains the user's updated data.
    ///
    /// # Returns
    ///
    /// * `Result<(), Error>` - An empty Ok result if the user was successfully updated, or an Error if any issues occurred.
    pub async fn update_user(
        &self,
        user: UpdateUser,
        user_id: &str,
        salt: &str,
    ) -> Result<(), Error> {
        // if both password and api_key are None, return BadRequest
        if user.api_key.is_none() && user.password.is_none() {
            return Err(Error::BadRequest("No data to update".to_string()));
        }

        // User found with the given user_id
        // Lets check and update either password or api_key
        if self.get_user_with_id(user_id).await.is_ok() {
            // update password if present
            if let Some(pass) = user.password.clone() {
                let password = hash_password(&pass, salt)?;
                if let Ok(verified) = verify_password(&pass, &password.to_owned()) {
                    if !verified {
                        return Err(Error::PasswordHashError);
                    }
                } else {
                    return Err(Error::PasswordHashError);
                }

                sqlx::query("UPDATE users SET password = ? WHERE user_id = ?")
                    .bind(pass)
                    .bind(user_id)
                    .execute(&self.pool.to_owned())
                    .await?;
            }
            // update api_key if present
            if let Some(api_key) = user.api_key.clone() {
                sqlx::query("UPDATE users SET api_key = ? WHERE user_id = ?")
                    .bind(api_key)
                    .bind(user_id)
                    .execute(&self.pool.to_owned())
                    .await?;
            }
        } else {
            // No user found with the given user_id
            return Err(Error::NotFound("User".to_string()));
        }
        Ok(())
    }

    /// Delete a user by their ID from the database.
    ///
    /// # Arguments
    ///
    /// - `pool`: A reference to the database connection pool.
    /// - `user_id`: The ID of the user to delete.
    ///
    /// # Returns
    ///
    /// This function returns `Result<(), Error>`, where `Error` represents any error that might occur.
    /// If the user with the given ID is successfully deleted, `Ok(())` is returned.
    /// If the user with the given ID is not found, `Err(Error::NotFound)` is returned.
    pub async fn delete_user(&self, user_id: &str) -> Result<(), Error> {
        sqlx::query("DELETE FROM users WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.pool.to_owned())
            .await
            .map_err(Into::into)
            .and_then(|res| {
                if res.rows_affected() > 0 {
                    Ok(())
                } else {
                    Err(Error::NotFound("User".to_string()))
                }
            })
    }

    /// Retrieves information for all users from the database.
    ///
    /// # Returns
    ///
    /// Returns a `Vec<UserInfo>` containing information for all users.
    ///
    /// # Errors
    ///
    /// An error is returned if there is any issue with the database query or fetching the data.
    pub async fn get_all_users(&self) -> Result<Vec<UserInfo>, Error> {
        sqlx::query_as::<_, UserInfo>(r#"SELECT * FROM users"#)
            .fetch_all(&self.pool.to_owned())
            .await
            .map_err(Into::into)
    }
}
