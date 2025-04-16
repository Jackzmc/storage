create table libraries
(
    owner_id   uuid                    not null
        constraint libraries_owner_id
            references users
            on update cascade on delete cascade,
    created_at timestamp default now() not null,
    name       varchar(255)            not null,
    repo_id    varchar(64)             not null
        constraint libraries_repo_id
            references repos
            on update cascade on delete cascade,
    id         uuid                    not null
        constraint libraries_pk
            primary key
);
create table repos
(
    id               varchar(64)             not null
        primary key,
    created_at       timestamp default now() not null,
    storage_type     varchar(32)             not null,
    storage_settings json                    not null,
    flags            smallint  default 0     not null
);
create table users
(
    id         uuid                    not null
        primary key,
    created_at timestamp default now() not null,
    name       varchar(255)            not null,
    password   varchar(128),
    email      varchar(128)            not null,
    username   varchar(64)             not null
);

