#### Suite of interoperability tests.
#
# We use pyuavcan as that is generally considered the reference implementation,
# and it is the easiest to write tooling for.
#
# Note that we don't test any DSDL generation here, simply message passing with
# arbitrary data, as that testing will be done inside of nunavut.

from typing import Callable
import asyncio
import argparse
import pathlib
import sys

# PyUAVCAN imports
import pyuavcan
import pyuavcan.transport.can
import pyuavcan.transport.can.media.socketcan

# DSDL compilation/imports
compiled_dsdl_dir = pathlib.Path(__file__).resolve().parent / ".test_dsdl_compiled"
sys.path.insert(0, str(compiled_dsdl_dir))

try:
    import uavcan.node
except (ImportError, AttributeError):
    print("Compiling DSDL, this may take a bit...")
    src_dir = pathlib.Path(__file__).resolve().parent / "public_regulated_data_types/uavcan"
    print(dir(pyuavcan.dsdl))
    pyuavcan.dsdl.compile_all([src_dir], output_directory=compiled_dsdl_dir)
    print("Finished compiling.")

_test_registry = {}
def register_test(test: Callable):
    """Adds test to registry so it can be called directly from CLI"""
    _test_registry.update({test.__name__: test})
    return test

@register_test
def hello_world():
    media = pyuavcan.transport.can.media.socketcan.SocketCANMedia("vcan0", 8)
    transport = pyuavcan.transport.can.CANTransport(media, 41)
    presentation = pyuavcan.presentation.Presentation(transport)

    pub = presentation.make_publisher(uavcan.node.Heartbeat_1_0, 100)

    async def run():
        await pub.publish(uavcan.node.Heartbeat_1_0(
            uptime = 0,
            health = uavcan.node.Health_1_0(uavcan.node.Health_1_0.NOMINAL),
            mode = uavcan.node.Mode_1_0(uavcan.node.Mode_1_0.OPERATIONAL),
            vendor_specific_status_code=0
        ))

    asyncio.run(run())


if __name__ == "__main__":
    args = argparse.ArgumentParser()
    args.add_argument("test", help="Test to run")
    opts = args.parse_args()

    # Run test
    _test_registry[opts.test]()

