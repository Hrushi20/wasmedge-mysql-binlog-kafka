### Build the project

``` rust
cargo build --target=wasm32-wasi
```

### Start the docker containers
```
docker-compose up -d
```

### Run the wasmedge binlog application
```
wasmedge target/wasm32-wasi/debug/mysql-binlog-kafka.wasm
```

### Open a new terminal and run the insert sql query using wasmedge
```
wasmedge --env "DATABASE_URL=mysql://root:password@127.0.0.1:3306/test" sql-commands-test-wasm/insert.wasm
```
* `insert.wasm` file is compiled to wasm using `src/sql-commands-test-wasm/insert.rs` file. 
* The password for root user is present in `docker-compose.yml` file.

### Observation
You should be able to see some event logs printed in the wasmedge binlog application console. These logs are first stored in kafka and fetched from it to print it in the terminal.  








