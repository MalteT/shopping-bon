use escpos_lib::Printer;
use serialport::{DataBits, FlowControl, Parity, StopBits};

fn main() {
    let port = serialport::new("/dev/serial0", 9600)
        .data_bits(DataBits::Eight)
        .parity(Parity::None)
        .stop_bits(StopBits::One)
        .flow_control(FlowControl::None)
        .open_native()
        .expect("Init serial failed");
    let mut printer = Printer::new(port).expect("Init writing failed");
    printer.print_test_page().expect("Test failed");
}
