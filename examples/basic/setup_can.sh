#!/bin/sh

modprobe can
modprobe can_raw
modprobe vcan
ip link add dev vcan0 type vcan
ip link set vcan0 mtu 72         # Enable CAN FD by configuring the MTU of 64+8
ip link set up vcan0