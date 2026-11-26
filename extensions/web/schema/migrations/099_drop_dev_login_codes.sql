-- The developer login link is gone: an install member cannot mint a browser
-- session, so a redeemed code could never sign anyone in, and nothing issued
-- codes. Its table goes with it.
DROP TABLE IF EXISTS dev_login_codes;
