CREATE TABLE IF NOT EXISTS betting_logs (
   created_at timestamptz PRIMARY KEY DEFAULT now(),
   content text not null
);