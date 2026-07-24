CREATE TABLE server_settings (
    key VARCHAR(128) PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID REFERENCES users(id)
);

INSERT INTO server_settings (key, value) VALUES
    ('maintenance_mode_enabled', 'false'),
    ('default_max_file_size_bytes', '26214400'),
    ('default_storage_quota_bytes', '10737418240'),
    ('allowed_signup_domains', '[]');
