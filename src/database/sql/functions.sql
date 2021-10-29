drop function if exists update_mapping;
create or replace function update_mapping(in _user_id bigint, in _username varchar) returns void as $$
declare
    _entry_exists bool;
begin
    select exists(select 1 from "mapping" where "user_id" = _user_id) limit 1 into _entry_exists;
    if _entry_exists then
        update "mapping" set username = _username where user_id = _user_id;
    else
        insert into "mapping" values (_user_id, _username);
    end if;
end;
$$ language plpgsql;

drop function if exists update_statistics;
create function update_statistics(_chat_id bigint, _user_id bigint, _kind integer, _today date) returns void as $$
declare
    _entry_exists bool;
    _count bigint;
begin
    select "count" from "statistics"
        where "chat" = _chat_id and "user" = _user_id and "kind" = _kind and "date" = _today
        into _count;

    _entry_exists = _count IS NOT NULL;

    if _entry_exists then
        update "statistics" set "count" = _count + 1
            where "chat" = _chat_id and "user" = _user_id and "kind" = _kind and "date" = _today;
    else
        insert into "statistics" values (_chat_id, _user_id, 1, _today, _kind);
    end if;
end;
$$ language plpgsql;