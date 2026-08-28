INSERT INTO organizations (id, name) VALUES
    ('11111111-1111-1111-1111-111111111111', 'Demo Payer Org'),
    ('22222222-2222-2222-2222-222222222222', 'Demo Payee Org');

INSERT INTO deals (
    id,
    payer_org_id,
    payee_org_id,
    status,
    intent_id,
    chain_address
) VALUES (
    '33333333-3333-3333-3333-333333333335',
    '11111111-1111-1111-1111-111111111111',
    '22222222-2222-2222-2222-222222222222',
    'pending_chain_confirm',
    '0x7fa7a6495795420781b9cf2c1310b3bbbd64fdb22c1645fbbb8a77f69ebe005c',
    NULL
);

INSERT INTO milestones (deal_id, milestone_index, amount, released) VALUES
    ('33333333-3333-3333-3333-333333333333', 0, 500000, false),
    ('33333333-3333-3333-3333-333333333333', 1, 300000, false);

