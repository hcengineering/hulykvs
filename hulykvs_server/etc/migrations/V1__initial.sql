create table kvs (
    namespace text not null, 
    key text not null,
    md5 bytes not null,
    value bytes not null,

    primary key (namespace, key)
);