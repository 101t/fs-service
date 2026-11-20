-- One row per finished call leg, written by the built-in CDR recorder.
CREATE TABLE cdr (
    uuid               TEXT PRIMARY KEY,
    batch_id           TEXT,                -- set when the call was started through this API
    direction          TEXT NOT NULL,       -- inbound | outbound
    caller_id_name     TEXT,
    caller_id_number   TEXT,
    destination_number TEXT,
    gateway            TEXT,
    start_at           INTEGER NOT NULL,    -- unix epoch seconds
    answer_at          INTEGER,
    end_at             INTEGER NOT NULL,
    duration           INTEGER NOT NULL,
    billsec            INTEGER NOT NULL,
    hangup_cause       TEXT NOT NULL,
    fax_success        INTEGER,             -- NULL when the call carried no fax
    fax_result_text    TEXT,
    fax_pages          INTEGER
);
CREATE INDEX cdr_start_at ON cdr (start_at);
CREATE INDEX cdr_batch_id ON cdr (batch_id) WHERE batch_id IS NOT NULL;
CREATE INDEX cdr_destination ON cdr (destination_number, start_at);
CREATE INDEX cdr_caller ON cdr (caller_id_number, start_at);

-- One row per call requested through POST /api/v1/calls/*.
CREATE TABLE call_job (
    uuid        TEXT PRIMARY KEY,           -- also the FreeSWITCH channel UUID and bgapi Job-UUID
    batch_id    TEXT NOT NULL,
    kind        TEXT NOT NULL,              -- voice | fax
    gateway     TEXT NOT NULL,
    destination TEXT NOT NULL,
    file        TEXT NOT NULL,
    created_at  INTEGER NOT NULL,
    sent_at     INTEGER,                    -- when FreeSWITCH accepted the originate
    error       TEXT                        -- originate failure (e.g. INVALID_GATEWAY)
);
CREATE INDEX call_job_batch ON call_job (batch_id);
