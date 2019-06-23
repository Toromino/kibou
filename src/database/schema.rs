table! {
    activities (id) {
        id -> Int8,
        data -> Jsonb,
        created -> Timestamp,
        modified -> Timestamp,
        actor_uri -> Varchar,
    }
}

table! {
    actors (id) {
        id -> Int8,
        email -> Nullable<Varchar>,
        password -> Nullable<Varchar>,
        actor_uri -> Varchar,
        username -> Nullable<Varchar>,
        preferred_username -> Varchar,
        summary -> Nullable<Text>,
        inbox -> Nullable<Varchar>,
        icon -> Nullable<Varchar>,
        keys -> Jsonb,
        created -> Timestamp,
        modified -> Timestamp,
        local -> Bool,
        followers -> Jsonb,
    }
}

table! {
    notifications (id) {
        id -> Int8,
        activity_id -> Int8,
        actor_id -> Int8,
        created -> Timestamp,
        modified -> Timestamp,
    }
}

table! {
    oauth_applications (id) {
        id -> Int8,
        client_name -> Nullable<Varchar>,
        client_id -> Varchar,
        client_secret -> Varchar,
        redirect_uris -> Varchar,
        scopes -> Varchar,
        website -> Nullable<Varchar>,
        created -> Timestamp,
        modified -> Timestamp,
    }
}

table! {
    oauth_authorizations (id) {
        id -> Int8,
        application -> Int8,
        actor -> Varchar,
        code -> Varchar,
        created -> Timestamp,
        modified -> Timestamp,
        valid_until -> Timestamp,
    }
}

table! {
    oauth_tokens (id) {
        id -> Int8,
        application -> Int8,
        actor -> Varchar,
        access_token -> Varchar,
        refresh_token -> Varchar,
        created -> Timestamp,
        modified -> Timestamp,
        valid_until -> Timestamp,
    }
}

joinable!(notifications -> activities (activity_id));
joinable!(notifications -> actors (actor_id));

allow_tables_to_appear_in_same_query!(
    activities,
    actors,
    notifications,
    oauth_applications,
    oauth_authorizations,
    oauth_tokens,
);
