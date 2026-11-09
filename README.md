# Rocket Ignite example

A Rocket app that mounts the [Ignite](https://github.com/lajennylove/Ignite) admin panel: a dashboard with stat cards, and a Users resource to list, search, sort, create, edit and delete users. Views are Propel views in `resources/views/`.

## Setup

You need Rust (stable), and MariaDB or MySQL. The app is built and run in debug mode. Release builds wait on a Propel compile error (see Propel's `docs/production.md`).

1. Create the database and a user that can reach it. With MariaDB on this machine, the socket login needs no password:

   ```sh
   mysql -e "CREATE DATABASE ignite_example CHARACTER SET utf8mb4; GRANT ALL PRIVILEGES ON ignite_example.* TO 'you'@'localhost';"
   ```

2. Copy the environment template and edit it:

   ```sh
   cp .env.example .env
   ```

   - `DATABASE_URL` is `mysql://user:password@host:port/database`. Keep the password out of git; `.env` is ignored.
   - `IGNITE_SECRET_KEY` signs the login session and the flash messages. Generate one with `openssl rand -base64 32`. Without it, a debug build makes a random key for each run, so every restart signs everyone out.

## Migrations and seed data

The panel owns the `users` table. Its migration runs when the app starts, so there is nothing to run by hand. The `ignite user create` command runs it too, so the table exists before the first admin is created.

To add the fake users in `seeds/users.sql` (five rows with fake names and emails, which cannot log in):

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

## Walkthrough

1. Open `http://127.0.0.1:8000/admin`. You are sent to the sign-in page.
2. Sign in with the email and password from the step above. The dashboard shows two cards: total users, and users created in the last seven days.
3. Open **Users**. The list shows 20 rows per page. Type in the search box to match a name or an email. Click a column heading to sort, and click it again to reverse.
4. Click **New Users** to create a user. A blank field, a bad email or a short password shows an error beside the field.
5. Click **Edit** on a row to change a user. The password is required on edit too, because the stored hash is never shown: enter the new password, or the same one again.
6. Click **Delete** on a row, then confirm. This cannot be undone.
7. Click **Sign out** to end the session.

## Known limits

- A duplicate email on create or edit returns a server error, not a field message. The form checks the format of the email, not whether it is taken.
- Diesel is pinned to a fork at `a07c4b2c` in `Cargo.toml`, with the libmariadb 3.4 fix. Remove that pin when a Diesel release contains the fix.
