-- Fake users for the example. Run after migrations:
--   mysql ignite_example < seeds/users.sql
-- The password_hash values are placeholders, not real argon2 hashes, so
-- these users cannot log in. Create the first admin with the Ignite CLI.
INSERT IGNORE INTO users (name, email, password_hash, created_at) VALUES
    ('Ada Lovelace', 'ada@example.test', 'seed-not-a-login', NOW() - INTERVAL 40 DAY),
    ('Grace Hopper', 'grace@example.test', 'seed-not-a-login', NOW() - INTERVAL 12 DAY),
    ('Alan Turing', 'alan@example.test', 'seed-not-a-login', NOW() - INTERVAL 9 DAY),
    ('Katherine Johnson', 'katherine@example.test', 'seed-not-a-login', NOW() - INTERVAL 3 DAY),
    ('Linus Torvalds', 'linus@example.test', 'seed-not-a-login', NOW() - INTERVAL 1 DAY);
