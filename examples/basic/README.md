# Basic usage/testing example

For the moment, this more exists as an easy way to run end-to-end testing than
a proper usage example, but it should be kept relatively up to date as more of the API
comes up.

## Usage

To get CAN up and running on most Linux systems:

```
sudo ./setup_can.sh
```

The Python node should only be dependant on having pyuavcan installed.

```
python3 external_node.py
```

Then you can run the Rust node:

```
cargo run
```

And you should see text being printed in the Python node.
