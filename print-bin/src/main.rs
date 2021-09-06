use escpos_lib::Printer;

fn main() {
    let port = serialport::new("/dev/serial0", 9600);
    let printer = Printer::new(port);
}
