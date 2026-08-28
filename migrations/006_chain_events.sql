CREATE TABLE chain_events (
    id                  UUID PRIMARY KEY,
    chain_id            BIGINT NOT NULL,
    block_number        BIGINT NOT NULL,
    tx_hash             TEXT NOT NULL,
    log_index           INTEGER NOT NULL,
    event_type          TEXT NOT NULL,
    deal_id             TEXT NOT NULL,
    contract_address    TEXT NOT NULL,
    payload             JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at          TIMESTAMP NOT NULL DEFAULT now(),
    
    CONSTRAINT chain_events_event_type_check CHECK (
        event_type IN ('deal_created', 'funded', 'milestone_released')
    ),

    CONSTRAINT chain_events_chain_tx_log_unique UNIQUE (chain_id, tx_hash, log_index)
);

CREATE INDEX chain_events_deal_id_idx ON chain_events (deal_id);
CREATE INDEX chain_events_block_number_idx ON chain_events (block_number);
CREATE INDEX chain_events_event_type_idx ON chain_events (event_type);