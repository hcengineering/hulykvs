create table kvs (
    namespace text not null,
    key text not null,
    md5 bytea not null,
    value bytea not null,

    primary key (namespace, key)
);
