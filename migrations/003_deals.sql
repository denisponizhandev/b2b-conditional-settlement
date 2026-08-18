CREATE TABLE deals (
    id              UUID PRIMARY KEY,
    payer_org_id    UUID NOT NULL REFERENCES organizations(id),
    payee_org_id    UUID NOT NULL REFERENCES organizations(id),
    status          TEXT NOT NULL,
    chain_address   TEXT,
    created_at      TIMESTAMP NOT NULL DEFAULT now()
);

CREATE INDEX deals_payer_org_id_idx ON deals (payer_org_id);
CREATE INDEX deals_payee_org_id_idx ON deals (payee_org_id);