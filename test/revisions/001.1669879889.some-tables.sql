-- Revision: some-tables
--
-- Sets up a few basic tables to test a handful of data types,
-- indexes, constraints, and foreign keys

begin;

create schema ext;
create extension ltree with schema ext;


create table state (
  id
    int
    primary key
    generated always as identity,

  abbreviation
    varchar(2)
    unique
    not null
    check (abbreviation similar to '[A-Z]{2}'),

  name
    text
    unique
    not null
    check (length(name) between 4 and 13)
);
comment on table state is 'A US state';


create table city (
  id
    int
    primary key
    generated always as identity,

  state_id
    int
    not null
    references state (id),

  name
    text
    unique
    not null
    check (length(name) between 3 and 20)
);
comment on table city is 'A US city';


create table band (
  id
    int
    primary key
    generated always as identity,

  added_on
    timestamptz
    not null
    default now(),

  name
    text
    unique
    not null
    check (length(name) < 50 and trim(name) <> '')
);
comment on table band is 'A band is an entity composed of musicians';


create table musician (
  id
    int
    primary key
    generated always as identity,

  added_on
    timestamptz
    not null
    default now(),

  hometown
    int
    not null
    references city (id),

  dob
    date
    not null,

  name
    text
    not null
    check (trim(name) <> ''),

  -- This is not STRICTLY true to real life
  unique (hometown, dob, name)
);
comment on table musician is 'A musician is any person who plays music or sings, in a band or alone';
comment on column musician.hometown is 'The town where the musician was born';
comment on column musician.dob is 'The date the musician was born';


create table band_member (
  id
    int
    primary key
    generated always as identity,

  band_id
    int
    not null
    references band (id),

  musician_id
    int
    not null
    references musician (id),

  active
    boolean
    not null
    default true,

  unique (band_id, musician_id)
);
comment on table band_member is 'The link between a musician and a band, including whether or not the musician is an active member';


create table genre (
  id
    int
    primary key
    generated always as identity,

  path
    ext.ltree
    unique
    not null,

  name
    text
    unique
    not null
    check (name similar to '[A-Za-z''-/ ]{2,20}')
);


create table song (
  id
    int
    primary key
    generated always as identity,

  released
    date
    not null,

  name
    text
    not null
    check (trim(name) <> '')
);


create table song_genre (
  id
    int
    primary key
    generated always as identity,

  song_id
    int
    references song (id),

  genre_id
    int
    references genre (id),

  unique (song_id, genre_id)
);
comment on table song_genre is 'The inclusion of a song within a genre; a song can belong to many genres';


create table song_artist (
  id
    int
    primary key
    generated always as identity,

  band_id
    int
    references band (id),

  musician_id
    int
    references musician (id),

  "primary"
    boolean
    not null
    default true,

  check (
    coalesce(band_id, musician_id) is not null and
    (band_id is null or musician_id is null)
  )
);
comment on table song_artist is 'Represents any given solo musician or band who contributed to recording the song, including whether or not they were the primary artist. Either the musician or band must be assigned, but not both.';

create unique index song_artist_primary_idx on song_artist (id, "primary") where "primary"; 


commit;
