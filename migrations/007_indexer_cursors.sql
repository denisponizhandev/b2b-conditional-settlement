CREATE TABLE indexer_cursors (
    chain_id    BIGINT PRIMARY KEY,
    next_block  BIGINT NOT NULL,
    updated_at  TIMESTAMP NOT NULL DEFAULT now()
);