PRAGMA foreign_keys=OFF;
BEGIN TRANSACTION;
CREATE TABLE cdr (
    caller_id_name VARCHAR,
    caller_id_number VARCHAR,
    destination_number VARCHAR,
    context VARCHAR,
    start_stamp DATETIME,
    answer_stamp DATETIME,
    end_stamp DATETIME,
    duration INTEGER,
    billsec INTEGER,
    hangup_cause VARCHAR,
    uuid VARCHAR,
    bleg_uuid VARCHAR,
    account_code VARCHAR
, flag_imported INTEGER DEFAULT 0);
COMMIT;


--<template name="example">
-- "${caller_id_name}",
-- "${caller_id_number}",
-- "${destination_number}",
-- "${context}",
-- "${start_stamp}",
-- "${answer_stamp}",
-- "${end_stamp}",
-- ${duration},
-- ${billsec},
-- "${hangup_cause}", 
-- "${uuid}",
-- "${bleg_uuid}",
-- "${accountcode}",
-- 0
-- </template>