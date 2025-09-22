-- Add migration script here
-- if user is added to chat, notify with chat data
CREATE OR REPLACE FUNCTION notify_user_added_to_chat()
RETURNS TRIGGER AS $$
BEGIN
    RAISE NOTICE 'added to chat: %', NEW;
    PERFORM pg_notify('user_added_to_chat', json_build_object(
        'op', TG_OP,
        'old', OLD,
        'new', NEW
    )::text);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER user_added_to_chat_trigger
AFTER INSERT OR UPDATE OR DELETE ON chats
FOR EACH ROW
EXECUTE FUNCTION notify_user_added_to_chat();

-- if new message is added, notify with message data
CREATE OR REPLACE FUNCTION notify_new_message()
RETURNS TRIGGER AS $$
BEGIN
    RAISE NOTICE 'new message: %', NEW;
    IF TG_OP = 'INSERT' THEN
        PERFORM pg_notify('new_message', row_to_json(NEW)::text);
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER new_message_trigger
AFTER INSERT ON messages
FOR EACH ROW
EXECUTE FUNCTION notify_new_message();
