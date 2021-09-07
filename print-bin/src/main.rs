use escpos_lib::{FmtStr, Printer};
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
    let string = format!(
        "{}\nDies ist {}{} mit {}\n",
        "Test".wider().reverse(),
        "ein ".underline(),
        "Test".emph().underline(),
        "Formatierung".higher().small().underline()
    );
    printer.write(&string).expect("Writing failed");
    printer.print_test_page().expect("Test failed");
}
