ALTER TABLE pairing_codes
ADD COLUMN IF NOT EXISTS pairing_secret VARCHAR(128);
