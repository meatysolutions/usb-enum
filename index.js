const binding = require('./usb-enum.node');

module.exports = {
  list: binding.list,
  watch: (connected, disconnected, vendor_id, product_id) => {
    binding.watch(
      (e, device) => {
        if (!e) connected(device);
      },
      (e, device) => {
        if (!e) disconnected(device);
      },
      vendor_id,
      product_id,
    );
  },
};
