- [RUN]
```
cargo run
```

- [WATCH]
```
cargo watch -x run
```

- [BUILD]
```
cargo build --release
```

Get database ipAddress:

```
docker ps
```

Get the ps responsible for the service that i want to get the ipAddress with:

```
docker inspect <ps-id-here> | grep IPAddress
```

SQLX-CLI
This cli is necessary for managing databases, migrations, and more...
https://github.com/launchbadge/sqlx/tree/main/sqlx-cli