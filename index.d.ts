interface UsbDevice {
  is: string
  vendor_id: number
  product_id: number
  description?: string
}

export const list: (vendor_id?: number, product_id?: number) => number
export const watch: (
  connected: (device: UsbDevice) => void,
  disconnected: (device: UsbDevice) => void,
  vendor_id?: number,
  product_id?: number,
) => Promise<number>
