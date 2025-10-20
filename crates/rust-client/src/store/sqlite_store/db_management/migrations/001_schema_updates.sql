CREATE INDEX IF NOT EXISTS idx_accounts_id_nonce ON accounts(id, nonce DESC);

CREATE INDEX IF NOT EXISTS idx_input_notes_state ON input_notes(state_discriminant);

CREATE INDEX IF NOT EXISTS idx_output_notes_state ON output_notes(state_discriminant);
