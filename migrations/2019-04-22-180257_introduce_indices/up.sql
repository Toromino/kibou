CREATE INDEX activities_ap_actor ON activities ((data->'actor'));
CREATE INDEX activities_ap_cc ON activities USING GIN ((data->'cc'));
CREATE INDEX activities_ap_public ON activities ((data->'to')) WHERE (data->'to')::jsonb ? 'https://www.w3.org/ns/activitystreams#Public';
CREATE INDEX activities_ap_to ON activities USING GIN ((data->'to'));
