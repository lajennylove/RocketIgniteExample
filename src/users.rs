//! The Users resource: the panel's list, form and actions for the `users` table.
//!
//! Written by hand rather than derived. The derive hashes a password field, but
//! this resource also needs an eight-character minimum and an email shape check
//! in `validate`, so those rules are written here.

use chrono::NaiveDateTime;
use ignite_core::{
    escape_like, Column, Direction, Field, FieldErrors, FieldKind, FormInput, IgniteResource,
    LatestRecords, ListQuery, Page, StatWidget,
};
use orbit_orm::{Database, Direction as OrbitDirection, Error, Model, Operator, OrbitModel, Query};
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
    // Required on create, optional on edit: a blank value keeps the stored hash.
    // The create check lives in `validate`, not in this flag.
    Field {
        name: "password",
        label: "Password",
        kind: FieldKind::Password,
        required: false,
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

/// One row of the list and the edit page. The password hash is not a field here, so
/// it is never selected. The panel's `update_password` writes it.
#[derive(Debug, OrbitModel, Serialize)]
#[orbit(table = "users")]
pub struct UserRecord {
    pub id: i64,
    #[orbit(fillable)]
    pub name: String,
    #[orbit(fillable)]
    pub email: String,
    pub created_at: NaiveDateTime,
}

/// Validated form values for create and edit.
pub struct UserInput {
    name: String,
    email: String,
    password: String,
}

/// Narrows `query` to the rows whose name or email contains `pattern`.
fn search(query: Query<UserRecord>, pattern: &str) -> Query<UserRecord> {
    query.where_group(|q| {
        q.where_op("name", Operator::Like, pattern)
            .or_where_op("email", Operator::Like, pattern)
    })
}

impl IgniteResource for Users {
    type Record = UserRecord;
    type Input = UserInput;

    const SLUG: &'static str = "users";
    const TITLE: &'static str = "Users";
    const SINGULAR: &'static str = "User";

    fn latest_records() -> Option<LatestRecords> {
        Some(LatestRecords {
            title: "Latest users",
            columns: &["name", "email", "created_at"],
            limit: 5,
        })
    }

    async fn latest(db: &Database, limit: i64) -> Result<Vec<UserRecord>, Error> {
        UserRecord::query()
            .order_by("id", OrbitDirection::Desc)
            .limit(limit.max(0) as u64)
            .get(db)
            .await
    }

    async fn widgets(db: &Database) -> Result<Vec<StatWidget>, Error> {
        let total = UserRecord::query().count(db).await?;
        // Compared in the database's own clock, which is what created_at uses.
        let this_week: i64 = orbit_orm::raw::fetch_scalar(
            db,
            "SELECT COUNT(*) FROM users WHERE created_at >= NOW() - INTERVAL 7 DAY",
            &[],
        )
        .await?;
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

    async fn list(db: &Database, query: &ListQuery) -> Result<Page<UserRecord>, Error> {
        let pattern = query
            .search
            .as_ref()
            .map(|s| format!("%{}%", escape_like(s)));
        let mut rows = UserRecord::query();
        let mut count = UserRecord::query();
        if let Some(pattern) = &pattern {
            rows = search(rows, pattern);
            count = search(count, pattern);
        }

        let desc = matches!(query.sort, Some((_, Direction::Desc)));
        let key = query.sort.as_ref().map(|(key, _)| key.as_str());
        let (column, direction) = match (key, desc) {
            (Some("name"), false) => ("name", OrbitDirection::Asc),
            (Some("name"), true) => ("name", OrbitDirection::Desc),
            (Some("email"), false) => ("email", OrbitDirection::Asc),
            (Some("email"), true) => ("email", OrbitDirection::Desc),
            (Some("created_at"), false) => ("created_at", OrbitDirection::Asc),
            (Some("created_at"), true) => ("created_at", OrbitDirection::Desc),
            _ => ("id", OrbitDirection::Asc),
        };
        rows = rows.order_by(column, direction);

        let total = count.count(db).await?;
        let rows = rows
            .limit(query.per_page.max(0) as u64)
            .offset(query.offset().max(0) as u64)
            .get(db)
            .await?;
        Ok(Page { rows, total })
    }

    async fn find(db: &Database, id: i64) -> Result<Option<UserRecord>, Error> {
        UserRecord::query().find(db, id).await
    }

    async fn create(db: &Database, input: UserInput) -> Result<UserRecord, Error> {
        // The panel's create_user hashes the password with argon2.
        let user = ignite_panel::users::create_user(db, &input.name, &input.email, &input.password)
            .await?;
        Ok(UserRecord {
            id: user.id,
            name: user.name,
            email: user.email,
            created_at: user.created_at,
        })
    }

    async fn update(db: &Database, id: i64, input: UserInput) -> Result<UserRecord, Error> {
        // A blank password keeps the stored hash. The name, the email and the hash change
        // in one transaction, so a failed hash leaves the row as it was.
        db.transaction(async |tx| {
            UserRecord::query()
                .where_eq("id", id)
                .update(
                    &mut *tx,
                    &[
                        ("name", input.name.as_str().into()),
                        ("email", input.email.as_str().into()),
                    ],
                )
                .await?;
            if !input.password.is_empty() {
                ignite_panel::users::update_password(&mut *tx, id, &input.password).await?;
            }
            Ok::<(), Error>(())
        })
        .await?;
        Self::find(db, id).await?.ok_or(Error::NotFound {
            table: UserRecord::TABLE,
        })
    }

    async fn delete(db: &Database, id: i64) -> Result<u64, Error> {
        UserRecord::query().where_eq("id", id).delete(db).await
    }

    fn validate(form: &FormInput) -> Result<UserInput, FieldErrors> {
        validate_form(form, true)
    }

    fn validate_update(form: &FormInput) -> Result<UserInput, FieldErrors> {
        validate_form(form, false)
    }
}

/// Checks the form. A password is required on create and optional on edit.
fn validate_form(form: &FormInput, password_required: bool) -> Result<UserInput, FieldErrors> {
    let mut errors = FieldErrors::new();
    let name = FIELDS[0].check_required(form, &mut errors);
    let email = FIELDS[1].check_required(form, &mut errors);
    let password = if password_required {
        FIELDS[2].check_required(form, &mut errors)
    } else {
        form.get("password")
    };

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
        (Some(name), Some(email), None) if errors.is_empty() && !password_required => {
            Ok(UserInput {
                name: name.to_string(),
                email: email.to_string(),
                password: String::new(),
            })
        }
        _ => Err(errors),
    }
}
