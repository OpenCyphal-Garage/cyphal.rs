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


if __name__ == "__main__":
    media = pyuavcan.transport.can.media.socketcan.SocketCANMedia("vcan0", 8)
    transport = pyuavcan.transport.can.CANTransport(media, 10)
    presentation = pyuavcan.presentation.Presentation(transport)

    hb_publisher = pyuavcan.application.heartbeat_publisher.HeartbeatPublisher(presentation)

    hb_publisher.start()

    loop = asyncio.get_event_loop()
    loop.run_forever()
