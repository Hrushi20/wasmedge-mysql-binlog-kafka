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

### Insert data into Mysql table after entering Mysql docker container
```
docker exec -it {docker_container_id} /bin/bash
mysql -u root -p
```
* Docker container id can be found using `docker ps` command. 
* Enter the password in the above step (Default Password: Password)

### Create a new database called test
``` mysql
create database test;
use test;
```

### Create a table in test database
``` mysql
create table names (name varchar(255));
```

### Insert data into table
``` mysql
insert into names values ('wasmedge');
insert into names values ('webassembly');
```

### Observation
You should be able to see some event logs printed in the wasmedge application console. These logs are stored, fetched and then displayed in the terminal.








