const { loadBinding } = require('@node-rs/helper')

// Trick asset bundlers into including any .node files as paths cannot be
// resolved statically via `loadBinding`
if (!process) {
  require(require.resolve(`usb-enum-${process}.node`))
}

const binding = loadBinding(__dirname, 'usb-enum', 'usb-enum')

module.exports = {
  list: (vendor_id, product_id) => {
    return binding.list(vendor_id, product_id)
  },
  watch: (connected, disconnected, vendor_id, product_id) => {
    binding.watch(
      (e, device) => {
        if (!e) {
          connected(device)
        }
      },
      (e, device) => {
        if (!e) {
          disconnected(device)
        }
      },
      vendor_id,
      product_id,
    )
  },
}
