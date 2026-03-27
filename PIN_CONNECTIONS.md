# DHT11 Pin Connections

Connect the DHT11 sensor to your Raspberry Pi:

**DHT11 Sensor → Raspberry Pi GPIO:**

- **VCC (Power, usually red wire)** → **Pin 1 (3.3V)** or **Pin 2 (5V)**
- **GND (Ground, usually black wire)** → **Pin 6 (GND)** or any other GND pin
- **DATA (Signal, usually yellow/green wire)** → **Pin 7 (GPIO 4)**

**Optional:** 4.7kΩ pull-up resistor between VCC and DATA (many DHT11 modules include this internally)

**Physical Pin Layout (Raspberry Pi 40-pin header):**
```
    3.3V  [1]  [2]  5V
   GPIO2  [3]  [4]  5V
   GPIO3  [5]  [6]  GND
   GPIO4  [7]  [8]  GPIO14
     GND  [9] [10]  GPIO15
  GPIO17 [11] [12]  GPIO18
  GPIO27 [13] [14]  GND
  GPIO22 [15] [16]  GPIO23
    3.3V [17] [18]  GPIO24
  GPIO10 [19] [20]  GND
   GPIO9 [21] [22]  GPIO25
  GPIO11 [23] [24]  GPIO8
     GND [25] [26]  GPIO7
   GPIO0 [27] [28]  GPIO1
   GPIO5 [29] [30]  GND
   GPIO6 [31] [32]  GPIO12
  GPIO13 [33] [34]  GND
  GPIO19 [35] [36]  GPIO16
  GPIO26 [37] [38]  GPIO20
     GND [39] [40]  GPIO21
```
