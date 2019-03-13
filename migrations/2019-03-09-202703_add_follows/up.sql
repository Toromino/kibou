ALTER TABLE actors
ADD followers JSONB NOT NULL DEFAULT '{"activitypub": []}'::jsonb;
