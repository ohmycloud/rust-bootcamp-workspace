# Notify Server

```bash
cd rchat/dev_tools && cargo run --bin db-helper -- -m ../migrations reset -d chat -s ../chat_server/fixtures/test.sql
cd rchat/chat_server && cargo run
cd rchat/notify_server && cargo run

# how to test pg notification
# signup
# signin
# create chat
```
