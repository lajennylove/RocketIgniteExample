//! The Users resource: the panel's list, form and actions for the `users` table.
//!
//! Written by hand rather than derived, because a password must be hashed
//! before it is stored, and the derive cannot do that.

use chrono::NaiveDateTime;
use diesel::dsl::sql;
use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
use diesel::result::QueryResult;
use diesel::sql_types::Bool;
use ignite_core::{
    escape_like, Column, Direction, Field, FieldErrors, FieldKind, FormInput, IgniteResource,
    ListQuery, Page, StatWidget,
};
use ignite_panel::users::users;
use serde::Serialize;

/// The shortest password the form accepts. Matches the Ignite CLI.
const MIN_PASSWORD_CHARS: usize = 8;

const FIELDS: &[Field] = &[
    Field {
        name: "name",
        label: "Name",
        kind: FieldKind::Text,
        required: true,
    },
    Field {
        name: "email",
        label: "Email",
        kind: FieldKind::Text,
        required: true,
    },
    // Required on edit too: the form never shows the stored hash, so an edit
    // sets a new password. See README.md.
    Field {
        name: "password",
        label: "Password",
        kind: FieldKind::Password,
        required: true,
    },
];

const COLUMNS: &[Column] = &[
    Column {
        key: "name",
        label: "Name",
        sortable: true,
        searchable: true,
    },
    Column {
        key: "email",
        label: "Email",
        sortable: true,
        searchable: true,
    },
    Column {
        key: "created_at",
        label: "Joined",
        sortable: true,
        searchable: false,
    },
];

/// The Users resource. Mounted at `/admin/users`.
pub struct Users;

/// One row of the list and the edit page. The password hash is never selected.
#[derive(Debug, Queryable, Selectable, Serialize)]
#[diesel(table_name = users)]
pub struct UserRecord {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub created_at: NaiveDateTime,
}

/// Validated form values for create and edit.
pub struct UserInput {
    name: String,
    email: String,
    password: String,
}

impl IgniteResource for Users {
    type Record = UserRecord;
    type Input = UserInput;

    const SLUG: &'static str = "users";
    const TITLE: &'static str = "Users";

    fn widgets(conn: &mut MysqlConnection) -> QueryResult<Vec<StatWidget>> {
        let total: i64 = users::table.count().get_result(conn)?;
        // Compared in the database's own clock, which is what created_at uses.
        let this_week: i64 = users::table
            .filter(sql::<Bool>("created_at >= NOW() - INTERVAL 7 DAY"))
            .count()
            .get_result(conn)?;
        Ok(vec![
            StatWidget {
                label: "Total users",
                value: total,
            },
            StatWidget {
                label: "New this week",
                value: this_week,
            },
        ])
    }

    fn fields() -> &'static [Field] {
        FIELDS
    }

    fn columns() -> &'static [Column] {
        COLUMNS
    }

    fn list(conn: &mut MysqlConnection, query: &ListQuery) -> QueryResult<Page<UserRecord>> {
        let pattern = query
            .search
            .as_ref()
            .map(|s| format!("%{}%", escape_like(s)));
        let mut rows = users::table.into_boxed();
        let mut count = users::table.into_boxed();
        if let Some(pattern) = &pattern {
            rows = rows.filter(
                users::name
                    .like(pattern.clone())
                    .or(users::email.like(pattern.clone())),
            );
            count = count.filter(
                users::name
                    .like(pattern.clone())
                    .or(users::email.like(pattern.clone())),
            );
        }

        let desc = matches!(query.sort, Some((_, Direction::Desc)));
        let key = query.sort.as_ref().map(|(key, _)| key.as_str());
        rows = match (key, desc) {
            (Some("name"), false) => rows.order(users::name.asc()),
            (Some("name"), true) => rows.order(users::name.desc()),
            (Some("email"), false) => rows.order(users::email.asc()),
            (Some("email"), true) => rows.order(users::email.desc()),
            (Some("created_at"), false) => rows.order(users::created_at.asc()),
            (Some("created_at"), true) => rows.order(users::created_at.desc()),
            _ => rows.order(users::id.asc()),
        };

        let total: i64 = count.count().get_result(conn)?;
        let rows = rows
            .limit(query.per_page)
            .offset(query.offset())
            .select(UserRecord::as_select())
            .load(conn)?;
        Ok(Page { rows, total })
    }

    fn find(conn: &mut MysqlConnection, id: i64) -> QueryResult<Option<UserRecord>> {
        users::table
            .find(id)
            .select(UserRecord::as_select())
            .first(conn)
            .optional()
    }

    fn create(conn: &mut MysqlConnection, input: UserInput) -> QueryResult<UserRecord> {
        // The panel's create_user hashes the password with argon2.
        let user =
            ignite_panel::users::create_user(conn, &input.name, &input.email, &input.password)?;
        Ok(UserRecord {
            id: user.id,
            name: user.name,
            email: user.email,
            created_at: user.created_at,
        })
    }

    fn update(conn: &mut MysqlConnection, id: i64, input: UserInput) -> QueryResult<UserRecord> {
        let password_hash = ignite_panel::users::hash_password(&input.password).map_err(|e| {
            diesel::result::Error::SerializationError(Box::new(std::io::Error::other(
                e.to_string(),
            )))
        })?;
        diesel::update(users::table.find(id))
            .set((
                users::name.eq(&input.name),
                users::email.eq(&input.email),
                users::password_hash.eq(password_hash),
            ))
            .execute(conn)?;
        Self::find(conn, id)?.ok_or(diesel::result::Error::NotFound)
    }

    fn delete(conn: &mut MysqlConnection, id: i64) -> QueryResult<usize> {
        diesel::delete(users::table.find(id)).execute(conn)
    }

    fn validate(form: &FormInput) -> Result<UserInput, FieldErrors> {
        let mut errors = FieldErrors::new();
        let name = FIELDS[0].check_required(form, &mut errors);
        let email = FIELDS[1].check_required(form, &mut errors);
        let password = FIELDS[2].check_required(form, &mut errors);

        if let Some(email) = email {
            if !email.contains('@') || email.starts_with('@') || email.ends_with('@') {
                errors.insert(
                    "email",
                    "Email must look like name@example.com.".to_string(),
                );
            }
        }
        if let Some(password) = password {
            if password.chars().count() < MIN_PASSWORD_CHARS {
                errors.insert(
                    "password",
                    format!("Password must be at least {MIN_PASSWORD_CHARS} characters."),
                );
            }
        }

        match (name, email, password) {
            (Some(name), Some(email), Some(password)) if errors.is_empty() => Ok(UserInput {
                name: name.to_string(),
                email: email.to_string(),
                password: password.to_string(),
            }),
            _ => Err(errors),
        }
    }
}
