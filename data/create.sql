CREATE SCHEMA betting;

CREATE USER betting_user WITH PASSWORD '<INSERT SECURE PASSWORD HERE>';
GRANT USAGE ON SCHEMA betting TO betting_user;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA betting TO betting_user;
ALTER DEFAULT PRIVILEGES FOR ROLE markaronin IN SCHEMA betting GRANT SELECT, UPDATE, INSERT, DELETE ON TABLES TO betting_user;

CREATE TABLE betting.logs (
   created_at timestamptz PRIMARY KEY DEFAULT now(),
   content text not null
);
CREATE TABLE betting.users (
   id CHAR(36) PRIMARY KEY,
   "name" TEXT UNIQUE NOT NULL,
   "money" DOUBLE PRECISION NOT NULL
);
CREATE TABLE betting.bets (
   id CHAR(36) PRIMARY KEY,
   creator_id CHAR(36) REFERENCES betting.users(id) NOT NULL,
   created_seconds_since_epoch INTEGER NOT NULL,
   "name" TEXT NOT NULL,
   closed BOOLEAN NOT NULL,
   yes_pool DOUBLE PRECISION NOT NULL,
   no_pool DOUBLE PRECISION NOT NULL
);
CREATE TABLE betting.user_bets (
   user_id CHAR(36) REFERENCES betting.users(id) ON DELETE CASCADE NOT NULL,
   bet_id CHAR(36) REFERENCES betting.bets(id) ON DELETE CASCADE NOT NULL,
   is_yes BOOLEAN NOT NULL,
   amount INTEGER NOT NULL,
   spent DOUBLE PRECISION NOT NULL,
   PRIMARY KEY (user_id, bet_id, is_yes)
);