# Benchmark scylla_bench results - 03 June 2025

## Goals

Compare Charybdis ORM, Scylla Driver and grapple_db scylla client

## About environment

OS: Debian GNU/Linux 12  
NVMe SSD 512Gb  
RAM: 16 Gb  
CPU: 12 x Ryzen 5600H

cargo 1.87.0 (99624be96 2025-05-06)  
rustc 1.87.0 (17067e9ac 2025-05-09)  
grapple_db 0.1.0
scylla 1.2.0  
charybdis 1.0.1

## Run Scylla

```bash
sudo docker run --rm -it -p 9042:9042 scylladb/scylla --smp 8
```

## Run benchmark

```bash
cargo bench --features scylla --bench scylla_bench
```

## Results

### Insert

Insert Benchmarks/ORM Insert  
time: [72.913 µs 74.287 µs 75.730 µs]  
thrpt: [13.205 Kelem/s 13.461 Kelem/s 13.715 Kelem/s]  
Found 13 outliers among 100 measurements (13.00%)  
12 (12.00%) high mild  
1 (1.00%) high severe

Insert Benchmarks/My Insert  
time: [72.402 µs 73.367 µs 74.340 µs]  
thrpt: [13.452 Kelem/s 13.630 Kelem/s 13.812 Kelem/s]  
Found 10 outliers among 100 measurements (10.00%)  
1 (1.00%) low mild  
5 (5.00%) high mild  
4 (4.00%) high severe

Insert Benchmarks/Native Insert  
time: [72.242 µs 72.824 µs 73.401 µs]  
thrpt: [13.624 Kelem/s 13.732 Kelem/s 13.842 Kelem/s]  
Found 4 outliers among 100 measurements (4.00%)  
2 (2.00%) low mild  
1 (1.00%) high mild  
1 (1.00%) high severe

### Batch Insert

ORM Batch Insert/ORM Batch Insert  
time: [6.3842 ms 6.4372 ms 6.4910 ms]  
thrpt: [154.06 Kelem/s 155.35 Kelem/s 156.64 Kelem/s]

ORM Batch Insert/My Batch Insert  
time: [6.5749 ms 6.6854 ms 6.8175 ms]  
thrpt: [146.68 Kelem/s 149.58 Kelem/s 152.09 Kelem/s]  
Found 5 outliers among 100 measurements (5.00%)  
1 (1.00%) high mild  
4 (4.00%) high severe

ORM Batch Insert/Native Batch Insert  
time: [8.8720 ms 9.0866 ms 9.2897 ms]  
thrpt: [107.65 Kelem/s 110.05 Kelem/s 112.71 Kelem/s]  
Found 9 outliers among 100 measurements (9.00%)  
8 (8.00%) low mild  
1 (1.00%) high mild

### Find

ORM Find time: [75.884 µs 77.298 µs 78.673 µs]  
Found 5 outliers among 100 measurements (5.00%)  
4 (4.00%) high mild  
1 (1.00%) high severe

My Find time: [76.775 µs 78.053 µs 79.338 µs]  
Found 7 outliers among 100 measurements (7.00%)  
1 (1.00%) low mild  
6 (6.00%) high mild

Native Find time: [72.876 µs 74.206 µs 75.598 µs]

### Stream

ORM Stream - Find Posts Per Partition  
time: [22.317 ms 22.489 ms 22.652 ms]  
Found 1 outliers among 100 measurements (1.00%)  
1 (1.00%) low mild

My Stream - Find Posts Per Partition  
time: [21.553 ms 21.711 ms 21.870 ms]

Native Stream - Find Posts Per Partition  
time: [22.485 ms 22.626 ms 22.771 ms]

### Update

ORM Update time: [70.696 µs 71.465 µs 72.231 µs]

My Update time: [71.621 µs 72.517 µs 73.391 µs]  
Found 1 outliers among 100 measurements (1.00%)  
1 (1.00%) high mild

Native Update time: [70.640 µs 71.818 µs 72.991 µs]  
Found 3 outliers among 100 measurements (3.00%)  
3 (3.00%) high mild

### Delete

ORM Delete time: [67.916 µs 68.876 µs 69.842 µs]

My Delete time: [65.924 µs 66.671 µs 67.471 µs]  
Found 2 outliers among 100 measurements (2.00%)  
2 (2.00%) high mild

Native Delete time: [67.863 µs 68.664 µs 69.423 µs]
