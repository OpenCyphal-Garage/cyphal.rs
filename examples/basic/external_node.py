import pathlib
import importlib
import pyuavcan
import pyuavcan.transport.can
import pyuavcan.transport.can.media.socketcan
import time
import sys
import os
import asyncio

dsdl_generated_dir = pathlib.Path("py-dsdl")
dsdl_generated_dir.mkdir(parents=True, exist_ok=True)
sys.path.insert(0, str(dsdl_generated_dir))

home_dir = os.environ.get("HOME")
data_types = pathlib.Path(f"{home_dir}/UAVCAN/public_regulated_data_types")
if not data_types.exists() or not data_types.is_dir():
    print(f"public_regulated_data_types has not been cloned! Please clone it into {data_types}")
    exit()

try:
    import pyuavcan.application
except:
    pyuavcan.dsdl.generate_package(
        root_namespace_directory= data_types / "uavcan/",
        output_directory=dsdl_generated_dir,
    )
    importlib.invalidate_caches()
    import pyuavcan.application

import uavcan.primitive

media = pyuavcan.transport.can.media.socketcan.SocketCANMedia("vcan0", 8)
transport = pyuavcan.transport.can.CANTransport(media, 10)
presentation = pyuavcan.presentation.Presentation(transport)

async def big_publisher():
    pub = presentation.make_publisher(uavcan.primitive.String_1_0, 100)
    while True:
        await pub.publish(uavcan.primitive.String_1_0([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]))
        await asyncio.sleep(1)

start_time = time.time()
async def hb_publisher():
    pub = presentation.make_publisher(uavcan.node.Heartbeat_1_0, uavcan.node.Heartbeat_1_0._FIXED_PORT_ID_)

    health = uavcan.node.Health_1_0(uavcan.node.Health_1_0.NOMINAL)
    mode = uavcan.node.Mode_1_0(uavcan.node.Mode_1_0.OPERATIONAL)

    while True:
        uptime = int(time.time() - start_time)
        await pub.publish(uavcan.node.Heartbeat_1_0(uptime, health, mode, 0))
        await asyncio.sleep(1)

async def big_subscriber(msg: uavcan.primitive.String_1_0, some_other_param):
    print(msg.value.tobytes().decode())

if __name__ == "__main__":
    loop = asyncio.get_event_loop()
    loop.create_task(big_publisher())
    loop.create_task(hb_publisher())

    sub = presentation.make_subscriber(uavcan.primitive.String_1_0, 100)
    sub.receive_in_background(big_subscriber)

    loop.run_forever()
