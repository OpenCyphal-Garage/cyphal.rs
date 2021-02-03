import pathlib
import importlib
import pyuavcan
import pyuavcan.transport.can
import pyuavcan.transport.can.media.socketcan
import sys
import asyncio

dsdl_generated_dir = pathlib.Path("py-dsdl")
dsdl_generated_dir.mkdir(parents=True, exist_ok=True)
sys.path.insert(0, str(dsdl_generated_dir))

data_types = pathlib.Path("../../../public_regulated_data_types")
if not data_types.exists():
    print("public_regulated_data_types has not been cloned!")
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

async def big_subscriber(msg: uavcan.primitive.String_1_0, some_other_param):
    print(msg.value)

if __name__ == "__main__":
    hb_publisher = pyuavcan.application.heartbeat_publisher.HeartbeatPublisher(presentation)

    hb_publisher.start()

    loop = asyncio.get_event_loop()
    loop.create_task(big_publisher())

    sub = presentation.make_subscriber(uavcan.primitive.String_1_0, 100)
    sub.receive_in_background(big_subscriber)

    loop.run_forever()
