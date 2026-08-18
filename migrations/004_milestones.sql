CREATE TABLE milestones (
    deal_id         UUID NOT NULL REFERENCES deals(id) ON DELETE CASCADE,
    milestone_index SMALLINT NOT NULL,
    amount          BIGINT NOT NULL CHECK (amount >= 0),
    released        BOOLEAN NOT NULL DEFAULT false,
    PRIMARY KEY     (deal_id, milestone_index)
);