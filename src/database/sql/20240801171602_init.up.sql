CREATE TYPE notify_type AS ENUM ('wednesday', 'crypto');

CREATE TABLE "chats" (
    "chat_id" bigint UNIQUE NOT NULL,
    "enabled_notifications" notify_type[]
);

CREATE TABLE "mapping" (
    user_id bigint UNIQUE NOT NULL,
    username text NOT NULL
);

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

create function add_chat(_chat_id bigint, _notify notify_type) returns void as $$
declare
	_exists bool;
begin
    select exists(select chat_id from chats where chat_id = _chat_id) into _exists;
	if _exists then
		update chats
		set enabled_notifications = array_append(enabled_notifications, _notify)
		where _chat_id = chat_id and not _notify = any(enabled_notifications);
	else
		insert into chats (chat_id, enabled_notifications) values (_chat_id, ARRAY[_notify]);
	end if;
end;
$$ language plpgsql;

create function remove_chat(_chat_id bigint, _notify notify_type) returns void as $$
declare
	_exists bool;
begin
    select exists(select chat_id from chats where chat_id = _chat_id) into _exists;
	if _exists then
		update chats
		set enabled_notifications = array_remove(enabled_notifications, _notify)
		where chat_id = _chat_id and _notify = any(enabled_notifications);
	end if;
end;
$$ language plpgsql;
