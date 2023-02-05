# accelerometer-service


## Verify/detect the presence of a device on the I2C bus
NOTE: - The M845xQ accelerometer defaults to address 0x1d
      - The ICM-20948 accelerometer defaults to address 0x68

      (only one is necessary)
```
pi@raspberrypi:~ $ i2cdetect -y 1
     0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f
00:                         -- -- -- -- -- -- -- --
10: -- -- -- -- -- -- -- -- -- -- -- -- -- 1d -- --
20: -- -- -- -- -- -- -- -- -- 29 -- -- -- -- -- --
30: -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --
40: -- -- -- -- -- -- -- -- 48 -- -- -- -- -- -- --
50: -- -- -- -- -- -- -- -- -- -- -- -- 5c -- -- --
60: -- -- -- -- -- -- -- -- 68 -- -- -- -- -- -- --
70: 70 -- -- -- -- -- -- --
```

## Cross-compile the code for ARM architecture
NOTE: This step is automated in the VSCode task *cross: build (aarch64)*

```
user@host:~ $ export CROSS_CONTAINER_IN_CONTAINER=1
user@host:~ $ cross build --release \
                          --target aarch64-unknown-linux-gnu
```

## Copy files to rpi
NOTE: This step is automated in the VSCode task *Deploy to rpi*

```
user@host:~ $ scp target/aarch64-unknown-linux-gnu/release/accelerometer-* \
                  data/* \
                  rpi:/var/tmp/
```

## Execute a device test (ICM-20948 shown)
```
pi@raspberrypi:~ $ export RUST_LOG=info

pi@raspberrypi:~ $ /var/tmp/accelerometer-tester --config-file-path /var/tmp/accelerometer.yaml
[2023-02-05T00:13:19Z INFO  acclrmtr] Chip type:          ICM20948
[2023-02-05T00:13:19Z INFO  acclrmtr] I²C Device file:    /dev/i2c-1
[2023-02-05T00:13:19Z INFO  acclrmtr] Full scale mode:    FourG
[2023-02-05T00:13:19Z INFO  acclrmtr] Data rate:          DataRate50Hz
[2023-02-05T00:13:19Z INFO  acclrmtr] Address:            0x68
[2023-02-05T00:13:20Z INFO  accelerometer_tester] Sample     0: 2023-02-05 00:13:20.445: Acc. (m/s²):  -0.00287,   0.01221,   0.00647

```

## Test the server
NOTE: RabbitMQ must be running in the k8s cluster on the rpi
      See https://github.com/kerrys-learning-lab/carbernetes for instructions
      on deploying/testing the cluster

```
# Start a port-forward to the RabbitMQ MQTT port (1883)
pi@raspberrypi:~ microk8s kubectl -n c9s port-forward svc/rabbitmq 1883:1883

# In another shell...
pi@raspberrypi:~ $ export RUST_LOG=info
pi@raspberrypi:~ $ /var/tmp/accelerometer-service --config-file-path /var/tmp/accelerometer-with-secrets.yaml
[2023-02-05T00:26:19Z INFO  acclrmtr] Chip type:          M845xQ
[2023-02-05T00:26:19Z INFO  acclrmtr] I²C Device file:    /dev/i2c-1
[2023-02-05T00:26:19Z INFO  acclrmtr] Full scale mode:    FourG
[2023-02-05T00:26:19Z INFO  acclrmtr] Data rate:          DataRate50Hz
[2023-02-05T00:26:19Z INFO  acclrmtr] Address:            0x1d
[2023-02-05T00:26:19Z INFO  accelerometer_service] Published sample     0: 2023-02-05T00:26:19.769: Acc. (m/s²):   0.00383,   0.03066,  -0.00766
[2023-02-05T00:27:59Z INFO  accelerometer_service] Published sample   100: 2023-02-05T00:27:59.941: Acc. (m/s²):   0.00383,   0.04982,  -0.02682

# In another shell...
pi@raspberrypi:~ $ export RUST_LOG=info
pi@raspberrypi:~ /var/tmp/accelerometer-test-consumer --config-file-path /var/tmp/accelerometer-with-secrets.yaml \
                                                      --max-samples 10
[2023-02-05T00:27:22Z INFO  accelerometer_test_consumer] Subscribed to topic: /c9s/accelerometer/measurement
[2023-02-05T00:27:22Z INFO  accelerometer_test_consumer] Received sample     1: 2023-02-05T00:27:22.881: Acc. (m/s²):   0.00383,   0.03066,  -0.00766
[2023-02-05T00:27:23Z INFO  accelerometer_test_consumer] Received sample     2: 2023-02-05T00:27:23.884: Acc. (m/s²):   0.00383,   0.01150,  -0.08430
[2023-02-05T00:27:24Z INFO  accelerometer_test_consumer] Received sample     3: 2023-02-05T00:27:24.885: Acc. (m/s²):   0.00383,   0.03066,  -0.00766
[2023-02-05T00:27:25Z INFO  accelerometer_test_consumer] Received sample     4: 2023-02-05T00:27:25.887: Acc. (m/s²):  -0.01533,  -0.00766,  -0.00766
[2023-02-05T00:27:26Z INFO  accelerometer_test_consumer] Received sample     5: 2023-02-05T00:27:26.888: Acc. (m/s²):  -0.03449,  -0.00766,  -0.08430
[2023-02-05T00:27:27Z INFO  accelerometer_test_consumer] Received sample     6: 2023-02-05T00:27:27.890: Acc. (m/s²):   0.00383,   0.03066,  -0.02682
[2023-02-05T00:27:28Z INFO  accelerometer_test_consumer] Received sample     7: 2023-02-05T00:27:28.891: Acc. (m/s²):   0.00383,   0.01150,  -0.04598
[2023-02-05T00:27:29Z INFO  accelerometer_test_consumer] Received sample     8: 2023-02-05T00:27:29.893: Acc. (m/s²):   0.00383,   0.01150,  -0.06514
[2023-02-05T00:27:30Z INFO  accelerometer_test_consumer] Received sample     9: 2023-02-05T00:27:30.894: Acc. (m/s²):   0.00383,   0.04982,  -0.00766
[2023-02-05T00:27:31Z INFO  accelerometer_test_consumer] Received sample    10: 2023-02-05T00:27:31.896: Acc. (m/s²):  -0.01533,  -0.00766,   0.01150
[2023-02-05T00:27:33Z INFO  accelerometer_test_consumer] Received sample    11: 2023-02-05T00:27:32.898: Acc. (m/s²):  -0.01533,   0.01150,  -0.00766

```
