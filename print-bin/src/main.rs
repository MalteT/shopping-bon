use escpos_lib::{FmtStr, Printer};
use serialport::{DataBits, FlowControl, Parity, StopBits};

use std::time::Duration;

fn main() {
    let port = serialport::new("/dev/serial0", 9600)
        .data_bits(DataBits::Eight)
        .parity(Parity::None)
        .stop_bits(StopBits::One)
        .flow_control(FlowControl::None)
        .timeout(Duration::from_secs(10))
        .open_native()
        .expect("Init serial failed");
    let mut printer = Printer::new(port).expect("Init writing failed");
    printer.print_test_page().expect("Test failed");
}
