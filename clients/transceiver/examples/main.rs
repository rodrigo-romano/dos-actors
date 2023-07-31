use gmt_dos_clients_scope::Transceiver;

fn main() -> eframe::Result<()> {
    let mut scope = Transceiver::new();
    scope.append("data", &vec![0.5; 10]);
    scope.show()
}
