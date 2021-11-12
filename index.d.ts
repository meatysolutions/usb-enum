interface UsbDevice {
  id: string
  vendorId: number
  productId: number
  description: string | null
}
export function list(vendorId: number | null, productId: number | null): Promise<Array<UsbDevice>>
export class Watch {
  
  constructor(connected: (...args: any[]) => any | null, disconnected: (...args: any[]) => any | null, vendorId: number | null, productId: number | null)
}
