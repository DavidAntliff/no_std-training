// To easily test this you can connect GPIO2 and GPIO4
// This way we will receive was we send. (loopback)

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    dma::{DmaRxBuf, DmaTxBuf},
    dma_buffers, main,
    spi::{
        master::{Config, Spi},
        Mode,
    },
    time::Rate,
};
use esp_println::{print, println};

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let sclk = peripherals.GPIO0;
    let miso = peripherals.GPIO2;
    let mosi = peripherals.GPIO4;
    let cs = peripherals.GPIO5;

    // we need to create the DMA driver and get a channel
    let dma_channel = peripherals.DMA_CH0;

    // DMA transfers need descriptors and buffers
    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(32000);
    let mut dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();
    let mut dma_tx_buf = DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();

    let mut spi = Spi::new(
        peripherals.SPI2,
        Config::default()
            .with_frequency(Rate::from_khz(100))
            .with_mode(Mode::_0),
    )
        .unwrap()
        .with_sck(sclk)
        .with_mosi(mosi)
        .with_miso(miso)
        .with_cs(cs)
        .with_dma(dma_channel);

    let delay = Delay::new();

    // The data to send:
    for (i, it) in dma_tx_buf.as_mut_slice().iter_mut().enumerate() {
        *it = i as u8;
    }

    loop {
        // ANCHOR: transfer
        // To transfer much larger amounts of data we can use DMA and
        // the CPU can even do other things while the transfer is in progress

        // `dma_transfer` will move the driver and the buffers into the
        // returned transfer.
        let transfer = spi
            .transfer(dma_rx_buf.len(), dma_rx_buf, dma_tx_buf.len(), dma_tx_buf)
            .map_err(|e| e.0)
            .unwrap();
        // let mut data = [0x01u8, 0x02, 0x03, 0x04];
        // spi.transfer(&mut data).unwrap();
        // ANCHOR_END: transfer

        // wait for transfer to complete
        while !transfer.is_done() {
            print!(".");
        }

        // get the results of the transfer via .wait()
        (spi, (dma_rx_buf, dma_tx_buf)) = transfer.wait();

        println!();
        println!("Received {:x?} .. {:x?}",
                 &dma_rx_buf.as_slice()[..10],
                 &dma_rx_buf.as_slice().last_chunk::<10>().unwrap()
        );

        delay.delay_millis(2500u32);
    }
}
