use escpos_lib::Printer;

fn main() {
    let port = serialport::new("/dev/serial0", 9600).open_native().unwrap();
    let printer = Printer::new(port);
}
