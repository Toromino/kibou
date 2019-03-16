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

allow_tables_to_appear_in_same_query!(activities, actors,);
