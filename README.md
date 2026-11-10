# Rocket Ignite example

A Rocket app that mounts the [Ignite](https://github.com/lajennylove/Ignite) admin panel: a dashboard with stat cards, and a Users resource to list, search, sort, create, edit and delete users. Views are Propel views in `resources/views/`.

## Setup

You need Rust (stable), and MariaDB or MySQL. Debug and release builds both work. A release build renders the views compiled by `build.rs`, and a debug build reads them at run time, so an edit shows on refresh.

1. Create the database and a user that can reach it. Replace `...` with a password of your choice, and put the same password in `DATABASE_URL` in step 2:

   ```sh
   mysql -e "CREATE DATABASE ignite_example CHARACTER SET utf8mb4; CREATE USER IF NOT EXISTS 'you'@'localhost' IDENTIFIED BY '...'; GRANT ALL PRIVILEGES ON ignite_example.* TO 'you'@'localhost';"
   ```

2. Copy the environment template and edit it:

   ```sh
   cp .env.example .env
   ```

   - `DATABASE_URL` is `mysql://user:password@host:port/database`. Keep the password out of git; `.env` is ignored.
   - `IGNITE_SECRET_KEY` signs the login session and the flash messages. Generate one with `openssl rand -base64 32`. Without it, a debug build makes a random key for each run, so every restart signs everyone out. A release build refuses to start without it.

## Migrations and seed data

The panel owns the `users` table. Create the tables before the first start, and again after each deploy that adds a migration, with the panel's command:

```sh
cargo run --bin ignite -- migrate
```

`ignite migrate:status` lists the migrations as applied or pending. `ignite user create` does not create tables, so run `migrate` first. The app itself does not migrate at startup.

To add the fake users in `seeds/users.sql` (five rows with fake names and emails, which cannot log in). Run it after the first start or after `ignite user create`, because the table must exist:

```sh
mysql ignite_example < seeds/users.sql
```

Running it twice adds nothing, because the emails are unique.

## Create the first admin

```sh
cargo run --bin ignite -- user create --email you@example.com --name "You"
```

The command asks for a password twice without echoing it. It must be at least eight characters.

## Run

```sh
cargo run
```

The app listens on `http://127.0.0.1:8000`. To use another port, set `ROCKET_PORT`, for example `ROCKET_PORT=8123 cargo run`.

## The sign-up form

`/subscribe` is the app's own form. Its POST route takes `CsrfForm` from Ignite, so a request without the page's `_token` is refused with status 419. The page writes the token into the form from the context, which the route fills with `CsrfToken`.

## The app's own view

`resources/views/home.propel` is the app's home page at `/`. It extends the panel's layout with `@extends('ignite::layouts.app')`, so it gets the panel's header and styles and fills the `title` and `content` sections. Copy it as the starting point for any page of your own.

## Walkthrough

1. Open `http://127.0.0.1:8000/admin`. You are sent to the sign-in page.
2. Sign in with the email and password from the step above. The dashboard shows two cards: **Total users**, and **New this week**, which counts the users created in the last seven days. The seven days are a rolling window measured on the database's clock, not a calendar week.
3. Open **Users**. The list shows 20 rows per page. Type in the search box to match a name or an email. Click a column heading to sort, and click it again to reverse.
4. Click **New User** to create a user. A blank field, a bad email or a short password shows an error beside the field.
5. Click **Edit** on a row to change a user. The password is optional on edit: a blank field keeps the stored password, and a new one replaces it.
6. Click **Delete** on a row, then confirm. This cannot be undone.
7. Click **Sign out** in the top bar. It ends every session of your account on every device, not only this one.

## Known limits

- The email must contain an `@` that is neither its first nor its last character. Nothing else is checked, so `a@b` is accepted. The error reads "Email must look like name@example.com." A duplicate email shows as a field error, "Email is already taken."
- Diesel is pinned to a fork at `a07c4b2c` in `Cargo.toml`, with the libmariadb 3.4 fix. Remove that pin when a Diesel release contains the fix.
