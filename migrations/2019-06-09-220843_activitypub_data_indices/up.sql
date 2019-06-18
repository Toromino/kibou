CREATE INDEX activities_ap_data ON activities USING GIN (data jsonb_path_ops);
