use escpos_lib::Printer;
use serialport::{DataBits, Parity, StopBits, FlowControl};

fn main() {
    let port = serialport::new("/dev/serial0", 9600).data_bits(DataBits::Eight)
        .parity(Parity::None)
        .stop_bits(StopBits::One)
        .flow_control(FlowControl::None)
        .open_native()
        .unwrap();
    let printer = Printer::new(port);
}
